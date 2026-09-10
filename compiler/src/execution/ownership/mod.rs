use crate::check::ast::Type;

pub(crate) fn is_managed(ty: &Type) -> bool {
    match ty {
        Type::Symbol | Type::Function { .. } => true,
        Type::Product(elements) | Type::Sum(elements) => elements.iter().any(is_managed),
        Type::External { .. }
        | Type::Unit
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
        | Type::Ptr => false,
    }
}
