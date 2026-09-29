//! Collection of source type declarations before canonical expansion.

use mal_syntax::ast::Node;

use crate::resolve::ast::{self as resolved};

use super::super::Checker;

#[derive(Clone)]
pub(in crate::check) struct GenericAliasDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) parameters: Vec<resolved::TypeBinding>,
    pub(super) used_parameters: Vec<bool>,
    pub(super) value: Node<resolved::TypeExpression>,
}

fn used_parameters(
    parameters: &[resolved::TypeBinding],
    value: &Node<resolved::TypeExpression>,
) -> Vec<bool> {
    let positions = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.id, index))
        .collect::<std::collections::HashMap<_, _>>();
    let mut used = vec![false; parameters.len()];
    let mut pending = vec![value];
    while let Some(expression) = pending.pop() {
        match &expression.kind {
            resolved::TypeExpression::Named(reference) => {
                if let Some(index) = positions.get(&reference.id) {
                    used[*index] = true;
                }
            }
            resolved::TypeExpression::Application {
                constructor,
                arguments,
            } => {
                if let Some(index) = positions.get(&constructor.id) {
                    used[*index] = true;
                }
                pending.extend(arguments);
            }
            resolved::TypeExpression::Product(arguments)
            | resolved::TypeExpression::Sum(arguments) => pending.extend(arguments),
            resolved::TypeExpression::Parenthesized(inner) => pending.push(inner),
            resolved::TypeExpression::Function { parameter, result } => {
                pending.push(parameter);
                pending.push(result);
            }
            resolved::TypeExpression::Unit => {}
        }
    }
    used
}

#[derive(Clone)]
pub(in crate::check) struct OpaqueDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) parameters: Vec<resolved::TypeBinding>,
    pub(super) representation: Node<resolved::TypeExpression>,
}

impl Checker {
    pub(in crate::check) fn collect_aliases(&mut self, program: &resolved::Program) {
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    self.aliases.insert(binding.id, value.clone());
                }
                resolved::TopItem::GenericTypeAlias {
                    binding,
                    parameters,
                    value,
                } => {
                    self.generic_aliases.insert(
                        binding.id,
                        GenericAliasDefinition {
                            binding: binding.clone(),
                            parameters: parameters.clone(),
                            used_parameters: used_parameters(parameters, value),
                            value: value.clone(),
                        },
                    );
                }
                resolved::TopItem::OpaqueType {
                    binding,
                    parameters,
                    representation,
                } => {
                    self.opaque_types.insert(
                        binding.id,
                        OpaqueDefinition {
                            binding: binding.clone(),
                            parameters: parameters.clone(),
                            representation: representation.clone(),
                        },
                    );
                }
                resolved::TopItem::ExternalType { binding } => {
                    self.external_types.insert(binding.id, binding.clone());
                }
                _ => {}
            }
        }
    }
}
