//! Type substitution over the patterns and completions of a generic body.

use std::collections::HashMap;

use super::super::ast::{Completion, Pattern, Type};
use super::super::types::{runtime_type, substitute_type};

pub(super) fn pattern(
    value: &mut Pattern,
    substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
) {
    match value {
        Pattern::Binding { ty, .. } | Pattern::Wildcard { ty, .. } => {
            *ty = runtime_type(&substitute_type(ty, substitutions))
        }
        Pattern::Product { elements, ty, .. } => {
            *ty = runtime_type(&substitute_type(ty, substitutions));
            for element in elements {
                pattern(element, substitutions);
            }
        }
    }
}

pub(super) fn completion(
    completion: &mut Completion,
    substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
) {
    if let Completion::Value(value) = completion {
        value.ty = runtime_type(&substitute_type(&value.ty, substitutions));
    }
}
