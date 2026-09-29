use std::collections::HashMap;

use super::{Kind, Type, kind::substitute};

pub(super) fn bound(ty: &Type, argument: &Type, depth: usize) -> Type {
    match ty {
        Type::Bound { index, .. } if *index == depth => shift(argument, 0, depth as isize),
        Type::Bound { index, kind } if *index > depth => Type::Bound {
            index: index - 1,
            kind: kind.clone(),
        },
        Type::Application {
            constructor,
            argument: applied,
            span,
            ..
        } => super::apply(
            bound(constructor, argument, depth),
            bound(applied, argument, depth),
            *span,
        )
        .expect("admitted substitution preserves application kinds"),
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: parameter_kind.clone(),
            body: bound(body, argument, depth + 1).into(),
        },
        _ => map_children(ty, |child| bound(child, argument, depth)),
    }
}

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

pub(super) fn kinds(ty: &Type, substitutions: &HashMap<u32, Kind>) -> Type {
    if substitutions.is_empty() {
        return ty.clone();
    }
    match ty {
        Type::Parameter { id, name, kind } => Type::Parameter {
            id: *id,
            name: name.clone(),
            kind: substitute(kind, substitutions),
        },
        Type::Bound { index, kind } => Type::Bound {
            index: *index,
            kind: substitute(kind, substitutions),
        },
        Type::Application {
            constructor,
            argument,
            kind,
            span,
        } => Type::Application {
            constructor: kinds(constructor, substitutions).into(),
            argument: kinds(argument, substitutions).into(),
            kind: substitute(kind, substitutions),
            span: *span,
        },
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: substitute(parameter_kind, substitutions),
            body: kinds(body, substitutions).into(),
        },
        _ => map_children(ty, |child| kinds(child, substitutions)),
    }
}

pub(super) fn canonicalize_kind_variables(types: &mut [Type]) {
    let mut variables = Vec::new();
    let mut pending = types.iter().collect::<Vec<_>>();
    while let Some(ty) = pending.pop() {
        collect_kind_variables(&ty.kind(), &mut variables);
        match ty {
            Type::Application {
                constructor,
                argument,
                ..
            } => {
                pending.push(argument);
                pending.push(constructor);
            }
            Type::Abstraction { body, .. } | Type::Buffer(body) => pending.push(body),
            Type::Opaque {
                arguments,
                representation,
                ..
            } => {
                pending.push(representation);
                pending.extend(arguments.iter().rev());
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().rev());
            }
            Type::Function { parameter, result } => {
                pending.push(result);
                pending.push(parameter);
            }
            _ => {}
        }
    }
    let substitutions = variables
        .into_iter()
        .enumerate()
        .map(|(index, id)| (id, Kind::Variable(u32::MAX - index as u32)))
        .collect();
    for ty in types {
        *ty = kinds(ty, &substitutions);
    }
}

fn collect_kind_variables(kind: &Kind, variables: &mut Vec<u32>) {
    match kind {
        Kind::Type => {}
        Kind::Variable(id) => {
            if !variables.contains(id) {
                variables.push(*id);
            }
        }
        Kind::Function { parameter, result } => {
            collect_kind_variables(parameter, variables);
            collect_kind_variables(result, variables);
        }
    }
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
