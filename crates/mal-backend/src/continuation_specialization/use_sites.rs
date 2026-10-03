//! Classification of closure-source uses in the control program.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, FunctionId, Pattern};
use crate::control::ast::{self as control, Terminator};

use super::plan::{ClosureUse, ClosureUseKind, Scope};
use super::provenance::{Analysis, source_key};

pub(super) fn collect(
    control: &control::Program,
    analysis: &Analysis<'_>,
    relevant: &HashSet<FunctionId>,
) -> Vec<ClosureUse> {
    let mut uses = Vec::new();
    for (index, state) in control.states.iter().enumerate() {
        let Some(scope) = analysis.scope(index) else {
            continue;
        };
        for binding in &state.bindings {
            match &binding.operation {
                control::Operation::Atom(atom) => {
                    record(
                        analysis,
                        relevant,
                        scope,
                        atom,
                        ClosureUseKind::Alias,
                        &mut uses,
                    );
                }
                control::Operation::Product(atoms) => {
                    let product = pattern_binding(&binding.pattern);
                    for atom in atoms {
                        record(
                            analysis,
                            relevant,
                            scope,
                            atom,
                            ClosureUseKind::Product(product),
                            &mut uses,
                        );
                    }
                }
                control::Operation::SumInjection { value, .. } => record(
                    analysis,
                    relevant,
                    scope,
                    value,
                    ClosureUseKind::Aggregate,
                    &mut uses,
                ),
                control::Operation::MakeClosure { function, captures } => {
                    let creator = pattern_binding(&binding.pattern);
                    for capture in captures {
                        record(
                            analysis,
                            relevant,
                            scope,
                            capture,
                            ClosureUseKind::Capture {
                                binding: creator,
                                function: *function,
                            },
                            &mut uses,
                        );
                    }
                }
                operation => operation.for_each_atom(|atom| {
                    record(
                        analysis,
                        relevant,
                        scope,
                        atom,
                        ClosureUseKind::Escape,
                        &mut uses,
                    );
                }),
            }
        }
        match &state.terminator {
            Terminator::Return(atom) => record(
                analysis,
                relevant,
                scope,
                atom,
                ClosureUseKind::Return,
                &mut uses,
            ),
            Terminator::Jump { value, .. } => record(
                analysis,
                relevant,
                scope,
                value,
                ClosureUseKind::Join,
                &mut uses,
            ),
            Terminator::Call {
                callee, argument, ..
            }
            | Terminator::TailCall { callee, argument } => {
                record(
                    analysis,
                    relevant,
                    scope,
                    callee,
                    ClosureUseKind::Callee(callee.id),
                    &mut uses,
                );
                record(
                    analysis,
                    relevant,
                    scope,
                    argument,
                    ClosureUseKind::CallArgument(callee.id),
                    &mut uses,
                );
            }
            Terminator::Case { scrutinee, .. } => record(
                analysis,
                relevant,
                scope,
                scrutinee,
                ClosureUseKind::Aggregate,
                &mut uses,
            ),
            Terminator::PrimitiveBranch { left, right, .. } => {
                record(
                    analysis,
                    relevant,
                    scope,
                    left,
                    ClosureUseKind::Escape,
                    &mut uses,
                );
                record(
                    analysis,
                    relevant,
                    scope,
                    right,
                    ClosureUseKind::Escape,
                    &mut uses,
                );
            }
            Terminator::Goto(_) => {}
        }
    }
    uses
}

fn record(
    analysis: &Analysis<'_>,
    relevant: &HashSet<FunctionId>,
    scope: Scope,
    atom: &Atom,
    kind: ClosureUseKind,
    uses: &mut Vec<ClosureUse>,
) {
    let mut sources = analysis
        .atom(scope, atom)
        .into_iter()
        .filter(|source| relevant.contains(&source.function()))
        .collect::<Vec<_>>();
    sources.sort_by_key(|source| source_key(*source));
    uses.extend(sources.into_iter().map(|source| ClosureUse {
        scope,
        atom: atom.id,
        source,
        kind,
    }));
}

fn pattern_binding(pattern: &Pattern) -> Option<ValueId> {
    match pattern {
        Pattern::Binding { id, .. } => Some(*id),
        Pattern::Product { .. } | Pattern::Wildcard { .. } => None,
    }
}
