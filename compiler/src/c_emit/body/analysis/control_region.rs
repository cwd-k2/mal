use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

use super::ContinuationGraph;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::c_emit::body) struct ControlRegionId(pub(in crate::c_emit::body) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::c_emit::body) struct ControlArenaId(pub(in crate::c_emit::body) usize);

pub(in crate::c_emit::body) struct ControlRegionPlan {
    regions: Vec<ControlRegion>,
    function_regions: HashMap<FunctionId, ControlRegionId>,
    site_regions: HashMap<StateId, ControlRegionId>,
    recursive_targets: HashMap<StateId, Vec<FunctionId>>,
}

struct ControlRegion {
    functions: Vec<FunctionId>,
    arena: Option<ControlArenaId>,
}

impl ControlRegionPlan {
    pub(in crate::c_emit::body) fn new(
        program: &Program,
        continuations: &ContinuationGraph,
    ) -> Self {
        let function_indices = program
            .functions
            .iter()
            .enumerate()
            .map(|(index, function)| (function.id, index))
            .collect::<HashMap<_, _>>();
        let mut graph = vec![Vec::new(); program.functions.len()];
        for function in &program.functions {
            let caller = function_indices[&function.id];
            for target in continuations.targets_from(function.id) {
                if let Some(target) = function_indices.get(&target).copied()
                    && !graph[caller].contains(&target)
                {
                    graph[caller].push(target);
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

        let regions = components
            .into_iter()
            .map(|component| ControlRegion {
                functions: component
                    .into_iter()
                    .map(|index| program.functions[index].id)
                    .collect(),
                arena: None,
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
        let mut recursive_targets = HashMap::new();
        for index in 0..program.states.len() {
            let site = StateId(index);
            let Some(caller) = continuations.caller(site) else {
                continue;
            };
            let Some(region) = function_regions.get(&caller).copied() else {
                continue;
            };
            let targets = continuations
                .targets(site)
                .into_iter()
                .flatten()
                .copied()
                .filter(|target| function_regions.get(target) == Some(&region))
                .collect::<Vec<_>>();
            if !targets.is_empty() {
                site_regions.insert(site, region);
                recursive_targets.insert(site, targets);
            }
        }

        let mut regions = regions;
        let mut next_arena = 0;
        for (index, region) in regions.iter_mut().enumerate() {
            if site_regions.iter().any(|(site, site_region)| {
                site_region.0 == index
                    && matches!(
                        program.states[site.0].terminator,
                        crate::control::ast::Terminator::Call { .. }
                    )
            }) {
                region.arena = Some(ControlArenaId(next_arena));
                next_arena += 1;
            }
        }

        Self {
            regions,
            function_regions,
            site_regions,
            recursive_targets,
        }
    }

    pub(in crate::c_emit::body) fn ids(&self) -> impl Iterator<Item = ControlRegionId> + '_ {
        (0..self.regions.len()).map(ControlRegionId)
    }

    pub(in crate::c_emit::body) fn arena_count(&self) -> usize {
        self.regions
            .iter()
            .filter(|region| region.arena.is_some())
            .count()
    }

    pub(in crate::c_emit::body) fn functions(&self, region: ControlRegionId) -> &[FunctionId] {
        &self.regions[region.0].functions
    }

    pub(in crate::c_emit::body) fn arena(&self, region: ControlRegionId) -> Option<ControlArenaId> {
        self.regions[region.0].arena
    }

    pub(in crate::c_emit::body) fn function_region(
        &self,
        function: FunctionId,
    ) -> Option<ControlRegionId> {
        self.function_regions.get(&function).copied()
    }

    pub(in crate::c_emit::body) fn site_region(&self, site: StateId) -> Option<ControlRegionId> {
        self.site_regions.get(&site).copied()
    }

    pub(in crate::c_emit::body) fn recursive_targets(
        &self,
        site: StateId,
    ) -> Option<&[FunctionId]> {
        self.recursive_targets.get(&site).map(Vec::as_slice)
    }

    pub(in crate::c_emit::body) fn is_valid(
        &self,
        program: &Program,
        continuations: &ContinuationGraph,
    ) -> bool {
        let all_sites_are_closed = self.site_regions.iter().all(|(site, region)| {
            self.recursive_targets(*site).is_some_and(|targets| {
                targets
                    .iter()
                    .all(|target| self.function_regions.get(target) == Some(region))
            }) && program.functions.iter().any(|function| {
                self.function_regions.get(&function.id) == Some(region)
                    && continuations.caller(*site) == Some(function.id)
            })
        });
        let all_recursive_sites_are_mapped = (0..program.states.len()).all(|index| {
            let site = StateId(index);
            let Some(caller) = continuations.caller(site) else {
                return !self.site_regions.contains_key(&site);
            };
            let Some(region) = self.function_regions.get(&caller).copied() else {
                return !self.site_regions.contains_key(&site);
            };
            let expected = continuations
                .targets(site)
                .into_iter()
                .flatten()
                .copied()
                .filter(|target| self.function_regions.get(target) == Some(&region))
                .collect::<Vec<_>>();
            if expected.is_empty() {
                !self.site_regions.contains_key(&site)
            } else {
                self.site_regions.get(&site) == Some(&region)
                    && self.recursive_targets(site) == Some(expected.as_slice())
            }
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
