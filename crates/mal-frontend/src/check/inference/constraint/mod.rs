//! Structural constraints and substitution resolution for generic inference.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

use super::super::ast::{Kind, Type};

mod scheme;
mod substitution;

pub(super) use scheme::constrain_generic_scheme;
pub(super) use substitution::resolve_substitution;

fn contains_unbound_from(
    ty: &Type,
    parameters: &HashSet<TypeId>,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    match ty {
        Type::Parameter { id, .. } => parameters.contains(id) && !substitutions.contains_key(id),
        Type::Application {
            constructor,
            argument,
            ..
        } => {
            contains_unbound_from(constructor, parameters, substitutions)
                || contains_unbound_from(argument, parameters, substitutions)
        }
        Type::Abstraction { body, .. } => contains_unbound_from(body, parameters, substitutions),
        Type::Buffer(element) => contains_unbound_from(element, parameters, substitutions),
        Type::Opaque { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_unbound_from(argument, parameters, substitutions)),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| contains_unbound_from(element, parameters, substitutions)),
        Type::Function { parameter, result } => {
            contains_unbound_from(parameter, parameters, substitutions)
                || contains_unbound_from(result, parameters, substitutions)
        }
        _ => false,
    }
}

pub(super) fn has_unresolved(
    ty: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &HashMap<TypeId, Type>,
) -> bool {
    match ty {
        Type::Parameter { id, .. } => flexible.contains(id) && !substitutions.contains_key(id),
        Type::Application {
            constructor,
            argument,
            ..
        } => {
            has_unresolved(constructor, flexible, substitutions)
                || has_unresolved(argument, flexible, substitutions)
        }
        Type::Abstraction { body, .. } => has_unresolved(body, flexible, substitutions),
        Type::Buffer(element) => has_unresolved(element, flexible, substitutions),
        Type::Opaque { arguments, .. } => arguments
            .iter()
            .any(|argument| has_unresolved(argument, flexible, substitutions)),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| has_unresolved(element, flexible, substitutions)),
        Type::Function { parameter, result } => {
            has_unresolved(parameter, flexible, substitutions)
                || has_unresolved(result, flexible, substitutions)
        }
        _ => false,
    }
}

pub(super) fn constrain(
    template: &Type,
    actual: &Type,
    flexible: &HashSet<TypeId>,
    substitutions: &mut HashMap<TypeId, Type>,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Type::Parameter {
        id,
        kind: Kind::Type,
        ..
    } = template
        && flexible.contains(id)
    {
        if let Some(previous) = substitutions.get(id) {
            if matches!(previous, Type::Parameter { id: previous, .. } if previous == id) {
                substitutions.insert(*id, actual.clone());
                return Ok(());
            }
            if previous == actual || same_parameter(previous, actual) {
                return Ok(());
            }
            return Err(inference_conflict(previous, actual, span));
        }
        substitutions.insert(*id, actual.clone());
        return Ok(());
    }
    // Only kind `Type` arguments are inferred. Leaving any other one unresolved reports the missing
    // explicit argument instead of a conflict between two uses of the same parameter.
    if let Type::Parameter { id, .. } = template
        && flexible.contains(id)
        && !substitutions.contains_key(id)
    {
        return Ok(());
    }
    // Two different opaque types meet only when one side, viewed through opaque layers declared in
    // this file, reaches the other; types that merely share a representation stay distinct.
    if let (
        Type::Opaque {
            id: template_id, ..
        },
        Type::Opaque { id: actual_id, .. },
    ) = (template, actual)
        && template_id != actual_id
    {
        let layers = super::super::types::layer_with_identity;
        if let Some(view) = layers(template, *actual_id, span.file()) {
            return constrain(view, actual, flexible, substitutions, span);
        }
        if let Some(view) = layers(actual, *template_id, span.file()) {
            return constrain(template, view, flexible, substitutions, span);
        }
    }
    let both_opaque = matches!(
        (template, actual),
        (Type::Opaque { .. }, Type::Opaque { .. })
    );
    let template_view = if both_opaque {
        template
    } else {
        super::super::types::representation_view(template, span.file())
    };
    let actual_view = if both_opaque {
        actual
    } else {
        super::super::types::representation_view(actual, span.file())
    };
    if !std::ptr::eq(template, template_view) || !std::ptr::eq(actual, actual_view) {
        return constrain(template_view, actual_view, flexible, substitutions, span);
    }
    match (template, actual) {
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
            constrain(
                left_constructor,
                right_constructor,
                flexible,
                substitutions,
                span,
            )?;
            constrain(left_argument, right_argument, flexible, substitutions, span)
        }
        (Type::Application { .. }, _) if has_unresolved(template, flexible, substitutions) => {
            Ok(())
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
            constrain(left_body, right_body, flexible, substitutions, span)
        }
        (Type::Buffer(left), Type::Buffer(right)) => {
            constrain(left, right, flexible, substitutions, span)
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
                constrain(left, right, flexible, substitutions, span)?;
            }
            Ok(())
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            for (left, right) in left.iter().zip(right.iter()) {
                constrain(left, right, flexible, substitutions, span)?;
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
            constrain(
                left_parameter,
                right_parameter,
                flexible,
                substitutions,
                span,
            )?;
            constrain(left_result, right_result, flexible, substitutions, span)
        }
        _ if template == actual => Ok(()),
        _ => Err(inference_conflict(template, actual, span)),
    }
}

fn same_parameter(left: &Type, right: &Type) -> bool {
    matches!(
        (left, right),
        (Type::Parameter { id: left, .. }, Type::Parameter { id: right, .. }) if left == right
    )
}

fn inference_conflict(expected: &Type, actual: &Type, span: Span) -> Diagnostic {
    Diagnostic::error("conflicting generic type inference").with_primary(
        span,
        format!(
            "inferred both `{}` and `{}`",
            super::super::types::type_name(expected),
            super::super::types::type_name(actual)
        ),
    )
}
