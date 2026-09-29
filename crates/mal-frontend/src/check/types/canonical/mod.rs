//! Canonical substitution, runtime erasure, and file-local type equivalence.

use mal_syntax::source::FileId;

use super::super::ast::Type;

mod substitution;

pub(in crate::check) use substitution::substitute_type;

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

pub(in crate::check) fn representation_view(ty: &Type, file: FileId) -> &Type {
    let mut current = ty;
    while let Type::Opaque {
        representation,
        declaration_file,
        ..
    } = current
    {
        if *declaration_file != file {
            break;
        }
        current = representation;
    }
    current
}

pub(in crate::check) fn equivalent_in_file(left: &Type, right: &Type, file: FileId) -> bool {
    let mut pending = vec![(left, right)];
    while let Some((left, right)) = pending.pop() {
        if left == right {
            continue;
        }
        if matches!((left, right), (Type::Opaque { .. }, Type::Opaque { .. })) {
            return false;
        }
        let left_view = representation_view(left, file);
        let right_view = representation_view(right, file);
        if !std::ptr::eq(left, left_view) || !std::ptr::eq(right, right_view) {
            pending.push((left_view, right_view));
            continue;
        }
        match (left, right) {
            (Type::Parameter { id: left, .. }, Type::Parameter { id: right, .. })
                if left == right =>
            {
                continue;
            }
            (
                Type::Application {
                    constructor: left_constructor,
                    argument: left_argument,
                    ..
                },
                Type::Application {
                    constructor: right_constructor,
                    argument: right_argument,
                    ..
                },
            ) => {
                pending.push((left_constructor, right_constructor));
                pending.push((left_argument, right_argument));
            }
            (Type::Buffer(left), Type::Buffer(right)) => pending.push((left, right)),
            (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
                if left.len() == right.len() =>
            {
                pending.extend(left.iter().zip(right.iter()));
            }
            (
                Type::Function {
                    parameter: left_parameter,
                    result: left_result,
                },
                Type::Function {
                    parameter: right_parameter,
                    result: right_result,
                },
            ) => {
                pending.push((left_parameter, right_parameter));
                pending.push((left_result, right_result));
            }
            _ => return false,
        }
    }
    true
}

pub(in crate::check) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
    }
}
