//! Canonical substitution, runtime erasure, and file-local type equivalence.

use mal_syntax::source::FileId;

use crate::resolve::ast::TypeId;

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

/// The first layer below `ty`, viewing only opaque layers declared in `file`, that is the opaque declaration `id`.
pub(in crate::check) fn layer_with_identity(ty: &Type, id: TypeId, file: FileId) -> Option<&Type> {
    same_file_layers(ty, file)
        .find(|layer| matches!(layer, Type::Opaque { id: layer_id, .. } if *layer_id == id))
}

/// Whether `left` and `right` are the same type once the file-local opaque types declared in `file` may be viewed as
/// their representations. At each position only one side is viewed, through as many layers declared in `file` as
/// needed: an opaque type equals its representation, but two opaque types with the same representation stay apart.
pub(in crate::check) fn equivalent_in_file(left: &Type, right: &Type, file: FileId) -> bool {
    left == right
        || same_file_layers(left, file).any(|view| same_structure(view, right, file))
        || same_file_layers(right, file).any(|view| same_structure(left, view, file))
        || same_structure(left, right, file)
}

/// The representations reached by viewing `ty` through one or more opaque layers declared in `file`.
fn same_file_layers(ty: &Type, file: FileId) -> impl Iterator<Item = &Type> {
    std::iter::successors(Some(ty), move |current| match current {
        Type::Opaque {
            representation,
            declaration_file,
            ..
        } if *declaration_file == file => Some(representation.as_ref()),
        _ => None,
    })
    .skip(1)
}

/// Equality of the outermost constructor without viewing either side, with views allowed again below it.
fn same_structure(left: &Type, right: &Type, file: FileId) -> bool {
    if left == right {
        return true;
    }
    let children: Vec<(&Type, &Type)> = match (left, right) {
        // Uses of one parameter may record kinds instantiated at different sites.
        (Type::Parameter { id: left, .. }, Type::Parameter { id: right, .. }) => {
            return left == right;
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
        ) => vec![
            (left_constructor, right_constructor),
            (left_argument, right_argument),
        ],
        (Type::Buffer(left), Type::Buffer(right)) => vec![(left, right)],
        // The identity is the declaration and its arguments; a phantom argument's parameter uses may
        // still carry kinds instantiated at different sites.
        (
            Type::Opaque {
                id: left_id,
                arguments: left_arguments,
                ..
            },
            Type::Opaque {
                id: right_id,
                arguments: right_arguments,
                ..
            },
        ) if left_id == right_id && left_arguments.len() == right_arguments.len() => {
            left_arguments.iter().zip(right_arguments.iter()).collect()
        }
        (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
            if left.len() == right.len() =>
        {
            left.iter().zip(right.iter()).collect()
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
        ) => vec![
            (left_parameter, right_parameter),
            (left_result, right_result),
        ],
        _ => return false,
    };
    children
        .into_iter()
        .all(|(left, right)| equivalent_in_file(left, right, file))
}

pub(in crate::check) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
    }
}
