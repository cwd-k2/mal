use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Operation, StateId, Terminator};
use crate::core::ast::PackedBuilderOperation;

use super::contains_buffer;

pub(super) fn close_stability_over_calls(
    execution: &crate::execution::Program,
    stable: &mut HashSet<FunctionId>,
) {
    loop {
        let unstable = stable
            .iter()
            .copied()
            .filter(|function| {
                execution
                    .applications
                    .sites_from(*function)
                    .any(|(_, targets)| {
                        targets.is_empty() || targets.iter().any(|target| !stable.contains(target))
                    })
            })
            .collect::<Vec<_>>();
        if unstable.is_empty() {
            return;
        }
        for function in unstable {
            stable.remove(&function);
        }
    }
}

pub(super) fn close_regions(
    execution: &crate::execution::Program,
    direct: &mut HashSet<FunctionId>,
) {
    for region in execution.control_regions.ids() {
        let functions = execution.control_regions.functions(region);
        if functions.iter().any(|function| !direct.contains(function)) {
            for function in functions {
                direct.remove(function);
            }
        }
    }
}

pub(super) fn close_buffer_calls(
    execution: &crate::execution::Program,
    direct: &mut HashSet<FunctionId>,
) {
    let mut remove = HashSet::new();
    for (site, caller) in execution.applications.sites() {
        let Some(argument) = call_argument(&execution.control.states[site.0].terminator) else {
            continue;
        };
        if !contains_buffer(&argument.ty) {
            continue;
        }
        let Some(targets) = execution.applications.targets(site) else {
            continue;
        };
        if execution.applications.direct_target(site).is_none()
            && targets.iter().any(|target| !direct.contains(target))
        {
            remove.extend(
                targets
                    .iter()
                    .filter(|target| direct.contains(target))
                    .copied(),
            );
        }
        if caller.is_some_and(|caller| direct.contains(&caller))
            && targets.iter().any(|target| !direct.contains(target))
        {
            remove.extend(caller);
        }
    }
    direct.retain(|function| !remove.contains(function));
}

pub(super) fn has_no_relocation(execution: &crate::execution::Program, entry: StateId) -> bool {
    reachable_states(&execution.control, entry).all(|state| {
        execution.control.states[state.0]
            .bindings
            .iter()
            .all(|binding| match &binding.operation {
                Operation::PackedBuilder { operation, .. } => matches!(
                    operation,
                    PackedBuilderOperation::Get | PackedBuilderOperation::Put
                ),
                _ => true,
            })
    })
}

pub(super) fn has_no_control_frame(execution: &crate::execution::Program, entry: StateId) -> bool {
    reachable_states(&execution.control, entry)
        .all(|state| execution.control_frames.frame(state).is_none())
}

pub(super) fn captures_buffer_in_nested_closure(
    execution: &crate::execution::Program,
    entry: StateId,
) -> bool {
    reachable_states(&execution.control, entry).any(|state| {
        execution.control.states[state.0]
            .bindings
            .iter()
            .any(|binding| {
                matches!(
                    &binding.operation,
                    Operation::MakeClosure { captures, .. }
                        if captures.iter().any(|capture| contains_buffer(&capture.ty))
                )
            })
    })
}

pub(super) fn call_argument(terminator: &Terminator) -> Option<&crate::closure::ast::Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
}

pub(super) fn reachable_states(
    control: &crate::control::ast::Program,
    entry: StateId,
) -> impl Iterator<Item = StateId> + '_ {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    std::iter::from_fn(move || {
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            match &control.states[id.0].terminator {
                Terminator::Return(_) | Terminator::TailCall { .. } => {}
                Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
                Terminator::Call { resume, .. } => pending.push(*resume),
                Terminator::Case { arms, .. } => {
                    pending.extend(arms.iter().map(|arm| arm.target));
                }
                Terminator::PrimitiveBranch {
                    otherwise, then, ..
                } => pending.extend([*otherwise, *then]),
            }
            return Some(id);
        }
        None
    })
}
