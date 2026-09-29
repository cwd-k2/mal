use mal_syntax::diagnostic::Diagnostic;

use super::{Type, normalization::Budget};

pub(super) fn shift_bounded(
    ty: &Type,
    depth: usize,
    amount: isize,
    budget: &mut Budget,
    traversal_depth: usize,
) -> Result<Type, Diagnostic> {
    if amount == 0 {
        return Ok(ty.clone());
    }
    budget.visit(traversal_depth)?;
    Ok(match ty {
        Type::Bound { index, kind } if *index >= depth => Type::Bound {
            index: index.saturating_add_signed(amount),
            kind: kind.clone(),
        },
        Type::Application {
            constructor,
            argument,
            kind,
            span,
        } => Type::Application {
            constructor: shift_bounded(constructor, depth, amount, budget, traversal_depth + 1)?
                .into(),
            argument: shift_bounded(argument, depth, amount, budget, traversal_depth + 1)?.into(),
            kind: kind.clone(),
            span: *span,
        },
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: parameter_kind.clone(),
            body: shift_bounded(body, depth + 1, amount, budget, traversal_depth + 1)?.into(),
        },
        _ => map_children_bounded(ty, |child| {
            shift_bounded(child, depth, amount, budget, traversal_depth + 1)
        })?,
    })
}

pub(super) fn contains_bound_bounded(
    ty: &Type,
    index: usize,
    budget: &mut Budget,
    traversal_depth: usize,
) -> Result<bool, Diagnostic> {
    let mut pending = vec![(ty, 0, traversal_depth)];
    while let Some((ty, binder_depth, traversal_depth)) = pending.pop() {
        budget.visit(traversal_depth)?;
        match ty {
            Type::Bound { index: found, .. } if *found == index + binder_depth => return Ok(true),
            Type::Application {
                constructor,
                argument,
                ..
            } => {
                pending.push((constructor, binder_depth, traversal_depth + 1));
                pending.push((argument, binder_depth, traversal_depth + 1));
            }
            Type::Abstraction { body, .. } => {
                pending.push((body, binder_depth + 1, traversal_depth + 1));
            }
            Type::Buffer(element) => {
                pending.push((element, binder_depth, traversal_depth + 1));
            }
            Type::Opaque {
                arguments,
                representation,
                ..
            } => {
                pending.extend(
                    arguments
                        .iter()
                        .map(|argument| (argument, binder_depth, traversal_depth + 1)),
                );
                pending.push((representation, binder_depth, traversal_depth + 1));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(
                    elements
                        .iter()
                        .map(|element| (element, binder_depth, traversal_depth + 1)),
                );
            }
            Type::Function { parameter, result } => {
                pending.push((parameter, binder_depth, traversal_depth + 1));
                pending.push((result, binder_depth, traversal_depth + 1));
            }
            _ => {}
        }
    }
    Ok(false)
}

pub(super) fn map_children_bounded(
    ty: &Type,
    mut map: impl FnMut(&Type) -> Result<Type, Diagnostic>,
) -> Result<Type, Diagnostic> {
    Ok(match ty {
        Type::Buffer(element) => Type::Buffer(map(element)?.into()),
        Type::Opaque {
            id,
            name,
            arguments,
            representation,
            declaration_file,
        } => Type::Opaque {
            id: *id,
            name: name.clone(),
            arguments: map_all(arguments, &mut map)?.into(),
            representation: map(representation)?.into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => Type::Product(map_all(elements, &mut map)?.into()),
        Type::Sum(elements) => Type::Sum(map_all(elements, &mut map)?.into()),
        Type::Function { parameter, result } => Type::Function {
            parameter: map(parameter)?.into(),
            result: map(result)?.into(),
        },
        _ => ty.clone(),
    })
}

fn map_all(
    types: &[Type],
    map: &mut impl FnMut(&Type) -> Result<Type, Diagnostic>,
) -> Result<Vec<Type>, Diagnostic> {
    let mut mapped = Vec::new();
    for ty in types {
        mapped.push(map(ty)?);
    }
    Ok(mapped)
}
