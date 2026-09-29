//! Collection of source type declarations before canonical expansion.

use mal_syntax::ast::Node;

use crate::resolve::ast::{self as resolved};

use super::super::Checker;
use super::expand::used_parameters;

#[derive(Clone)]
pub(in crate::check) struct GenericAliasDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) parameters: Vec<resolved::TypeBinding>,
    pub(super) used_parameters: Vec<bool>,
    pub(super) value: Node<resolved::TypeExpression>,
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
