//! Type substitution over the patterns and completions of a generic body.

use super::super::ast::{Completion, Pattern};
use super::super::types::{runtime_type, substitute_type};
use super::Substitutions;
use mal_syntax::diagnostic::Diagnostic;

pub(super) fn pattern(
    value: &mut Pattern,
    substitutions: &Substitutions,
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
    substitutions: &Substitutions,
) -> Result<(), Diagnostic> {
    if let Completion::Value(value) = completion {
        value.ty = runtime_type(&substitute_type(&value.ty, substitutions, value.span)?);
    }
    Ok(())
}
