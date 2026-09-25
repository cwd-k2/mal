use std::collections::HashSet;

use super::ast::{Program, StateId};

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
        pending.extend(program.states[id.0].terminator.successors());
    }
    states
}
