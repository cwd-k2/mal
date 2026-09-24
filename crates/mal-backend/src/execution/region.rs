use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

use super::ContinuationGraph;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ControlRegionId(pub(crate) usize);

#[derive(Eq, PartialEq)]
pub(crate) struct ControlRegionPlan {
    regions: Vec<ControlRegion>,
    function_regions: HashMap<FunctionId, ControlRegionId>,
    site_regions: HashMap<StateId, ControlRegionId>,
    recursive_targets: HashMap<StateId, Vec<FunctionId>>,
}

#[derive(Eq, PartialEq)]
struct ControlRegion {
    functions: Vec<FunctionId>,
}

impl ControlRegionPlan {
    pub(crate) fn new(program: &Program, continuations: &ContinuationGraph<'_>) -> Self {
        let function_indices = program
            .functions
            .iter()
            .enumerate()
            .map(|(index, function)| (function.id, index))
            .collect::<HashMap<_, _>>();
        let mut edges = vec![HashSet::new(); program.functions.len()];
        for function in &program.functions {
            let caller = function_indices[&function.id];
            for target in continuations.targets_from(function.id) {
                if let Some(target) = function_indices.get(&target).copied() {
                    edges[caller].insert(target);
                }
            }
        }
        let graph = edges
            .into_iter()
            .map(|targets| {
                let mut targets = targets.into_iter().collect::<Vec<_>>();
                targets.sort_unstable();
                targets
            })
            .collect::<Vec<_>>();

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

        Self {
            regions,
            function_regions,
            site_regions,
            recursive_targets,
        }
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = ControlRegionId> + '_ {
        (0..self.regions.len()).map(ControlRegionId)
    }

    pub(crate) fn functions(&self, region: ControlRegionId) -> &[FunctionId] {
        &self.regions[region.0].functions
    }

    pub(crate) fn function_region(&self, function: FunctionId) -> Option<ControlRegionId> {
        self.function_regions.get(&function).copied()
    }

    pub(crate) fn site_region(&self, site: StateId) -> Option<ControlRegionId> {
        self.site_regions.get(&site).copied()
    }

    pub(crate) fn recursive_targets(&self, site: StateId) -> Option<&[FunctionId]> {
        self.recursive_targets.get(&site).map(Vec::as_slice)
    }

    pub(crate) fn is_valid(
        &self,
        program: &Program,
        continuations: &ContinuationGraph<'_>,
    ) -> bool {
        self == &Self::new(program, continuations)
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
    use crate::execution::{ApplicationGraph, ClosureUsePlan, OptimizationPlan, OptimizationSet};
    use crate::{anf, closure, control, core};
    use mal_frontend::{check, resolve};
    use mal_syntax::parser;
    use mal_syntax::source::{FileId, SourceFile};

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

    #[test]
    fn rejects_a_plan_with_a_missing_region() {
        let source = SourceFile::new(
            FileId::new(84),
            "region-plan.mal",
            "recurse :: Int32 -> Int32 := (value) -> {\n\
               if (value == 0i32) then { 0i32 } else {\n\
                 child := recurse(value - 1i32);\n\
                 child + 1i32;\n\
               };\n\
             };"
            .into(),
        );
        let parsed = parser::parse(&source).expect("parse region fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve region fixture");
        let checked = check::check(&resolved).expect("check region fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = closure::convert(&anf);
        let control = control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let optimizations = OptimizationPlan::new(
            &closure,
            &control,
            &applications,
            OptimizationSet::production(),
        );
        let continuations = ContinuationGraph::new(&applications, &optimizations);
        let mut regions = ControlRegionPlan::new(&control, &continuations);

        assert!(regions.is_valid(&control, &continuations));
        assert!(!regions.regions.is_empty());
        regions.regions.clear();
        assert!(!regions.is_valid(&control, &continuations));
    }
}
