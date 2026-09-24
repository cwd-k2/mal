use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::{self as control, StateId};

use super::{ControlCallMode, reachable_states};

pub(in crate::execution) fn direct_graph(
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

pub(in crate::execution) fn is_acyclic(
    graph: &HashMap<FunctionId, Vec<FunctionId>>,
    nodes: impl IntoIterator<Item = FunctionId>,
) -> bool {
    let mut active = HashSet::new();
    let mut finished = HashSet::new();
    for root in nodes {
        if finished.contains(&root) {
            continue;
        }
        active.insert(root);
        let mut pending = vec![(root, 0_usize)];
        while let Some((node, next_target)) = pending.last_mut() {
            if let Some(target) = graph
                .get(node)
                .and_then(|targets| targets.get(*next_target))
            {
                *next_target += 1;
                if finished.contains(target) {
                    continue;
                }
                if !active.insert(*target) {
                    return false;
                }
                pending.push((*target, 0));
                continue;
            }
            let (node, _) = pending.pop().expect("pending function exists");
            active.remove(&node);
            finished.insert(node);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::ast::LambdaId;

    fn function(index: u32) -> FunctionId {
        FunctionId::Lambda(LambdaId(index))
    }

    #[test]
    fn checks_deep_call_graphs_without_host_recursion() {
        let count = 100_000_u32;
        let graph = (0..count - 1)
            .map(|index| (function(index), vec![function(index + 1)]))
            .collect::<HashMap<_, _>>();

        assert!(is_acyclic(&graph, (0..count).map(function)));

        let mut cyclic = graph;
        cyclic.insert(function(count - 1), vec![function(0)]);
        assert!(!is_acyclic(&cyclic, (0..count).map(function)));
    }
}
