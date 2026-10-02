//! Operations over canonical terms that later stages read: substitution of type parameters, erasure to runtime
//! types, and file-local views of opaque types.

use super::super::ast::Type;

mod opaque_view;
mod substitution;

pub(in crate::check) use opaque_view::{
    equivalent_in_file, layer_with_identity, representation_view,
};
pub(in crate::check) use substitution::{rigid_parameters, substitute_type};

pub(in crate::check) fn runtime_type(ty: &Type) -> Type {
    match ty {
        Type::Application { .. } | Type::Abstraction { .. } | Type::Bound { .. } => {
            unreachable!("specialization removes type-level terms from runtime types")
        }
        Type::Opaque { representation, .. } => runtime_type(representation),
        Type::Buffer(element) => Type::Buffer(runtime_type(element).into()),
        Type::Product(elements) => {
            Type::Product(elements.iter().map(runtime_type).collect::<Vec<_>>().into())
        }
        Type::Sum(members) => {
            Type::Sum(members.iter().map(runtime_type).collect::<Vec<_>>().into())
        }
        Type::Function { parameter, result } => Type::Function {
            parameter: runtime_type(parameter).into(),
            result: runtime_type(result).into(),
        },
        _ => ty.clone(),
    }
}

pub(in crate::check) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit].into())
}

pub(in crate::check) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
    }
}
