use std::collections::{HashMap, HashSet};

use crate::check::ast::Type;
use crate::check::{CheckFailure, GenericSignature};
use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

use super::constraint::resolve_substitution;

pub(super) fn parameter_ids(signature: &GenericSignature) -> HashSet<TypeId> {
    signature
        .parameters
        .iter()
        .map(|parameter| parameter.id)
        .collect()
}

pub(super) fn argument_templates(parameter: &Type, count: usize) -> Option<Vec<&Type>> {
    match (count, parameter) {
        (0, Type::Unit) => Some(Vec::new()),
        (1, parameter) => Some(vec![parameter]),
        (_, Type::Product(elements)) if elements.len() == count => Some(elements.iter().collect()),
        _ => None,
    }
}

pub(super) fn inferred_arguments(
    signature: &GenericSignature,
    substitutions: &HashMap<TypeId, Type>,
    span: Span,
) -> Result<Vec<Type>, CheckFailure> {
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
    let mut arguments = signature
        .parameters
        .iter()
        .map(|parameter| {
            resolve_substitution(parameter.id, substitutions, &mut HashSet::new(), span)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Inferred kind-polymorphic arguments may share kind variables with other parameters, so they
    // pass the same kind check as explicit arguments.
    crate::check::types::require_type_argument_kinds(
        &signature.parameter_kinds,
        &mut arguments,
        span,
    )?;
    Ok(arguments)
}
