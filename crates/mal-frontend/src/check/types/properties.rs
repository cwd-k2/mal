use std::collections::HashSet;

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::{Type, type_name};

pub(in crate::check) fn ensure_buffer_storable(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    let Err(offending) = normalized_storable_requirements(ty) else {
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

/// Reduces the closed `Storable` judgment to the open parameter or application atoms that a caller must provide.
/// Buffer formation admits those atoms; a generic use proves them against its signature requirements.
fn normalized_storable_requirements(ty: &Type) -> Result<Vec<Type>, &Type> {
    let mut requirements = Vec::new();
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Opaque { representation, .. } => pending.push(representation),
            Type::Parameter { .. } | Type::Bound { .. } | Type::Application { .. } => {
                if !requirements.iter().any(|existing| existing == ty) {
                    requirements.push(ty.clone());
                }
            }
            Type::Product(elements) | Type::Sum(elements) if !elements.is_empty() => {
                pending.extend(elements.iter().rev());
            }
            Type::Buffer(element) => pending.push(element),
            Type::Function { .. } | Type::Sum(_) => {
                return Err(ty);
            }
            Type::Abstraction { .. } => return Err(ty),
            _ => {}
        }
    }
    Ok(requirements)
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
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Opaque { representation, .. } => pending.push(representation),
            Type::Buffer(element) => {
                let element_requirements = normalized_storable_requirements(element)
                    .expect("Buffer formation rejects a non-storable element");
                for requirement in element_requirements {
                    if !requirements.iter().any(|existing| existing == &requirement) {
                        requirements.push(requirement);
                    }
                }
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter());
            }
            Type::Function { parameter, result } => {
                pending.push(parameter);
                pending.push(result);
            }
            _ => {}
        }
    }
    requirements
}

/// Whether `ty` is known to be a valid Buffer element, given the type parameters that the enclosing signature
/// already requires to be storable.
pub(in crate::check) fn satisfies_storable_requirement(ty: &Type, available: &[Type]) -> bool {
    normalized_storable_requirements(ty).is_ok_and(|requirements| {
        requirements
            .iter()
            .all(|required| available.iter().any(|candidate| candidate == required))
    })
}

/// Whether `ty` has a canonical memory representation for C host copy. A type parameter never does, because
/// generic code cannot derive the layout of an opaque type.
pub(in crate::check) fn satisfies_representable_requirement(ty: &Type) -> bool {
    satisfies_representable(ty)
}

fn satisfies_representable(ty: &Type) -> bool {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
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
            _ => return false,
        }
    }
    true
}
