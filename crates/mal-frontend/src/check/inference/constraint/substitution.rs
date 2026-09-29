//! Bounded resolution of inference substitutions into canonical type terms.

use std::collections::{HashMap, HashSet};

use crate::check::ast::Type;
use crate::check::types::term::Normalizer;
use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

pub(in crate::check::inference) fn resolve_substitution(
    id: TypeId,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
    span: Span,
) -> Result<Type, Diagnostic> {
    resolve_substitution_with(id, substitutions, visiting, &mut Normalizer::new(span))
}

fn resolve_substitution_with(
    id: TypeId,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
    normalizer: &mut Normalizer,
) -> Result<Type, Diagnostic> {
    if !visiting.insert(id) {
        return Ok(substitutions[&id].clone());
    }
    let resolved = resolve_type(&substitutions[&id], substitutions, visiting, normalizer)?;
    visiting.remove(&id);
    Ok(resolved)
}

fn resolve_type(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
    normalizer: &mut Normalizer,
) -> Result<Type, Diagnostic> {
    Ok(match ty {
        Type::Parameter { id, .. } if substitutions.contains_key(id) => {
            resolve_substitution_with(*id, substitutions, visiting, normalizer)?
        }
        Type::Application {
            constructor,
            argument,
            span,
            ..
        } => {
            let constructor = resolve_type(constructor, substitutions, visiting, normalizer)?;
            let argument = resolve_type(argument, substitutions, visiting, normalizer)?;
            normalizer.apply(constructor, argument, *span)?
        }
        Type::Abstraction {
            parameter_kind,
            body,
        } => {
            let body = resolve_type(body, substitutions, visiting, normalizer)?;
            normalizer.abstraction(parameter_kind.clone(), body)?
        }
        Type::Buffer(element) => {
            Type::Buffer(resolve_type(element, substitutions, visiting, normalizer)?.into())
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
            arguments: resolve_all(arguments, substitutions, visiting, normalizer)?.into(),
            representation: resolve_type(representation, substitutions, visiting, normalizer)?
                .into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => {
            Type::Product(resolve_all(elements, substitutions, visiting, normalizer)?.into())
        }
        Type::Sum(members) => {
            Type::Sum(resolve_all(members, substitutions, visiting, normalizer)?.into())
        }
        Type::Function { parameter, result } => Type::Function {
            parameter: resolve_type(parameter, substitutions, visiting, normalizer)?.into(),
            result: resolve_type(result, substitutions, visiting, normalizer)?.into(),
        },
        _ => ty.clone(),
    })
}

fn resolve_all(
    types: &[Type],
    substitutions: &HashMap<TypeId, Type>,
    visiting: &mut HashSet<TypeId>,
    normalizer: &mut Normalizer,
) -> Result<Vec<Type>, Diagnostic> {
    types
        .iter()
        .map(|ty| resolve_type(ty, substitutions, visiting, normalizer))
        .collect()
}
