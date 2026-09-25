use std::collections::HashSet;

use super::ast::{Program, StateId, Terminator};

/// The states reachable from `entry` through the terminators of the control graph, in discovery order.
pub(crate) fn reachable_states(program: &Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        states.push(id);
        match &program.states[id.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => pending.extend(arms.iter().map(|arm| arm.target)),
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
}
