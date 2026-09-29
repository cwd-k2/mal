use std::collections::{HashMap, HashSet};

use crate::check::GenericSignature;
use crate::check::ast::{Kind, Type};
use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

use super::{contains_unbound_from, inference_conflict, resolve_substitution};
use crate::check::inference::parameter_ids;

pub(in crate::check::inference) fn constrain_generic_scheme(
    signature: &GenericSignature,
    expected: &Type,
    outer_flexible: &HashSet<TypeId>,
    outer_substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    let inner_flexible = parameter_ids(signature);
    let flexible = outer_flexible
        .union(&inner_flexible)
        .copied()
        .collect::<HashSet<_>>();
    let mut substitutions = outer_substitutions.clone();
    unify_flexible(&signature.ty, expected, &flexible, &mut substitutions, span)?;
    for id in outer_flexible {
        if substitutions.contains_key(id) {
            let resolved = resolve_substitution(*id, &substitutions, &mut HashSet::new());
            if !contains_unbound_from(&resolved, &inner_flexible, &substitutions) {
                outer_substitutions.insert(*id, resolved);
            }
        }
    }
    Ok(())
}

fn unify_flexible(
    left: &Type,
    right: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Type::Parameter {
        id,
        kind: Kind::Type,
        ..
    } = left
        && flexible.contains(id)
    {
        if let Some(bound) = substitutions.get(id).cloned() {
            return unify_flexible(&bound, right, flexible, substitutions, span);
        }
        substitutions.insert(*id, right.clone());
        return Ok(());
    }
    if let Type::Parameter {
        id,
        kind: Kind::Type,
        ..
    } = right
        && flexible.contains(id)
    {
        if let Some(bound) = substitutions.get(id).cloned() {
            return unify_flexible(left, &bound, flexible, substitutions, span);
        }
        substitutions.insert(*id, left.clone());
        return Ok(());
    }
    let both_opaque = matches!((left, right), (Type::Opaque { .. }, Type::Opaque { .. }));
    let left_view = if both_opaque {
        left
    } else {
        crate::check::types::representation_view(left, span.file())
    };
    let right_view = if both_opaque {
        right
    } else {
        crate::check::types::representation_view(right, span.file())
    };
    if !std::ptr::eq(left, left_view) || !std::ptr::eq(right, right_view) {
        return unify_flexible(left_view, right_view, flexible, substitutions, span);
    }
    match (left, right) {
        (Type::Parameter { id: left, .. }, Type::Parameter { id: right, .. }) if left == right => {
            Ok(())
        }
        (
            Type::Application {
                constructor: left_constructor,
                argument: left_argument,
                kind: left_kind,
                ..
            },
            Type::Application {
                constructor: right_constructor,
                argument: right_argument,
                kind: right_kind,
                ..
            },
        ) if left_kind == right_kind => {
            unify_flexible(
                left_constructor,
                right_constructor,
                flexible,
                substitutions,
                span,
            )?;
            unify_flexible(left_argument, right_argument, flexible, substitutions, span)
        }
        (
            Type::Abstraction {
                parameter_kind: left_kind,
                body: left_body,
            },
            Type::Abstraction {
                parameter_kind: right_kind,
                body: right_body,
            },
        ) if left_kind == right_kind => {
            unify_flexible(left_body, right_body, flexible, substitutions, span)
        }
        (Type::Buffer(left), Type::Buffer(right)) => {
            unify_flexible(left, right, flexible, substitutions, span)
        }
        (
            Type::Opaque {
                id: left_id,
                arguments: left,
                ..
            },
            Type::Opaque {
                id: right_id,
                arguments: right,
                ..
            },
        ) if left_id == right_id && left.len() == right.len() => {
            for (left, right) in left.iter().zip(right.iter()) {
                unify_flexible(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            for (left, right) in left.iter().zip(right.iter()) {
                unify_flexible(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (
            Type::Function {
                parameter: left_parameter,
                result: left_result,
            },
            Type::Function {
                parameter: right_parameter,
                result: right_result,
            },
        ) => {
            unify_flexible(
                left_parameter,
                right_parameter,
                flexible,
                substitutions,
                span,
            )?;
            unify_flexible(left_result, right_result, flexible, substitutions, span)
        }
        _ if left == right => Ok(()),
        _ => Err(inference_conflict(left, right, span)),
    }
}
