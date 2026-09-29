use std::collections::{HashMap, HashSet};

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::Kind;

pub(super) fn unify(
    expected: &Kind,
    actual: &Kind,
    substitutions: &mut HashMap<u32, Kind>,
    span: Span,
) -> Result<(), Diagnostic> {
    match (
        resolve(expected, substitutions),
        resolve(actual, substitutions),
    ) {
        (Kind::Variable(left), Kind::Variable(right)) if left == right => Ok(()),
        (Kind::Variable(id), kind) | (kind, Kind::Variable(id)) => {
            if contains(&kind, id, substitutions) {
                return Err(Diagnostic::error("infinite kind")
                    .with_primary(span, "this type application contains its own kind"));
            }
            substitutions.insert(id, kind);
            Ok(())
        }
        (Kind::Type, Kind::Type) => Ok(()),
        (
            Kind::Function {
                parameter: left_parameter,
                result: left_result,
            },
            Kind::Function {
                parameter: right_parameter,
                result: right_result,
            },
        ) => {
            unify(&left_parameter, &right_parameter, substitutions, span)?;
            unify(&left_result, &right_result, substitutions, span)
        }
        (expected, actual) => Err(Diagnostic::error("type kind mismatch").with_primary(
            span,
            format!(
                "expected `{}` but found `{}`",
                name(&expected),
                name(&actual)
            ),
        )),
    }
}

pub(super) fn substitute(kind: &Kind, substitutions: &HashMap<u32, Kind>) -> Kind {
    match resolve(kind, substitutions) {
        Kind::Function { parameter, result } => Kind::function(
            substitute(&parameter, substitutions),
            substitute(&result, substitutions),
        ),
        kind => kind,
    }
}

fn resolve(kind: &Kind, substitutions: &HashMap<u32, Kind>) -> Kind {
    let mut current = kind.clone();
    let mut visited = HashSet::new();
    while let Kind::Variable(id) = current {
        if !visited.insert(id) {
            break;
        }
        let Some(next) = substitutions.get(&id) else {
            break;
        };
        current = next.clone();
    }
    current
}

fn contains(kind: &Kind, needle: u32, substitutions: &HashMap<u32, Kind>) -> bool {
    let mut pending = vec![kind.clone()];
    while let Some(kind) = pending.pop() {
        match resolve(&kind, substitutions) {
            Kind::Type => {}
            Kind::Variable(id) if id == needle => return true,
            Kind::Variable(_) => {}
            Kind::Function { parameter, result } => {
                pending.push((*parameter).clone());
                pending.push((*result).clone());
            }
        }
    }
    false
}

pub(in crate::check::types) fn name(kind: &Kind) -> String {
    match kind {
        Kind::Type => "Type".into(),
        Kind::Variable(id) => format!("k{id}"),
        Kind::Function { parameter, result } => {
            let parameter = match parameter.as_ref() {
                Kind::Function { .. } => format!("({})", name(parameter)),
                kind => name(kind),
            };
            format!("{parameter} -> {}", name(result))
        }
    }
}
