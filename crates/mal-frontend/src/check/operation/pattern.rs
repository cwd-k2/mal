//! Implementation key patterns: parameter occurrence, nominal constructor heads, and the structural decrease
//! that requirement keys of a generic implementation must show.

use crate::resolve::ast::TypeId;

use super::super::ast::Type;

pub(in crate::check) fn contains_parameter(ty: &Type) -> bool {
    contains_parameter_id_if(ty, |_| true)
}

pub(super) fn contains_parameter_id(ty: &Type, expected: TypeId) -> bool {
    contains_parameter_id_if(ty, |id| id == expected)
}

fn contains_parameter_id_if(ty: &Type, predicate: impl Copy + Fn(TypeId) -> bool) -> bool {
    match ty {
        Type::Parameter { id, .. } => predicate(*id),
        Type::Application {
            constructor,
            argument,
            ..
        } => {
            contains_parameter_id_if(constructor, predicate)
                || contains_parameter_id_if(argument, predicate)
        }
        Type::Abstraction { body, .. } => contains_parameter_id_if(body, predicate),
        Type::Buffer(element) => contains_parameter_id_if(element, predicate),
        Type::Opaque { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_parameter_id_if(argument, predicate)),
        Type::Product(elements) | Type::Sum(elements) => elements
            .iter()
            .any(|element| contains_parameter_id_if(element, predicate)),
        Type::Function { parameter, result } => {
            contains_parameter_id_if(parameter, predicate)
                || contains_parameter_id_if(result, predicate)
        }
        _ => false,
    }
}

pub(super) fn operation_requirement_decreases(pattern: &[Type], requirement: &[Type]) -> bool {
    requirement.iter().all(|required| {
        pattern
            .iter()
            .any(|root| proper_type_subterm(root, required))
    })
}

fn proper_type_subterm(root: &Type, required: &Type) -> bool {
    let mut pending = Vec::new();
    extend_children(root, &mut pending);
    while let Some(candidate) = pending.pop() {
        if candidate == required {
            return true;
        }
        extend_children(candidate, &mut pending);
    }
    false
}

fn extend_children<'a>(ty: &'a Type, pending: &mut Vec<&'a Type>) {
    match ty {
        Type::Abstraction { body, .. } => pending.push(body),
        Type::Buffer(element) => pending.push(element),
        Type::Opaque { arguments, .. } => pending.extend(arguments.iter()),
        Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
        Type::Function { parameter, result } => {
            pending.push(parameter);
            pending.push(result);
        }
        _ => {}
    }
}

/// Whether a constructor term is an opaque type or `Buffer` partially applied. Such a head is matched by
/// identity, so the variables among its arguments are found by first-order matching.
pub(super) fn has_nominal_head(constructor: &Type) -> bool {
    let mut body = constructor;
    while let Type::Abstraction { body: inner, .. } = body {
        body = inner;
    }
    matches!(body, Type::Opaque { .. } | Type::Buffer(_))
}
