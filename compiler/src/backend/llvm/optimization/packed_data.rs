use std::collections::{HashMap, HashSet};

use crate::closure::ast::{FunctionId, FunctionKind};
use crate::control::ast::{StateId, Terminator};
use crate::core::ast::PackedBuilderOperation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::backend::llvm) struct StablePackedAccess {
    pub(in crate::backend::llvm) operation: PackedBuilderOperation,
    pub(in crate::backend::llvm) element: crate::check::ast::Type,
    pub(in crate::backend::llvm) stable_data: bool,
    pub(in crate::backend::llvm) prepares_edit: bool,
}

#[derive(Eq, PartialEq)]
pub(crate) struct StablePackedAccessPlan {
    pub(super) stable_applications: HashMap<StateId, StablePackedAccess>,
}

impl StablePackedAccessPlan {
    pub(super) fn new(execution: &crate::execution::Program) -> Self {
        let lowered = &execution.lowered;
        let control = &execution.control;
        let applications = &execution.applications;
        let operations = lowered
            .functions
            .iter()
            .filter_map(|function| match function.kind {
                FunctionKind::PackedCapability {
                    operation,
                    ref element,
                } => Some((function.id, (operation, element.clone()))),
                FunctionKind::Ordinary { .. } => None,
            })
            .collect::<HashMap<_, _>>();
        let mut mutating_functions = HashSet::new();
        loop {
            let previous = mutating_functions.len();
            for (site, caller) in applications.sites() {
                let mutates = applications.targets(site).is_some_and(|targets| {
                    targets.iter().any(|target| {
                        operations.get(target).is_some_and(|(operation, _)| {
                            matches!(
                                operation,
                                PackedBuilderOperation::New
                                    | PackedBuilderOperation::NewUnique
                                    | PackedBuilderOperation::Put
                            )
                        }) || mutating_functions.contains(target)
                    })
                });
                if mutates && let Some(caller) = caller {
                    mutating_functions.insert(caller);
                }
            }
            if mutating_functions.len() == previous {
                break;
            }
        }

        let mut future = vec![false; control.states.len()];
        loop {
            let mut changed = false;
            for (index, state) in control.states.iter().enumerate() {
                let site = StateId(index);
                let value = successors(&state.terminator).any(|next| future[next.0])
                    || applications.targets(site).is_some_and(|targets| {
                        targets.iter().any(|target| {
                            operations.get(target).is_some_and(|(operation, _)| {
                                matches!(
                                    operation,
                                    PackedBuilderOperation::New
                                        | PackedBuilderOperation::NewUnique
                                        | PackedBuilderOperation::Put
                                )
                            }) || mutating_functions.contains(target)
                        })
                    });
                if value != future[index] {
                    future[index] = value;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut after_return = HashMap::<FunctionId, bool>::new();
        loop {
            let mut changed = false;
            for (site, caller) in applications.sites() {
                let caller_after = caller.is_some_and(|id| after_return.get(&id) == Some(&true));
                let local_after = match control.states[site.0].terminator {
                    Terminator::Call { resume, .. } => future[resume.0],
                    Terminator::TailCall { .. } => false,
                    _ => continue,
                };
                if !(caller_after || local_after) {
                    continue;
                }
                for target in applications.targets(site).into_iter().flatten() {
                    if after_return.insert(*target, true) != Some(true) {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        let stable_applications = applications
            .sites()
            .filter_map(|(site, caller)| {
                let targets = applications.targets(site)?;
                let accesses = targets
                    .iter()
                    .map(|target| operations.get(target))
                    .collect::<Option<Vec<_>>>()?;
                let (access, remaining) = accesses.split_first()?;
                let access = (*access).clone();
                let direct_access = matches!(
                    access.0,
                    PackedBuilderOperation::Get
                        | PackedBuilderOperation::Put
                        | PackedBuilderOperation::PutUnique
                ) && remaining
                    .iter()
                    .all(|candidate| equivalent_access(&access, candidate));
                let local_after = match control.states[site.0].terminator {
                    Terminator::Call { resume, .. } => future[resume.0],
                    Terminator::TailCall { .. } => false,
                    _ => return None,
                };
                let caller_after = caller.is_some_and(|id| after_return.get(&id) == Some(&true));
                let prepares_edit = accesses
                    .iter()
                    .any(|candidate| candidate.0 == PackedBuilderOperation::Put);
                direct_access.then_some((
                    site,
                    StablePackedAccess {
                        operation: access.0,
                        element: access.1,
                        stable_data: !local_after && !caller_after && !prepares_edit,
                        prepares_edit,
                    },
                ))
            })
            .collect();
        Self {
            stable_applications,
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            stable_applications: HashMap::new(),
        }
    }

    pub(super) fn stable_access(&self, site: StateId) -> Option<&StablePackedAccess> {
        self.stable_applications.get(&site)
    }
}

fn equivalent_access(
    left: &(PackedBuilderOperation, crate::check::ast::Type),
    right: &(PackedBuilderOperation, crate::check::ast::Type),
) -> bool {
    left.1 == right.1
        && (left.0 == right.0
            || matches!(
                (left.0, right.0),
                (
                    PackedBuilderOperation::Put | PackedBuilderOperation::PutUnique,
                    PackedBuilderOperation::Put | PackedBuilderOperation::PutUnique
                )
            ))
}

fn successors(terminator: &Terminator) -> impl Iterator<Item = StateId> + '_ {
    let mut result = Vec::new();
    match terminator {
        Terminator::Return(_) | Terminator::TailCall { .. } => {}
        Terminator::Goto(target) | Terminator::Jump { target, .. } => result.push(*target),
        Terminator::Call { resume, .. } => result.push(*resume),
        Terminator::Case { arms, .. } => result.extend(arms.iter().map(|arm| arm.target)),
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => result.extend([*otherwise, *then]),
    }
    result.into_iter()
}

#[cfg(test)]
#[path = "packed_data_tests.rs"]
mod tests;
