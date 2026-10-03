use std::collections::HashSet;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Type, type_name};

pub(in crate::check) fn ensure_buffer_storable(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    let Some(offending) = first_unstorable_type(ty) else {
        return Ok(());
    };
    Err(
        Diagnostic::error("buffer element type is not storable").with_primary(
            span,
            format!(
                "`{}` does not satisfy the Buffer element lifecycle contract",
                type_name(offending)
            ),
        ),
    )
}

/// A Buffer element has a lifecycle the runtime can preserve through place operations. Buffer handles recurse into
/// their element type; functions remain excluded because their hidden captures can create ownership cycles.
fn first_unstorable_type(ty: &Type) -> Option<&Type> {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Opaque { representation, .. } => pending.push(representation),
            Type::Product(elements) | Type::Sum(elements) if !elements.is_empty() => {
                pending.extend(elements.iter().rev());
            }
            Type::Buffer(element) => pending.push(element),
            Type::Function { .. } | Type::Sum(_) => {
                return Some(ty);
            }
            Type::Abstraction { .. } => return Some(ty),
            _ => {}
        }
    }
    None
}

pub(in crate::check) fn is_memory_representable(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Opaque { representation, .. } => pending.push(representation),
            Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Address
            | Type::ByteSize
            | Type::USize
            | Type::Parameter { .. }
            | Type::Bound { .. }
            | Type::Application { .. } => {}
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Symbol
            | Type::External { .. }
            | Type::Function { .. }
            | Type::Buffer(_)
            | Type::Abstraction { .. }
            | Type::Sum(_) => return false,
        }
    }
    true
}

pub(in crate::check) fn storable_requirements(ty: &Type) -> Vec<Type> {
    let mut requirements = Vec::new();
    let mut pending = vec![(ty, false)];
    while let Some((ty, required)) = pending.pop() {
        match ty {
            Type::Opaque { representation, .. } => pending.push((representation, required)),
            Type::Parameter { .. } | Type::Application { .. } if required => {
                if !requirements.iter().any(|existing| existing == ty) {
                    requirements.push(ty.clone());
                }
            }
            Type::Parameter { .. } | Type::Application { .. } | Type::Bound { .. } => {}
            Type::Abstraction { .. } => {}
            Type::Buffer(element) => {
                pending.push((element, true));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().map(|element| (element, required)));
            }
            Type::Function { parameter, result } => {
                pending.push((parameter, required));
                pending.push((result, required));
            }
            _ => {}
        }
    }
    requirements
}

/// Whether `ty` is known to be a valid Buffer element, given the type parameters that the enclosing signature
/// already requires to be storable.
pub(in crate::check) fn satisfies_storable_requirement(ty: &Type, available: &[Type]) -> bool {
    satisfies_requirement(ty, available, Requirement::Storable)
}

/// Whether `ty` has a canonical memory representation for C host copy. A type parameter never does, because
/// generic code cannot derive the layout of an opaque type.
pub(in crate::check) fn satisfies_representable_requirement(ty: &Type) -> bool {
    satisfies_requirement(ty, &[], Requirement::Representable)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Requirement {
    Storable,
    Representable,
}

fn satisfies_requirement(ty: &Type, available: &[Type], requirement: Requirement) -> bool {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        if available.iter().any(|requirement| requirement == ty) {
            continue;
        }
        match ty {
            Type::Opaque { representation, .. } => pending.push(representation),
            Type::Parameter { .. } => return false,
            Type::Bound { .. } | Type::Application { .. } | Type::Abstraction { .. } => {
                return false;
            }
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Address
            | Type::ByteSize
            | Type::USize => {}
            Type::Symbol if requirement == Requirement::Storable => {}
            Type::External { .. } if requirement == Requirement::Storable => {}
            Type::Buffer(element) if requirement == Requirement::Storable => pending.push(element),
            _ => return false,
        }
    }
    true
}
