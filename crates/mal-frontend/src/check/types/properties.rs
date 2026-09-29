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
                "`{}` is not an immutable value that a Buffer can hold",
                type_name(offending)
            ),
        ),
    )
}

/// A Buffer element is an immutable value: it holds no `Buffer`, function, or external opaque value, so element
/// storage forms no ownership cycle and no alias observes a later mutation.
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
            Type::External { .. } | Type::Function { .. } | Type::Buffer(_) | Type::Sum(_) => {
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
    satisfies_requirement(ty, available, true)
}

/// Whether `ty` has a canonical memory representation for C host copy. A type parameter never does, because
/// generic code cannot derive the layout of an opaque type.
pub(in crate::check) fn satisfies_representable_requirement(ty: &Type) -> bool {
    satisfies_requirement(ty, &[], false)
}

fn satisfies_requirement(ty: &Type, available: &[Type], symbols: bool) -> bool {
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
            Type::Symbol if symbols => {}
            _ => return false,
        }
    }
    true
}
