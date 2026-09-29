use std::collections::HashMap;

use mal_syntax::diagnostic::Diagnostic;

use super::{Kind, Type, kind::substitute, normalization::Budget};

pub(super) fn rewrite(
    ty: &Type,
    substitutions: &HashMap<u32, Kind>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Type, Diagnostic> {
    if substitutions.is_empty() {
        return Ok(ty.clone());
    }
    budget.visit(depth)?;
    Ok(match ty {
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
            constructor: rewrite(constructor, substitutions, budget, depth + 1)?.into(),
            argument: rewrite(argument, substitutions, budget, depth + 1)?.into(),
            kind: substitute(kind, substitutions),
            span: *span,
        },
        Type::Abstraction {
            parameter_kind,
            body,
        } => Type::Abstraction {
            parameter_kind: substitute(parameter_kind, substitutions),
            body: rewrite(body, substitutions, budget, depth + 1)?.into(),
        },
        Type::Buffer(element) => {
            Type::Buffer(rewrite(element, substitutions, budget, depth + 1)?.into())
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
            arguments: rewrite_all(arguments, substitutions, budget, depth + 1)?.into(),
            representation: rewrite(representation, substitutions, budget, depth + 1)?.into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => {
            Type::Product(rewrite_all(elements, substitutions, budget, depth + 1)?.into())
        }
        Type::Sum(elements) => {
            Type::Sum(rewrite_all(elements, substitutions, budget, depth + 1)?.into())
        }
        Type::Function { parameter, result } => Type::Function {
            parameter: rewrite(parameter, substitutions, budget, depth + 1)?.into(),
            result: rewrite(result, substitutions, budget, depth + 1)?.into(),
        },
        _ => ty.clone(),
    })
}

fn rewrite_all(
    types: &[Type],
    substitutions: &HashMap<u32, Kind>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Vec<Type>, Diagnostic> {
    let mut rewritten = Vec::new();
    for ty in types {
        rewritten.push(rewrite(ty, substitutions, budget, depth)?);
    }
    Ok(rewritten)
}

pub(super) fn canonicalize_variables(
    types: &mut [Type],
    budget: &mut Budget,
) -> Result<(), Diagnostic> {
    let mut variables = Vec::new();
    let mut pending = types.iter().collect::<Vec<_>>();
    while let Some(ty) = pending.pop() {
        collect_variables(&ty.kind(), &mut variables);
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
        *ty = rewrite(ty, &substitutions, budget, 0)?;
    }
    Ok(())
}

fn collect_variables(kind: &Kind, variables: &mut Vec<u32>) {
    match kind {
        Kind::Type => {}
        Kind::Variable(id) => {
            if !variables.contains(id) {
                variables.push(*id);
            }
        }
        Kind::Function { parameter, result } => {
            collect_variables(parameter, variables);
            collect_variables(result, variables);
        }
    }
}
