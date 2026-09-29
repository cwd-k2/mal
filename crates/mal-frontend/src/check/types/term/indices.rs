use mal_syntax::diagnostic::Diagnostic;

use super::{Type, normalization::Budget};

pub(super) fn shift(ty: &Type, depth: usize, amount: isize) -> Type {
    if amount == 0 {
        return ty.clone();
    }
    match ty {
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
            constructor: shift(constructor, depth, amount).into(),
            argument: shift(argument, depth, amount).into(),
            kind: kind.clone(),
            span: *span,
        },
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: parameter_kind.clone(),
            body: shift(body, depth + 1, amount).into(),
        },
        _ => map_children(ty, |child| shift(child, depth, amount)),
    }
}

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

pub(super) fn contains_bound(ty: &Type, index: usize) -> bool {
    let mut pending = vec![(ty, 0)];
    while let Some((ty, depth)) = pending.pop() {
        match ty {
            Type::Bound { index: found, .. } if *found == index + depth => return true,
            Type::Application {
                constructor,
                argument,
                ..
            } => {
                pending.push((constructor, depth));
                pending.push((argument, depth));
            }
            Type::Abstraction { body, .. } => pending.push((body, depth + 1)),
            Type::Buffer(element) => pending.push((element, depth)),
            Type::Opaque {
                arguments,
                representation,
                ..
            } => {
                pending.extend(arguments.iter().map(|argument| (argument, depth)));
                pending.push((representation, depth));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().map(|element| (element, depth)));
            }
            Type::Function { parameter, result } => {
                pending.push((parameter, depth));
                pending.push((result, depth));
            }
            _ => {}
        }
    }
    false
}

fn map_children(ty: &Type, mut map: impl FnMut(&Type) -> Type) -> Type {
    match ty {
        Type::Buffer(element) => Type::Buffer(map(element).into()),
        Type::Opaque {
            id,
            name,
            arguments,
            representation,
            declaration_file,
        } => Type::Opaque {
            id: *id,
            name: name.clone(),
            arguments: arguments.iter().map(&mut map).collect::<Vec<_>>().into(),
            representation: map(representation).into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => {
            Type::Product(elements.iter().map(&mut map).collect::<Vec<_>>().into())
        }
        Type::Sum(elements) => Type::Sum(elements.iter().map(&mut map).collect::<Vec<_>>().into()),
        Type::Function { parameter, result } => Type::Function {
            parameter: map(parameter).into(),
            result: map(result).into(),
        },
        _ => ty.clone(),
    }
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
