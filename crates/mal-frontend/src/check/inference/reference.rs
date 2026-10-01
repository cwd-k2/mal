//! A generic reference whose type arguments are known: arity and kinds, self recursion, and the Storable,
//! operation, and kind requirements the enclosing generic body inherits from it.

use crate::resolve::ast as resolved;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{self, Expression, Type};
use super::super::{CheckResult, Checker, operation, types};

/// Only a family reference that still names a rigid type parameter is a requirement of the enclosing generic body;
/// a closed reference is selected directly during specialization.
fn is_open_requirement(arguments: &[Type]) -> bool {
    arguments.iter().any(operation::contains_parameter)
}

impl Checker {
    pub(in crate::check) fn check_generic_reference(
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
        let kinds = types::require_type_argument_kinds(
            &signature.parameter_kinds,
            &mut arguments,
            reference.name.span,
        )?;
        self.record_kinds(kinds);
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
            let required = types::substitute_type(required, &substitutions, reference.name.span)?;
            if !types::satisfies_storable_requirement(&required, &self.active_requirements) {
                return Err(
                    Diagnostic::error("generic application lacks a Storable requirement")
                        .with_primary(
                            reference.name.span,
                            format!(
                                "type argument `{}` is not known to be storable",
                                types::type_name(&required)
                            ),
                        )
                        .into(),
                );
            }
        }
        let kind = if self.operation_families.contains(&reference.id) {
            if self.active_generic.is_some()
                && is_open_requirement(&arguments)
                && !self.active_operations.iter().any(|requirement| {
                    requirement.family.id == reference.id && requirement.arguments == arguments
                })
            {
                self.active_operations.push(ast::OperationRequirement {
                    family: reference.clone(),
                    arguments: arguments.clone(),
                });
            }
            ast::ExpressionKind::OperationReference {
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
                            types::substitute_type(argument, &substitutions, reference.name.span)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if is_open_requirement(&requirement_arguments)
                        && !self.active_operations.iter().any(|existing| {
                            existing.family.id == requirement.family.id
                                && existing.arguments == requirement_arguments
                        })
                    {
                        self.active_operations.push(ast::OperationRequirement {
                            family: requirement.family.clone(),
                            arguments: requirement_arguments,
                        });
                    }
                }
            }
            ast::ExpressionKind::GenericReference {
                reference: reference.clone(),
                arguments,
            }
        };
        Ok(Expression {
            kind,
            ty: types::substitute_type(&signature.ty, &substitutions, reference.name.span)?,
            span,
        })
    }
}
