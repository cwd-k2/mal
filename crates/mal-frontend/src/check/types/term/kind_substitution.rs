use std::collections::{HashMap, HashSet};

use mal_syntax::diagnostic::Diagnostic;

use super::{Kind, Type, kind_variables, normalization::Budget};

/// Rewrites the kinds recorded in `ty`. Variables and node shapes are kept, so nodes are rebuilt directly without the
/// canonical constructors.
pub(super) fn rewrite(
    ty: &Type,
    substitutions: &HashMap<u32, Kind>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Type, Diagnostic> {
    if substitutions.is_empty() {
        return Ok(ty.clone());
    }
    rewrite_with(
        ty,
        budget,
        depth,
        &mut kind_variables::Rewriter::new(substitutions),
    )
}

fn rewrite_with(
    ty: &Type,
    budget: &mut Budget,
    depth: usize,
    kinds: &mut kind_variables::Rewriter<'_>,
) -> Result<Type, Diagnostic> {
    budget.visit(depth)?;
    Ok(match ty {
        Type::Parameter { id, name, kind } => Type::Parameter {
            id: *id,
            name: name.clone(),
            kind: kinds.rewrite(kind),
        },
        Type::Bound { index, kind } => Type::Bound {
            index: *index,
            kind: kinds.rewrite(kind),
        },
        Type::Application {
            constructor,
            argument,
            kind,
            span,
        } => Type::Application {
            constructor: rewrite_with(constructor, budget, depth + 1, kinds)?.into(),
            argument: rewrite_with(argument, budget, depth + 1, kinds)?.into(),
            kind: kinds.rewrite(kind),
            span: *span,
        },
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: kinds.rewrite(parameter_kind),
            body: rewrite_with(body, budget, depth + 1, kinds)?.into(),
        },
        Type::Buffer(element) => {
            Type::Buffer(rewrite_with(element, budget, depth + 1, kinds)?.into())
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
            arguments: rewrite_all(arguments, budget, depth + 1, kinds)?.into(),
            representation: rewrite_with(representation, budget, depth + 1, kinds)?.into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => {
            Type::Product(rewrite_all(elements, budget, depth + 1, kinds)?.into())
        }
        Type::Sum(elements) => Type::Sum(rewrite_all(elements, budget, depth + 1, kinds)?.into()),
        Type::Function { parameter, result } => Type::Function {
            parameter: rewrite_with(parameter, budget, depth + 1, kinds)?.into(),
            result: rewrite_with(result, budget, depth + 1, kinds)?.into(),
        },
        _ => ty.clone(),
    })
}

fn rewrite_all(
    types: &[Type],
    budget: &mut Budget,
    depth: usize,
    kinds: &mut kind_variables::Rewriter<'_>,
) -> Result<Vec<Type>, Diagnostic> {
    let mut rewritten = Vec::new();
    for ty in types {
        rewritten.push(rewrite_with(ty, budget, depth, kinds)?);
    }
    Ok(rewritten)
}

pub(super) fn canonicalize_variables(
    types: &mut [Type],
    rigid: &HashSet<u32>,
    budget: &mut Budget,
) -> Result<(), Diagnostic> {
    let substitutions = kind_variables::collect(types.iter())
        .into_iter()
        .filter(|id| !rigid.contains(id))
        .enumerate()
        .filter_map(|(index, id)| {
            let canonical = u32::MAX - index as u32;
            (id != canonical).then_some((id, Kind::Variable(canonical)))
        })
        .collect();
    for ty in types {
        *ty = rewrite(ty, &substitutions, budget, 0)?;
    }
    Ok(())
}

pub(super) fn freshen_variables(
    ty: &Type,
    next: &mut u32,
    budget: &mut Budget,
) -> Result<Type, Diagnostic> {
    let substitutions = kind_variables::collect(std::iter::once(ty))
        .into_iter()
        .filter_map(|id| {
            let fresh = *next;
            *next = next.checked_add(1).expect("kind identity space");
            (id != fresh).then_some((id, Kind::Variable(fresh)))
        })
        .collect::<HashMap<_, _>>();
    if substitutions.is_empty() {
        return Ok(ty.clone());
    }
    let mut kinds = kind_variables::Rewriter::new(&substitutions);
    rewrite_with(ty, budget, 0, &mut kinds)
}
