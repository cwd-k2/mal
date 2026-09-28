//! Local type-argument constraints for generic value references and calls.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast as resolved;
use crate::resolve::ast::TypeId;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{Expression, Type};
use super::float::is_contextual_float;
use super::integer::is_contextual_integer;
use super::{CheckResult, Checker, GenericSignature};

impl Checker {
    pub(super) fn check_generic_reference(
        &self,
        reference: &resolved::ValueReference,
        arguments: Vec<Type>,
        span: Span,
    ) -> CheckResult<Expression> {
        let signature = self
            .generic_signatures
            .get(&reference.id)
            .expect("generic reference has a collected signature");
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
        if self
            .active_generic
            .as_ref()
            .is_some_and(|(id, _)| *id == reference.id)
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
        for required in &signature.requirements {
            let index = signature
                .parameters
                .iter()
                .position(|parameter| parameter.id == *required)
                .expect("requirements refer to declared parameters");
            if !super::types::satisfies_storable_requirement(
                &arguments[index],
                &self.active_requirements,
            ) {
                return Err(
                    Diagnostic::error("generic application lacks a Storable requirement")
                        .with_primary(
                            reference.name.span,
                            format!(
                                "type argument `{}` is not known to be storable",
                                super::types::type_name(&arguments[index])
                            ),
                        )
                        .into(),
                );
            }
        }
        let substitutions = signature
            .parameters
            .iter()
            .map(|parameter| parameter.id)
            .zip(arguments.iter().cloned())
            .collect();
        Ok(Expression {
            kind: super::ast::ExpressionKind::GenericReference {
                reference: reference.clone(),
                arguments,
            },
            ty: super::types::substitute_type(&signature.ty, &substitutions),
            span,
        })
    }

    pub(super) fn check_inferred_generic_reference(
        &self,
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

    pub(super) fn check_inferred_generic_call(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let signature = self.generic_signatures[&reference.id].clone();
        let Type::Function { parameter, result } = &signature.ty else {
            return Err(Diagnostic::error("cannot call a non-function value")
                .with_primary(reference.name.span, "this generic value is not a function")
                .into());
        };
        let flexible = parameter_ids(&signature);
        let mut substitutions = HashMap::new();
        if let Some(expected) = expected {
            constrain(
                result,
                expected,
                &flexible,
                &mut substitutions,
                reference.name.span,
            )?;
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

    fn probe_constraint(
        &self,
        argument: &Node<resolved::Expression>,
        template: &Type,
        flexible: &HashSet<TypeId>,
        substitutions: &mut HashMap<TypeId, Type>,
        allow_defaults: bool,
    ) -> Result<(), Diagnostic> {
        let contextual = is_contextual_integer(argument) || is_contextual_float(argument);
        if contextual && !allow_defaults && has_unresolved(template, flexible, substitutions) {
            return Ok(());
        }
        let instantiated = super::types::substitute_type(template, substitutions);
        let unresolved = has_unresolved(&instantiated, flexible, substitutions);
        let checked = if let resolved::Expression::Lambda(lambda) = &argument.kind
            && let Type::Function { parameter, result } = &instantiated
            && !has_unresolved(parameter, flexible, substitutions)
        {
            let mut probe = self.clone();
            let expected_result =
                (!has_unresolved(result, flexible, substitutions)).then_some(result.as_ref());
            probe.check_lambda_against(
                lambda,
                argument.span,
                parameter.as_ref().clone(),
                expected_result,
            )
        } else if contextual {
            let mut probe = self.clone();
            if unresolved {
                probe.check_expression(argument, None)
            } else {
                probe.check_expression(argument, Some(&instantiated))
            }
        } else {
            let mut probe = self.clone();
            match probe.check_expression(argument, None) {
                Ok(checked) => Ok(checked),
                Err(_) if !unresolved => {
                    let mut contextual_probe = self.clone();
                    contextual_probe.check_expression(argument, Some(&instantiated))
                }
                Err(error) => Err(error),
            }
        };
        if let Ok(checked) = checked {
            constrain(
                template,
                &checked.ty,
                flexible,
                substitutions,
                argument.span,
            )?;
        }
        Ok(())
    }
}

fn parameter_ids(signature: &GenericSignature) -> HashSet<TypeId> {
    signature
        .parameters
        .iter()
        .map(|parameter| parameter.id)
        .collect()
}

fn argument_templates(parameter: &Type, count: usize) -> Option<Vec<&Type>> {
    match (count, parameter) {
        (0, Type::Unit) => Some(Vec::new()),
        (1, parameter) => Some(vec![parameter]),
        (_, Type::Product(elements)) if elements.len() == count => Some(elements.iter().collect()),
        _ => None,
    }
}

fn inferred_arguments(
    signature: &GenericSignature,
    substitutions: &HashMap<TypeId, Type>,
    span: Span,
) -> Result<Vec<Type>, super::CheckFailure> {
    let missing = signature
        .parameters
        .iter()
        .filter(|parameter| !substitutions.contains_key(&parameter.id))
        .map(|parameter| parameter.name.text.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(
            Diagnostic::error("generic type arguments cannot be inferred")
                .with_primary(
                    span,
                    format!("write explicit type arguments for {}", missing.join(", ")),
                )
                .into(),
        );
    }
    Ok(signature
        .parameters
        .iter()
        .map(|parameter| substitutions[&parameter.id].clone())
        .collect())
}

fn has_unresolved(
    ty: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    match ty {
        Type::Parameter { id, .. } => flexible.contains(id) && !substitutions.contains_key(id),
        Type::Buffer(element) => has_unresolved(element, flexible, substitutions),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| has_unresolved(element, flexible, substitutions)),
        Type::Function { parameter, result } => {
            has_unresolved(parameter, flexible, substitutions)
                || has_unresolved(result, flexible, substitutions)
        }
        _ => false,
    }
}

fn constrain(
    template: &Type,
    actual: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Type::Parameter { id, .. } = template
        && flexible.contains(id)
    {
        if let Some(previous) = substitutions.get(id) {
            if previous == actual {
                return Ok(());
            }
            return Err(inference_conflict(previous, actual, span));
        }
        substitutions.insert(*id, actual.clone());
        return Ok(());
    }
    match (template, actual) {
        (Type::Buffer(left), Type::Buffer(right)) => {
            constrain(left, right, flexible, substitutions, span)
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            for (left, right) in left.iter().zip(right.iter()) {
                constrain(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (
            Type::Function {
                parameter: left_parameter,
                result: left_result,
            },
            Type::Function {
                parameter: right_parameter,
                result: right_result,
            },
        ) => {
            constrain(
                left_parameter,
                right_parameter,
                flexible,
                substitutions,
                span,
            )?;
            constrain(left_result, right_result, flexible, substitutions, span)
        }
        _ if template == actual => Ok(()),
        _ => Err(inference_conflict(template, actual, span)),
    }
}

fn inference_conflict(expected: &Type, actual: &Type, span: Span) -> Diagnostic {
    Diagnostic::error("conflicting generic type inference").with_primary(
        span,
        format!(
            "inferred both `{}` and `{}`",
            super::types::type_name(expected),
            super::types::type_name(actual)
        ),
    )
}
