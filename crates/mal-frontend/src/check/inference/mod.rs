//! Generic value reference and call checking from local type constraints.

use std::collections::HashMap;

use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{Expression, Type};
use super::{CheckResult, Checker};

enum GenericCallExpectation<'a> {
    Result(Option<&'a Type>),
    ReturnedFunctionParameter(&'a Type),
}

mod arguments;
mod constraint;
#[cfg(test)]
mod constraint_tests;
mod probe;

use arguments::{argument_templates, inferred_arguments, parameter_ids};
use constraint::constrain;

impl Checker {
    pub(super) fn check_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        mut arguments: Vec<Type>,
        span: Span,
    ) -> CheckResult<Expression> {
        let signature = self
            .generic_signatures
            .get(&reference.id)
            .expect("generic reference has a collected signature")
            .clone();
        if arguments.len() != signature.parameters.len() {
            return Err(Diagnostic::error("generic value argument arity mismatch")
                .with_primary(
                    reference.name.span,
                    format!(
                        "expected {} arguments but found {}",
                        signature.parameters.len(),
                        arguments.len()
                    ),
                )
                .into());
        }
        super::types::require_type_argument_kinds(
            &signature.parameter_kinds,
            &mut arguments,
            reference.name.span,
        )?;
        if self
            .active_generic
            .as_ref()
            .is_some_and(|(id, _)| *id == reference.id)
            && !self.operation_families.contains(&reference.id)
        {
            let (_, parameters) = self.active_generic.as_ref().unwrap();
            let same_key = arguments.iter().zip(parameters).all(|(argument, parameter)| {
                matches!(argument, Type::Parameter { id, .. } if id == parameter)
            });
            if !same_key {
                return Err(Diagnostic::error("polymorphic recursion is not supported")
                    .with_primary(
                        reference.name.span,
                        "self recursion must preserve the type argument list",
                    )
                    .into());
            }
        }
        let substitutions = signature
            .parameters
            .iter()
            .map(|parameter| parameter.id)
            .zip(arguments.iter().cloned())
            .collect();
        for required in &signature.requirements {
            let required =
                super::types::substitute_type(required, &substitutions, reference.name.span)?;
            if !super::types::satisfies_storable_requirement(&required, &self.active_requirements) {
                return Err(
                    Diagnostic::error("generic application lacks a Storable requirement")
                        .with_primary(
                            reference.name.span,
                            format!(
                                "type argument `{}` is not known to be storable",
                                super::types::type_name(&required)
                            ),
                        )
                        .into(),
                );
            }
        }
        let kind = if self.operation_families.contains(&reference.id) {
            if self.active_generic.is_some()
                && !self.active_operations.iter().any(|requirement| {
                    requirement.family.id == reference.id && requirement.arguments == arguments
                })
            {
                self.active_operations
                    .push(super::ast::OperationRequirement {
                        family: reference.clone(),
                        arguments: arguments.clone(),
                    });
            }
            super::ast::ExpressionKind::OperationReference {
                family: reference.clone(),
                arguments,
            }
        } else {
            if self.active_generic.is_some() {
                for requirement in &signature.operations {
                    let requirement_arguments = requirement
                        .arguments
                        .iter()
                        .map(|argument| {
                            super::types::substitute_type(
                                argument,
                                &substitutions,
                                reference.name.span,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if !self.active_operations.iter().any(|existing| {
                        existing.family.id == requirement.family.id
                            && existing.arguments == requirement_arguments
                    }) {
                        self.active_operations
                            .push(super::ast::OperationRequirement {
                                family: requirement.family.clone(),
                                arguments: requirement_arguments,
                            });
                    }
                }
            }
            super::ast::ExpressionKind::GenericReference {
                reference: reference.clone(),
                arguments,
            }
        };
        Ok(Expression {
            kind,
            ty: super::types::substitute_type(&signature.ty, &substitutions, reference.name.span)?,
            span,
        })
    }

    pub(super) fn check_inferred_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        if let Some(expected) = expected {
            constrain(
                &signature.ty,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?;
        }
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_explicit_generic_reference(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let mut signature = self.generic_signatures[&reference.id].clone();
        if type_arguments.len() > signature.parameters.len() {
            return Err(Diagnostic::error("generic value argument arity mismatch")
                .with_primary(
                    reference.name.span,
                    format!(
                        "expected at most {} arguments but found {}",
                        signature.parameters.len(),
                        type_arguments.len()
                    ),
                )
                .into());
        }
        let mut explicit = type_arguments
            .iter()
            .map(|argument| self.expand_type_term(argument))
            .collect::<Result<Vec<_>, _>>()?;
        super::types::require_type_argument_kinds(
            &signature.parameter_kinds[..explicit.len()],
            &mut explicit,
            reference.name.span,
        )?;
        (signature.parameter_kinds, signature.ty) = super::types::instantiate_signature_kinds(
            &signature.parameter_kinds,
            &explicit,
            &signature.ty,
            reference.name.span,
        )?;
        let mut substitutions = signature
            .parameters
            .iter()
            .zip(explicit)
            .map(|(parameter, argument)| (parameter.id, argument))
            .collect::<HashMap<_, _>>();
        if let Some(expected) = expected {
            let instantiated =
                super::types::substitute_type(&signature.ty, &substitutions, reference.name.span)?;
            constrain(
                &instantiated,
                expected,
                &parameter_ids(&signature),
                &mut substitutions,
                reference.name.span,
            )?;
        }
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_inferred_generic_continuation_reference(
        &mut self,
        reference: &resolved::ValueReference,
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let Type::Function {
            parameter: parameter_template,
            ..
        } = &signature.ty
        else {
            return self.check_inferred_generic_reference(reference, span, None);
        };
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        constrain(
            parameter_template,
            parameter,
            &flexible,
            &mut substitutions,
            reference.name.span,
        )?;
        let arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        self.check_generic_reference(reference, arguments, span)
    }

    pub(super) fn check_inferred_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            &[],
            arguments,
            span,
            GenericCallExpectation::Result(expected),
        )
    }

    pub(super) fn check_inferred_generic_continuation_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        parameter: &Type,
    ) -> CheckResult<Expression> {
        self.check_inferred_generic_call_with_expectations(
            reference,
            &[],
            arguments,
            span,
            GenericCallExpectation::ReturnedFunctionParameter(parameter),
        )
    }

    pub(super) fn check_explicit_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        if type_arguments.len() > signature.parameters.len() {
            return Err(Diagnostic::error("generic value argument arity mismatch")
                .with_primary(
                    reference.name.span,
                    format!(
                        "expected at most {} arguments but found {}",
                        signature.parameters.len(),
                        type_arguments.len()
                    ),
                )
                .into());
        }
        let mut explicit = type_arguments
            .iter()
            .map(|argument| self.expand_type_term(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if self
            .active_generic
            .as_ref()
            .is_some_and(|(id, _)| *id == reference.id)
            && !self.operation_families.contains(&reference.id)
        {
            let (_, parameters) = self.active_generic.as_ref().unwrap();
            let preserves_key = explicit.iter().zip(parameters).all(|(argument, parameter)| {
                matches!(argument, Type::Parameter { id, .. } if id == parameter)
            });
            if !preserves_key {
                return Err(Diagnostic::error("polymorphic recursion is not supported")
                    .with_primary(
                        reference.name.span,
                        "self recursion must preserve the type argument list",
                    )
                    .into());
            }
        }
        super::types::require_type_argument_kinds(
            &signature.parameter_kinds[..explicit.len()],
            &mut explicit,
            reference.name.span,
        )?;
        self.check_inferred_generic_call_with_expectations(
            reference,
            &explicit,
            arguments,
            span,
            GenericCallExpectation::Result(expected),
        )
    }

    fn check_inferred_generic_call_with_expectations(
        &mut self,
        reference: &resolved::ValueReference,
        explicit: &[Type],
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: GenericCallExpectation<'_>,
    ) -> CheckResult<Expression> {
        let mut signature = self.generic_signatures[&reference.id].clone();
        (signature.parameter_kinds, signature.ty) = super::types::instantiate_signature_kinds(
            &signature.parameter_kinds,
            explicit,
            &signature.ty,
            reference.name.span,
        )?;
        let mut substitutions = signature
            .parameters
            .iter()
            .zip(explicit)
            .map(|(parameter, argument)| (parameter.id, argument.clone()))
            .collect::<HashMap<_, _>>();
        let instantiated =
            super::types::substitute_type(&signature.ty, &substitutions, reference.name.span)?;
        let Type::Function { parameter, result } = &instantiated else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(reference.name.span, "this generic value is not a function")
                .into());
        };
        let flexible = parameter_ids(&signature);
        match expected {
            GenericCallExpectation::Result(Some(expected)) => constrain(
                result,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?,
            GenericCallExpectation::ReturnedFunctionParameter(expected)
                if let Type::Function { parameter, .. } = result.as_ref() =>
            {
                constrain(
                    parameter,
                    expected,
                    &flexible,
                    &mut substitutions,
                    reference.name.span,
                )?;
            }
            GenericCallExpectation::Result(None)
            | GenericCallExpectation::ReturnedFunctionParameter(_) => {}
        }

        let templates = argument_templates(parameter, arguments.len());
        let mut allow_defaults = false;
        loop {
            let before = substitutions.len();
            match &templates {
                Some(templates) => {
                    for (argument, template) in arguments.iter().zip(templates) {
                        self.probe_constraint(
                            argument,
                            template,
                            &flexible,
                            &mut substitutions,
                            allow_defaults,
                        )?;
                    }
                }
                None => {
                    let mut probe = self.clone();
                    if let Ok(actual) = probe.check_untyped_argument(arguments, span) {
                        constrain(parameter, &actual.ty, &flexible, &mut substitutions, span)?;
                    }
                }
            }
            if substitutions.len() == before {
                if allow_defaults {
                    break;
                }
                allow_defaults = true;
            }
        }

        let type_arguments = inferred_arguments(&signature, &substitutions, reference.name.span)?;
        let callee =
            self.check_generic_reference(reference, type_arguments, reference.name.span)?;
        let Type::Function { parameter, result } = &callee.ty else {
            unreachable!("the generic signature was a function");
        };
        let argument = match self.check_argument(arguments, parameter, span) {
            Ok(argument) => argument,
            Err(super::CheckFailure::Abrupt(_)) => {
                return Err(Diagnostic::error(
                    "function value is unreachable after abrupt argument evaluation",
                )
                .with_primary(callee.span, "this callee cannot be evaluated")
                .into());
            }
            Err(error) => return Err(error),
        };
        Ok(Expression {
            ty: result.as_ref().clone(),
            kind: super::ast::ExpressionKind::Call {
                callee: Box::new(callee),
                argument: Box::new(argument),
            },
            span,
        })
    }
}
