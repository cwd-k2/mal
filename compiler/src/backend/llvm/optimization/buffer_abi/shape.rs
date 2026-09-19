use std::collections::HashSet;

use crate::check::ast::Type;

pub(super) fn buffer_parameter_is_supported(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut buffers = 0usize;
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Buffer(_) => buffers += 1,
            Type::Product(elements) => pending.extend(elements.iter()),
            _ if contains_buffer(ty) => return false,
            _ => {}
        }
    }
    buffers == 1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_exactly_one_buffer_leaf_in_a_product() {
        let buffer = Type::Buffer(Type::Int32.into());
        assert!(buffer_parameter_is_supported(&Type::Product(
            vec![Type::USize, buffer.clone()].into()
        )));
        assert!(!buffer_parameter_is_supported(&Type::Product(
            vec![buffer.clone(), buffer].into()
        )));
    }
}
