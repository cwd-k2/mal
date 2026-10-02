//! Generic value bindings: their principal parameter kinds, one check of the body under rigid parameters, and
//! the operation and kind requirements that the body leaves for specialization.

use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::source::Span;

use super::{CheckResult, Checker, GenericSignature, ast, types};

impl Checker {
    pub(super) fn check_generic_binding(
        &mut self,
        binding: &resolved::ValueBinding,
        parameters: &[resolved::TypeBinding],
        annotation: &Node<resolved::TypeExpression>,
        value: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<ast::GenericBinding> {
        let parameter_kinds =
            self.kinds
                .parameters(parameters, annotation, &mut self.next_kind_variable)?;
        let signature_parameter_kinds = parameter_kinds.clone();
        let substitutions = types::rigid_parameters(parameters, parameter_kinds);
        self.with_body(substitutions, |checker| {
            let ty = checker.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            checker.active_requirements = requirements.clone();
            checker.active_generic = Some((
                binding.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            checker.values.insert(binding.id, ty.clone());
            checker.generic_signatures.insert(
                binding.id,
                GenericSignature {
                    parameters: parameters.to_vec(),
                    parameter_kinds: signature_parameter_kinds,
                    ty: ty.clone(),
                    requirements,
                    operations: Vec::new(),
                },
            );
            let checked_value = checker.check_expression(value, Some(&ty))?;
            checker.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut checker.active_operations);
            checker
                .generic_signatures
                .get_mut(&binding.id)
                .expect("active generic signature is registered")
                .operations = operations.clone();
            let kinds = std::mem::take(&mut checker.active_kinds);
            Ok(ast::GenericBinding {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                value: checked_value,
                operations,
                parameter_kinds: checker.generic_signatures[&binding.id]
                    .parameter_kinds
                    .clone(),
                kinds,
                span,
            })
        })
    }

    /// Keeps the kind equations of a type application for the active generic body to check at specialization.
    pub(super) fn record_kinds(&mut self, requirements: Vec<ast::KindRequirement>) {
        if self.active_generic.is_none() {
            return;
        }
        for requirement in requirements {
            if !self.active_kinds.iter().any(|existing| {
                existing.left == requirement.left && existing.right == requirement.right
            }) {
                self.active_kinds.push(requirement);
            }
        }
    }

    /// Checks one declaration body under its rigid parameters with no requirements yet recorded, then restores the
    /// enclosing state, so that bodies checked while another is active do not share requirements.
    pub(super) fn with_body<T>(
        &mut self,
        substitutions: std::sync::Arc<
            std::collections::HashMap<crate::resolve::ast::TypeId, ast::Type>,
        >,
        check: impl FnOnce(&mut Self) -> CheckResult<T>,
    ) -> CheckResult<T> {
        let substitutions = std::mem::replace(&mut self.type_substitutions, substitutions);
        let requirements = std::mem::take(&mut self.active_requirements);
        let generic = self.active_generic.take();
        let operations = std::mem::take(&mut self.active_operations);
        let kinds = std::mem::take(&mut self.active_kinds);
        let result = check(self);
        self.type_substitutions = substitutions;
        self.active_requirements = requirements;
        self.active_generic = generic;
        self.active_operations = operations;
        self.active_kinds = kinds;
        result
    }
}
