use std::collections::HashSet;

use crate::check::ast::Type;

pub(super) fn buffer_parameter_is_supported(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut found = false;
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Buffer(_) => found = true,
            Type::Product(elements) => pending.extend(elements.iter()),
            _ if contains_buffer(ty) => return false,
            _ => {}
        }
    }
    found
}

pub(crate) fn contains_buffer(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut seen = HashSet::new();
    while let Some(ty) = pending.pop() {
        if let Some(identity) = ty.shared_id()
            && !seen.insert(identity)
        {
            continue;
        }
        match ty {
            Type::Buffer(_) => return true,
            Type::Cursor(element) | Type::Region(element) | Type::Packed(element) => {
                pending.push(element)
            }
            Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
            Type::Function { parameter, result } => {
                pending.extend([parameter.as_ref(), result.as_ref()]);
            }
            _ => {}
        }
    }
    false
}
