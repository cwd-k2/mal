use super::ast::Type;

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit])
}

pub(super) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Box::new(Type::Unit),
        result: Box::new(Type::Unit),
    }
}

pub(super) fn type_name(ty: &Type) -> String {
    match ty {
        Type::Unit => "Unit".into(),
        Type::Int8 => "Int8".into(),
        Type::Int16 => "Int16".into(),
        Type::Int32 => "Int32".into(),
        Type::Int64 => "Int64".into(),
        Type::UInt8 => "UInt8".into(),
        Type::UInt16 => "UInt16".into(),
        Type::UInt32 => "UInt32".into(),
        Type::UInt64 => "UInt64".into(),
        Type::Float32 => "Float32".into(),
        Type::Float64 => "Float64".into(),
        Type::String => "String".into(),
        Type::External { name, .. } => name.clone(),
        Type::Product(elements) => format!(
            "({})",
            elements
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Sum(members) if *members == vec![Type::Unit, Type::Unit] => "Bool".into(),
        Type::Sum(members) => format!(
            "[{}]",
            members.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        Type::Function { parameter, result } => {
            format!("{} -> {}", type_name(parameter), type_name(result))
        }
    }
}
