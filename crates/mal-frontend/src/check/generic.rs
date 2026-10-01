//! Generic value bindings: their principal parameter kinds, one check of the body under rigid parameters, and
//! the operation and kind requirements that the body leaves for specialization.

use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::source::Span;

use super::ast::Type;
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
        let previous_requirements = std::mem::take(&mut self.active_requirements);
        let previous_generic = self.active_generic.take();
        let previous_operations = std::mem::take(&mut self.active_operations);
        let previous_kinds = std::mem::take(&mut self.active_kinds);
        let result = (|| {
            let ty = self.expand_type(annotation)?;
            let requirements = types::storable_requirements(&ty);
            self.active_requirements = requirements.clone();
            self.active_generic = Some((
                binding.id,
                parameters.iter().map(|parameter| parameter.id).collect(),
            ));
            self.values.insert(binding.id, ty.clone());
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
            let checked_value = self.check_expression(value, Some(&ty))?;
            self.check_top_level_initializer(&checked_value)?;
            let operations = std::mem::take(&mut self.active_operations);
            self.generic_signatures
                .get_mut(&binding.id)
                .expect("active generic signature is registered")
                .operations = operations.clone();
            let kinds = std::mem::take(&mut self.active_kinds);
            Ok(ast::GenericBinding {
                binding: binding.clone(),
                parameters: parameters.to_vec(),
                ty,
                value: checked_value,
                operations,
                parameter_kinds: self.generic_signatures[&binding.id].parameter_kinds.clone(),
                kinds,
                span,
            })
        })();
        self.type_substitutions = previous;
        self.active_requirements = previous_requirements;
        self.active_generic = previous_generic;
        self.active_operations = previous_operations;
        self.active_kinds = previous_kinds;
        result
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
}
