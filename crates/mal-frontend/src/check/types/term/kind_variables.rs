//! Collection and shared rewriting of kind variables embedded in canonical terms.

use std::collections::{HashMap, HashSet};

use super::{Kind, Type};

pub(super) fn collect<'a>(types: impl IntoIterator<Item = &'a Type>) -> Vec<u32> {
    let mut variables = Vec::new();
    let mut visited = HashSet::new();
    let mut pending = types.into_iter().collect::<Vec<_>>();
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Parameter { kind, .. } | Type::Bound { kind, .. } => {
                collect_kind(kind, &mut variables, &mut visited);
            }
            Type::Application {
                constructor,
                argument,
                kind,
                ..
            } => {
                collect_kind(kind, &mut variables, &mut visited);
                pending.push(argument);
                pending.push(constructor);
            }
            Type::Abstraction {
                parameter_kind,
                body,
            } => {
                collect_kind(parameter_kind, &mut variables, &mut visited);
                pending.push(body);
            }
            Type::Buffer(body) => pending.push(body),
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
    variables
}

fn collect_kind(kind: &Kind, variables: &mut Vec<u32>, visited: &mut HashSet<KindIdentity>) {
    if !visited.insert(KindIdentity::of(kind)) {
        return;
    }
    match kind {
        Kind::Type => {}
        Kind::Variable(id) => variables.push(*id),
        Kind::Function { parameter, result } => {
            collect_kind(parameter, variables, visited);
            collect_kind(result, variables, visited);
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum KindIdentity {
    Type,
    Variable(u32),
    Function(*const Kind, *const Kind),
}

impl KindIdentity {
    fn of(kind: &Kind) -> Self {
        match kind {
            Kind::Type => Self::Type,
            Kind::Variable(id) => Self::Variable(*id),
            Kind::Function { parameter, result } => Self::Function(
                std::sync::Arc::as_ptr(parameter),
                std::sync::Arc::as_ptr(result),
            ),
        }
    }
}

pub(super) struct Rewriter<'a> {
    substitutions: &'a HashMap<u32, Kind>,
    rewritten: HashMap<KindIdentity, Kind>,
    functions: HashMap<(KindIdentity, KindIdentity), Kind>,
}

impl<'a> Rewriter<'a> {
    pub(super) fn new(substitutions: &'a HashMap<u32, Kind>) -> Self {
        Self {
            substitutions,
            rewritten: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub(super) fn rewrite(&mut self, kind: &Kind) -> Kind {
        let identity = KindIdentity::of(kind);
        if let Some(rewritten) = self.rewritten.get(&identity) {
            return rewritten.clone();
        }
        let rewritten = match kind {
            Kind::Type => Kind::Type,
            Kind::Variable(id) => match self.substitutions.get(id) {
                Some(substitution) => self.rewrite(substitution),
                None => kind.clone(),
            },
            Kind::Function { parameter, result } => {
                let parameter = self.rewrite(parameter);
                let result = self.rewrite(result);
                let key = (KindIdentity::of(&parameter), KindIdentity::of(&result));
                self.functions.get(&key).cloned().unwrap_or_else(|| {
                    let function = Kind::function(parameter, result);
                    self.functions.insert(key, function.clone());
                    function
                })
            }
        };
        self.rewritten.insert(identity, rewritten.clone());
        rewritten
    }
}
