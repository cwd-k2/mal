//! Implementation key patterns: where parameters occur, and which constructor heads are nominal.

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

/// Whether a constructor term is an opaque type or `Buffer` partially applied. Such a head is matched by
/// identity, so the variables among its arguments are found by first-order matching.
pub(super) fn has_nominal_head(constructor: &Type) -> bool {
    let mut body = constructor;
    while let Type::Abstraction { body: inner, .. } = body {
        body = inner;
    }
    matches!(body, Type::Opaque { .. } | Type::Buffer(_))
}
