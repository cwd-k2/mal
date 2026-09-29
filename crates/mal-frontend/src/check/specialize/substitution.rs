//! Type substitution over the patterns and completions of a generic body.

use std::collections::HashMap;

use super::super::ast::{Completion, Pattern, Type};
use super::super::types::{runtime_type, substitute_type};
use mal_syntax::diagnostic::Diagnostic;

pub(super) fn pattern(
    value: &mut Pattern,
    substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
) -> Result<(), Diagnostic> {
    match value {
        Pattern::Binding { binding, ty } => {
            *ty = runtime_type(&substitute_type(ty, substitutions, binding.name.span)?)
        }
        Pattern::Wildcard { ty, span } => {
            *ty = runtime_type(&substitute_type(ty, substitutions, *span)?)
        }
        Pattern::Product { elements, ty, span } => {
            *ty = runtime_type(&substitute_type(ty, substitutions, *span)?);
            for element in elements {
                pattern(element, substitutions)?;
            }
        }
    }
    Ok(())
}

pub(super) fn completion(
    completion: &mut Completion,
    substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
) -> Result<(), Diagnostic> {
    if let Completion::Value(value) = completion {
        value.ty = runtime_type(&substitute_type(&value.ty, substitutions, value.span)?);
    }
    Ok(())
}
