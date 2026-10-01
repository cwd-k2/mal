//! Operation-family declaration, implementation coherence, and termination checking.

use crate::resolve::ast as resolved;
use mal_syntax::{ast::Node, diagnostic::Diagnostic, source::Span};

use super::{CheckResult, Checker, GenericSignature, ast, ast::Type, types};

mod pattern;

pub(super) use pattern::contains_parameter;
use pattern::{contains_parameter_id, operation_patterns_overlap, operation_requirement_decreases};

impl Checker {
    pub(super) fn check_operation_family(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        span: Span,
    ) -> CheckResult<ast::OperationFamily> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let signature_parameter_kinds = parameter_kinds.clone();
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .zip(parameter_kinds)
                .map(|(parameter, kind)| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                            kind,
                        },
                    )
                })
                .collect(),
        );
        let previous = std::mem::replace(&mut self.type_substitutions, substitutions);
        let result = (|| {
            let ty = self.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            self.generic_signatures.insert(
                binding.id,
                GenericSignature {
                    parameters: parameters.to_vec(),
                    parameter_kinds: signature_parameter_kinds,
                    ty: ty.clone(),
                    requirements,
                    operations: Vec::new(),
                },
            );
            self.operation_families.insert(binding.id);
            Ok(ast::OperationFamily {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                span,
            })
        })();
        self.type_substitutions = previous;
        result
    }

    pub(super) fn check_operation_implementation(
        &mut self,
        family: &resolved::ValueReference,
        parameters: &[resolved::TypeBinding],
        arguments: &[Node<resolved::TypeExpression>],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::OperationImplementation> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let substitutions = std::sync::Arc::new(
            parameters
                .iter()
                .zip(parameter_kinds.iter().cloned())
                .map(|(parameter, kind)| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                            kind,
                        },
                    )
                })
                .collect(),
        );
        let previous_substitutions = std::mem::replace(&mut self.type_substitutions, substitutions);
        let previous_generic = self.active_generic.take();
        let previous_operations = std::mem::take(&mut self.active_operations);
        let previous_requirements = std::mem::take(&mut self.active_requirements);
        let previous_kinds = std::mem::take(&mut self.active_kinds);
        let result = (|| {
            let mut arguments = arguments
                .iter()
                .map(|argument| self.expand_type_term(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let signature = self
                .generic_signatures
                .get(&family.id)
                .expect("an implementation refers to a checked operation family");
            let key_kinds = types::require_type_argument_kinds(
                &signature.parameter_kinds,
                &mut arguments,
                family.name.span,
            )?;
            if signature
                .parameter_kinds
                .iter()
                .zip(&arguments)
                .any(|(kind, argument)| {
                    matches!(kind, ast::Kind::Function { .. }) && contains_parameter(argument)
                })
            {
                return Err(
                    Diagnostic::error("operation constructor key must be closed")
                        .with_primary(
                            family.name.span,
                            "replace the constructor parameter with a declared type constructor",
                        )
                        .into(),
                );
            }
            if parameters.is_empty() && arguments.iter().any(contains_parameter) {
                return Err(Diagnostic::error(
                    "exact operation implementation requires closed types",
                )
                .with_primary(family.name.span, "remove generic parameters from this key")
                .into());
            }
            if !parameters.is_empty()
                && arguments
                    .iter()
                    .all(|argument| matches!(argument, Type::Parameter { .. }))
            {
                return Err(Diagnostic::error(
                    "generic operation implementation requires structure",
                )
                .with_primary(
                    family.name.span,
                    "a catch-all parameter key is not supported",
                )
                .into());
            }
            if let Some(parameter) = parameters.iter().find(|parameter| {
                !arguments
                    .iter()
                    .any(|argument| contains_parameter_id(argument, parameter.id))
            }) {
                return Err(Diagnostic::error(
                    "generic operation pattern leaves a parameter unbound",
                )
                .with_primary(
                    parameter.name.span,
                    "use this family parameter in the implementation key",
                )
                .into());
            }
            if self.operation_keys.iter().any(|(id, existing)| {
                *id == family.id && operation_patterns_overlap(existing, &arguments)
            }) {
                return Err(Diagnostic::error("duplicate operation implementation")
                    .with_primary(
                        family.name.span,
                        "this family key overlaps an existing implementation",
                    )
                    .into());
            }
            let expected = self
                .check_generic_reference(family, arguments.clone(), family.name.span)?
                .ty;
            let declared = self.expand_type(annotation)?;
            self.require_type(&declared, &expected, annotation.span)?;
            // Like a generic binding, the body may assume what its signature makes well formed.
            self.active_requirements = types::storable_requirements(&expected);
            self.active_generic = (!parameters.is_empty()).then_some((
                family.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            let checked_value = self.check_expression(value, Some(&expected))?;
            self.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut self.active_operations);
            if !parameters.is_empty() {
                for requirement in &operations {
                    let direct_self =
                        requirement.family.id == family.id && requirement.arguments == arguments;
                    if !direct_self
                        && !operation_requirement_decreases(&arguments, &requirement.arguments)
                    {
                        return Err(Diagnostic::error(
                            "generic operation requirement does not decrease",
                        )
                        .with_primary(
                            requirement.family.name.span,
                            "the required key must be a proper subterm of the implementation key",
                        )
                        .into());
                    }
                }
            }
            let mut kinds = key_kinds;
            for requirement in std::mem::take(&mut self.active_kinds) {
                if !kinds.iter().any(|existing| {
                    existing.left == requirement.left && existing.right == requirement.right
                }) {
                    kinds.push(requirement);
                }
            }
            self.operation_keys.push((family.id, arguments.clone()));
            Ok(ast::OperationImplementation {
                family: family.clone(),
                parameters: parameters.to_vec(),
                arguments,
                ty: expected,
                value: checked_value,
                operations,
                parameter_kinds,
                kinds,
                span,
            })
        })();
        self.type_substitutions = previous_substitutions;
        self.active_generic = previous_generic;
        self.active_operations = previous_operations;
        self.active_requirements = previous_requirements;
        self.active_kinds = previous_kinds;
        result
    }
}
