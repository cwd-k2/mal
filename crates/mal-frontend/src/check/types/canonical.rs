//! Canonical substitution, runtime erasure, and file-local type equivalence.

use std::collections::HashMap;

use crate::resolve::ast::TypeId;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::FileId;
use mal_syntax::source::Span;

use super::super::ast::Type;
use super::term;

pub(in crate::check) fn substitute_type(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    span: Span,
) -> Result<Type, Diagnostic> {
    substitute(ty, substitutions, &mut term::Normalizer::new(span))
}

fn substitute(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    normalizer: &mut term::Normalizer,
) -> Result<Type, Diagnostic> {
    Ok(match ty {
        Type::Parameter { id, .. } => substitutions.get(id).cloned().unwrap_or_else(|| ty.clone()),
        Type::Application {
            constructor,
            argument,
            span,
            ..
        } => {
            let constructor = substitute(constructor, substitutions, normalizer)?;
            let argument = substitute(argument, substitutions, normalizer)?;
            normalizer.apply(constructor, argument, *span)?
        }
        Type::Abstraction {
            parameter_kind,
            body,
        } => {
            let body = substitute(body, substitutions, normalizer)?;
            normalizer.abstraction(parameter_kind.clone(), body)?
        }
        Type::Buffer(element) => {
            Type::Buffer(substitute(element, substitutions, normalizer)?.into())
        }
        Type::Opaque {
            id,
            name,
            arguments,
            representation,
            declaration_file,
        } => Type::Opaque {
            id: *id,
            name: name.clone(),
            arguments: substitute_all(arguments, substitutions, normalizer)?.into(),
            representation: substitute(representation, substitutions, normalizer)?.into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => {
            Type::Product(substitute_all(elements, substitutions, normalizer)?.into())
        }
        Type::Sum(members) => Type::Sum(substitute_all(members, substitutions, normalizer)?.into()),
        Type::Function { parameter, result } => Type::Function {
            parameter: substitute(parameter, substitutions, normalizer)?.into(),
            result: substitute(result, substitutions, normalizer)?.into(),
        },
        _ => ty.clone(),
    })
}

fn substitute_all(
    types: &[Type],
    substitutions: &HashMap<TypeId, Type>,
    normalizer: &mut term::Normalizer,
) -> Result<Vec<Type>, Diagnostic> {
    types
        .iter()
        .map(|ty| substitute(ty, substitutions, normalizer))
        .collect()
}

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
