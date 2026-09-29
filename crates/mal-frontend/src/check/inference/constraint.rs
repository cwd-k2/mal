//! Structural constraints and substitution resolution for generic inference.

use std::collections::{HashMap, HashSet};

use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

use super::super::GenericSignature;
use super::super::ast::{Kind, Type};
use super::parameter_ids;

pub(super) fn constrain_generic_scheme(
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
        super::super::types::representation_view(left, span.file())
    };
    let right_view = if both_opaque {
        right
    } else {
        super::super::types::representation_view(right, span.file())
    };
    if !std::ptr::eq(left, left_view) || !std::ptr::eq(right, right_view) {
        return unify_flexible(left_view, right_view, flexible, substitutions, span);
    }
    match (left, right) {
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

pub(super) fn resolve_substitution(
    id: TypeId,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
) -> Type {
    if !visiting.insert(id) {
        return substitutions[&id].clone();
    }
    let resolved = resolve_type(&substitutions[&id], substitutions, visiting);
    visiting.remove(&id);
    resolved
}

fn resolve_type(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
) -> Type {
    match ty {
        Type::Parameter { id, .. } if substitutions.contains_key(id) => {
            resolve_substitution(*id, substitutions, visiting)
        }
        Type::Application {
            constructor,
            argument,
            span,
            ..
        } => super::super::types::term::apply(
            resolve_type(constructor, substitutions, visiting),
            resolve_type(argument, substitutions, visiting),
            *span,
        )
        .expect("inference substitution preserves application kinds"),
        Type::Abstraction {
            parameter_kind,
            body,
        } => super::super::types::term::abstraction(
            parameter_kind.clone(),
            resolve_type(body, substitutions, visiting),
        ),
        Type::Buffer(element) => {
            Type::Buffer(resolve_type(element, substitutions, visiting).into())
        }
        Type::Opaque {
            id,
            name,
            arguments,
            representation,
            declaration_file,
        } => Type::Opaque {
            id: *id,
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| resolve_type(argument, substitutions, visiting))
                .collect::<Vec<_>>()
                .into(),
            representation: resolve_type(representation, substitutions, visiting).into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => Type::Product(
            elements
                .iter()
                .map(|element| resolve_type(element, substitutions, visiting))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Sum(members) => Type::Sum(
            members
                .iter()
                .map(|member| resolve_type(member, substitutions, visiting))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Function { parameter, result } => Type::Function {
            parameter: resolve_type(parameter, substitutions, visiting).into(),
            result: resolve_type(result, substitutions, visiting).into(),
        },
        _ => ty.clone(),
    }
}

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
            if previous == actual {
                return Ok(());
            }
            return Err(inference_conflict(previous, actual, span));
        }
        substitutions.insert(*id, actual.clone());
        return Ok(());
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
