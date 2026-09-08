use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::{self as control, StateId};

use super::{ControlCallMode, reachable_states};

pub(super) fn direct_graph(
    program: &control::Program,
    modes: &HashMap<StateId, ControlCallMode>,
) -> HashMap<FunctionId, Vec<FunctionId>> {
    let mut graph: HashMap<FunctionId, Vec<FunctionId>> = HashMap::new();
    for function in &program.functions {
        for site in reachable_states(program, function.entry) {
            if let Some(ControlCallMode::Direct(callee)) = modes.get(&site) {
                graph.entry(function.id).or_default().push(*callee);
            }
        }
    }
    graph
}

pub(super) fn creates_cycle(
    graph: &HashMap<FunctionId, Vec<FunctionId>>,
    caller: FunctionId,
    callee: FunctionId,
) -> bool {
    let mut pending = vec![callee];
    let mut seen = HashSet::new();
    while let Some(current) = pending.pop() {
        if current == caller {
            return true;
        }
        if seen.insert(current)
            && let Some(next) = graph.get(&current)
        {
            pending.extend(next);
        }
    }
    false
}

pub(super) fn is_acyclic(
    graph: &HashMap<FunctionId, Vec<FunctionId>>,
    nodes: impl IntoIterator<Item = FunctionId>,
) -> bool {
    fn visit(
        graph: &HashMap<FunctionId, Vec<FunctionId>>,
        current: FunctionId,
        active: &mut HashSet<FunctionId>,
        finished: &mut HashSet<FunctionId>,
    ) -> bool {
        if finished.contains(&current) {
            return true;
        }
        if !active.insert(current) {
            return false;
        }
        if graph.get(&current).is_some_and(|next| {
            next.iter()
                .any(|target| !visit(graph, *target, active, finished))
        }) {
            return false;
        }
        active.remove(&current);
        finished.insert(current);
        true
    }

    let mut active = HashSet::new();
    let mut finished = HashSet::new();
    nodes
        .into_iter()
        .all(|node| visit(graph, node, &mut active, &mut finished))
}
