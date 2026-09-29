use std::collections::{HashMap, HashSet};

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use crate::check::ast::Kind;

#[derive(Clone, Debug)]
pub(super) enum InferredKind {
    Type,
    Variable(u32),
    Function(Box<InferredKind>, Box<InferredKind>),
}

#[derive(Default)]
pub(super) struct Solver {
    next: u32,
    substitutions: HashMap<u32, InferredKind>,
}

impl Solver {
    pub(super) fn fresh(&mut self) -> InferredKind {
        let id = self.next;
        self.next = self.next.checked_add(1).expect("kind identity space");
        InferredKind::Variable(id)
    }

    pub(super) fn instantiate(&mut self, kind: &Kind) -> InferredKind {
        fn visit(
            solver: &mut Solver,
            kind: &Kind,
            variables: &mut HashMap<u32, InferredKind>,
        ) -> InferredKind {
            match kind {
                Kind::Type => InferredKind::Type,
                Kind::Variable(id) => variables
                    .entry(*id)
                    .or_insert_with(|| solver.fresh())
                    .clone(),
                Kind::Function { parameter, result } => InferredKind::Function(
                    Box::new(visit(solver, parameter, variables)),
                    Box::new(visit(solver, result, variables)),
                ),
            }
        }
        visit(self, kind, &mut HashMap::new())
    }

    pub(super) fn function(parameter: InferredKind, result: InferredKind) -> InferredKind {
        InferredKind::Function(Box::new(parameter), Box::new(result))
    }

    pub(super) fn unify(
        &mut self,
        left: InferredKind,
        right: InferredKind,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let left = self.resolve(left);
        let right = self.resolve(right);
        match (left, right) {
            (InferredKind::Type, InferredKind::Type) => Ok(()),
            (InferredKind::Variable(left), InferredKind::Variable(right)) if left == right => {
                Ok(())
            }
            (InferredKind::Variable(id), kind) | (kind, InferredKind::Variable(id)) => {
                if contains(&kind, id, &self.substitutions) {
                    return Err(Diagnostic::error("infinite kind")
                        .with_primary(span, "this type application contains its own kind"));
                }
                self.substitutions.insert(id, kind);
                Ok(())
            }
            (
                InferredKind::Function(left_parameter, left_result),
                InferredKind::Function(right_parameter, right_result),
            ) => {
                self.unify(*left_parameter, *right_parameter, span)?;
                self.unify(*left_result, *right_result, span)
            }
            (left, right) => Err(Diagnostic::error("type kind mismatch").with_primary(
                span,
                format!(
                    "expected `{}` but found `{}`",
                    self.display(&left),
                    self.display(&right)
                ),
            )),
        }
    }

    pub(super) fn generalize(&self, kind: InferredKind) -> Kind {
        self.generalize_all(&[kind])
            .pop()
            .expect("one inferred kind produces one scheme")
    }

    pub(super) fn generalize_all(&self, kinds: &[InferredKind]) -> Vec<Kind> {
        fn visit(solver: &Solver, kind: InferredKind, variables: &mut HashMap<u32, u32>) -> Kind {
            match solver.resolve(kind) {
                InferredKind::Type => Kind::Type,
                InferredKind::Variable(id) => {
                    let next = variables.len() as u32;
                    Kind::Variable(*variables.entry(id).or_insert(next))
                }
                InferredKind::Function(parameter, result) => Kind::function(
                    visit(solver, *parameter, variables),
                    visit(solver, *result, variables),
                ),
            }
        }
        let mut variables = HashMap::new();
        kinds
            .iter()
            .cloned()
            .map(|kind| visit(self, kind, &mut variables))
            .collect()
    }

    fn resolve(&self, mut kind: InferredKind) -> InferredKind {
        let mut path = Vec::new();
        while let InferredKind::Variable(id) = kind {
            let Some(next) = self.substitutions.get(&id) else {
                return InferredKind::Variable(id);
            };
            path.push(id);
            kind = next.clone();
        }
        match kind {
            InferredKind::Function(parameter, result) => InferredKind::Function(
                Box::new(self.resolve(*parameter)),
                Box::new(self.resolve(*result)),
            ),
            kind => kind,
        }
    }

    fn display(&self, kind: &InferredKind) -> String {
        match self.resolve(kind.clone()) {
            InferredKind::Type => "Type".into(),
            InferredKind::Variable(id) => format!("k{id}"),
            InferredKind::Function(parameter, result) => {
                let parameter = self.resolve(*parameter);
                let parameter = match parameter {
                    InferredKind::Function(_, _) => format!("({})", self.display(&parameter)),
                    _ => self.display(&parameter),
                };
                format!("{parameter} -> {}", self.display(&result))
            }
        }
    }
}

fn contains(kind: &InferredKind, needle: u32, substitutions: &HashMap<u32, InferredKind>) -> bool {
    let mut pending = vec![kind];
    let mut visited = HashSet::new();
    while let Some(kind) = pending.pop() {
        match kind {
            InferredKind::Type => {}
            InferredKind::Variable(id) => {
                if *id == needle {
                    return true;
                }
                if visited.insert(*id)
                    && let Some(kind) = substitutions.get(id)
                {
                    pending.push(kind);
                }
            }
            InferredKind::Function(parameter, result) => {
                pending.push(parameter);
                pending.push(result);
            }
        }
    }
    false
}
