use std::collections::{HashMap, HashSet};

use crate::closure::ast::{AtomKind, FunctionId, Reference};
use crate::control::ast::{Program, StateId, Terminator};

use super::control_call::{ControlCallPlan, reachable_states};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::c_emit::body) struct ControlRegionId(pub(in crate::c_emit::body) usize);

pub(in crate::c_emit::body) struct ControlRegionPlan {
    regions: Vec<ControlRegion>,
    function_regions: HashMap<FunctionId, ControlRegionId>,
    site_regions: HashMap<StateId, ControlRegionId>,
}

struct ControlRegion {
    functions: Vec<FunctionId>,
    requires_common_control: bool,
}

impl ControlRegionPlan {
    pub(in crate::c_emit::body) fn new(program: &Program, calls: &ControlCallPlan) -> Self {
        let function_indices = program
            .functions
            .iter()
            .enumerate()
            .map(|(index, function)| (function.id, index))
            .collect::<HashMap<_, _>>();
        let mut graph = vec![Vec::new(); program.functions.len()];
        let mut recursive_sites = Vec::new();
        for function in &program.functions {
            let caller = function_indices[&function.id];
            for site in reachable_states(program, function.entry) {
                if !calls.is_recursive_dispatch(site) {
                    continue;
                }
                let Some(targets) = calls.recursive_dispatch_targets(site) else {
                    unreachable!("recursive dispatch retains its targets");
                };
                recursive_sites.push((site, function.id));
                for target in targets {
                    let target = function_indices[target];
                    if !graph[caller].contains(&target) {
                        graph[caller].push(target);
                    }
                }
            }
        }

        let mut components = strongly_connected_components(&graph)
            .into_iter()
            .filter(|component| {
                component.len() > 1
                    || component
                        .first()
                        .is_some_and(|node| graph[*node].contains(node))
            })
            .collect::<Vec<_>>();
        for component in &mut components {
            component.sort_unstable();
        }
        components.sort_by_key(|component| component[0]);

        let mut regions = components
            .into_iter()
            .map(|component| ControlRegion {
                functions: component
                    .into_iter()
                    .map(|index| program.functions[index].id)
                    .collect(),
                requires_common_control: false,
            })
            .collect::<Vec<_>>();
        let mut function_regions = HashMap::new();
        for (index, region) in regions.iter().enumerate() {
            let id = ControlRegionId(index);
            for function in &region.functions {
                function_regions.insert(*function, id);
            }
        }

        let mut site_regions = HashMap::new();
        for (site, caller) in recursive_sites {
            let region = function_regions[&caller];
            debug_assert!(
                calls
                    .recursive_dispatch_targets(site)
                    .expect("recursive site retains targets")
                    .iter()
                    .all(|target| function_regions.get(target) == Some(&region))
            );
            site_regions.insert(site, region);
            if !is_direct_self_call(&program.states[site.0].terminator, caller) {
                regions[region.0].requires_common_control = true;
            }
        }

        Self {
            regions,
            function_regions,
            site_regions,
        }
    }

    pub(in crate::c_emit::body) fn common_functions(
        &self,
    ) -> impl Iterator<Item = FunctionId> + '_ {
        self.regions
            .iter()
            .filter(|region| region.requires_common_control)
            .flat_map(|region| region.functions.iter().copied())
    }

    pub(in crate::c_emit::body) fn is_valid(
        &self,
        program: &Program,
        calls: &ControlCallPlan,
    ) -> bool {
        let all_sites_are_closed = self.site_regions.iter().all(|(site, region)| {
            calls
                .recursive_dispatch_targets(*site)
                .is_some_and(|targets| {
                    targets
                        .iter()
                        .all(|target| self.function_regions.get(target) == Some(region))
                })
                && program.functions.iter().any(|function| {
                    self.function_regions.get(&function.id) == Some(region)
                        && reachable_states(program, function.entry).contains(site)
                })
        });
        let all_recursive_sites_are_mapped = program.functions.iter().all(|function| {
            reachable_states(program, function.entry)
                .into_iter()
                .filter(|site| calls.is_recursive_dispatch(*site))
                .all(|site| self.site_regions.contains_key(&site))
        });
        let all_region_functions_are_mapped =
            self.regions.iter().enumerate().all(|(index, region)| {
                region.functions.iter().all(|function| {
                    self.function_regions.get(function) == Some(&ControlRegionId(index))
                })
            });
        all_sites_are_closed && all_recursive_sites_are_mapped && all_region_functions_are_mapped
    }
}

fn is_direct_self_call(terminator: &Terminator, caller: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::Call {
            callee:
                crate::closure::ast::Atom {
                    kind: AtomKind::Reference(Reference::SelfClosure(target)),
                    ..
                },
            ..
        } if *target == caller
    )
}

fn strongly_connected_components(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut finished = vec![false; graph.len()];
    let mut order = Vec::with_capacity(graph.len());
    for root in 0..graph.len() {
        if finished[root] {
            continue;
        }
        let mut active = HashSet::new();
        let mut stack = vec![(root, 0)];
        active.insert(root);
        while let Some((node, next)) = stack.last_mut() {
            if *next < graph[*node].len() {
                let target = graph[*node][*next];
                *next += 1;
                if !finished[target] && active.insert(target) {
                    stack.push((target, 0));
                }
            } else {
                let (node, _) = stack.pop().expect("active traversal has a node");
                active.remove(&node);
                if !finished[node] {
                    finished[node] = true;
                    order.push(node);
                }
            }
        }
    }

    let mut reverse = vec![Vec::new(); graph.len()];
    for (source, targets) in graph.iter().enumerate() {
        for target in targets {
            reverse[*target].push(source);
        }
    }
    let mut assigned = vec![false; graph.len()];
    let mut components = Vec::new();
    for root in order.into_iter().rev() {
        if assigned[root] {
            continue;
        }
        assigned[root] = true;
        let mut component = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            component.push(node);
            for target in &reverse[node] {
                if !assigned[*target] {
                    assigned[*target] = true;
                    pending.push(*target);
                }
            }
        }
        components.push(component);
    }
    components
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitions_cycles_without_joining_acyclic_edges() {
        let graph = vec![vec![1], vec![0, 2], vec![3], vec![2], vec![]];
        let mut components = strongly_connected_components(&graph);
        for component in &mut components {
            component.sort_unstable();
        }
        components.sort();

        assert_eq!(components, vec![vec![0, 1], vec![2, 3], vec![4]]);
    }

    #[test]
    fn handles_deep_graphs_without_recursive_host_traversal() {
        let count = 100_000;
        let mut graph = vec![Vec::new(); count];
        for (index, edges) in graph.iter_mut().enumerate().take(count - 1) {
            edges.push(index + 1);
        }

        let components = strongly_connected_components(&graph);

        assert_eq!(components.len(), count);
    }
}
