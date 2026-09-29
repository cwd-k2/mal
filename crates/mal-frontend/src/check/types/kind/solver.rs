use std::collections::{HashMap, HashSet};

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use crate::check::ast::Kind;

use super::admission::Budget;

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
        let left = self.resolve_head(left);
        let right = self.resolve_head(right);
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

    pub(super) fn generalize(&self, kind: InferredKind, span: Span) -> Result<Kind, Diagnostic> {
        Ok(self
            .generalize_all(&[kind], span)?
            .pop()
            .expect("one inferred kind produces one scheme"))
    }

    pub(super) fn generalize_all(
        &self,
        kinds: &[InferredKind],
        span: Span,
    ) -> Result<Vec<Kind>, Diagnostic> {
        fn visit(
            solver: &Solver,
            kind: &InferredKind,
            function_depth: usize,
            budget: &mut Budget,
            variables: &mut HashMap<u32, u32>,
        ) -> Result<Kind, Diagnostic> {
            budget.enter(function_depth)?;
            match kind {
                InferredKind::Type => Ok(Kind::Type),
                InferredKind::Variable(id) => {
                    if let Some(kind) = solver.substitutions.get(id) {
                        return visit(solver, kind, function_depth, budget, variables);
                    }
                    let next = variables.len() as u32;
                    Ok(Kind::Variable(*variables.entry(*id).or_insert(next)))
                }
                InferredKind::Function(parameter, result) => Ok(Kind::function(
                    visit(solver, parameter, function_depth + 1, budget, variables)?,
                    visit(solver, result, function_depth + 1, budget, variables)?,
                )),
            }
        }
        let mut variables = HashMap::new();
        let mut budget = Budget::new(span);
        kinds
            .iter()
            .map(|kind| visit(self, kind, 0, &mut budget, &mut variables))
            .collect()
    }

    fn resolve_head(&self, mut kind: InferredKind) -> InferredKind {
        while let InferredKind::Variable(id) = kind {
            let Some(next) = self.substitutions.get(&id) else {
                return InferredKind::Variable(id);
            };
            kind = next.clone();
        }
        kind
    }

    fn display(&self, kind: &InferredKind) -> String {
        match kind {
            InferredKind::Type => "Type".into(),
            InferredKind::Variable(id) => self
                .substitutions
                .get(id)
                .map_or_else(|| format!("k{id}"), |kind| self.display(kind)),
            InferredKind::Function(parameter, result) => {
                let parameter = match self.resolves_to_function(parameter) {
                    true => format!("({})", self.display(parameter)),
                    false => self.display(parameter),
                };
                format!("{parameter} -> {}", self.display(result))
            }
        }
    }

    fn resolves_to_function(&self, kind: &InferredKind) -> bool {
        match kind {
            InferredKind::Function(_, _) => true,
            InferredKind::Variable(id) => self
                .substitutions
                .get(id)
                .is_some_and(|kind| self.resolves_to_function(kind)),
            InferredKind::Type => false,
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
