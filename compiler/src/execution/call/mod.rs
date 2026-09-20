use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ApplicationGraph, ControlRegionId, ControlRegionPlan, OptimizationPlan};

mod graph;

use graph::{direct_graph, is_acyclic};

pub(super) use super::application::reachable_states;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlCallMode {
    Direct(FunctionId),
    DirectRegion(FunctionId),
    DirectSelfTail,
    Dispatch,
}

pub(crate) struct ControlCallPlan {
    modes: HashMap<StateId, ControlCallMode>,
    common_regions: HashSet<ControlRegionId>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
}

impl ControlCallPlan {
    pub(crate) fn new(
        control: &control::Program,
        applications: &ApplicationGraph,
        optimizations: &OptimizationPlan,
        regions: &ControlRegionPlan,
    ) -> Self {
        let mut modes = HashMap::new();
        for (site, caller) in applications.sites() {
            let mode = if caller.is_some() && optimizations.is_fused(site) {
                ControlCallMode::DirectSelfTail
            } else if caller.is_some() && regions.site_region(site).is_some() {
                optimizations
                    .direct_target(site)
                    .map(ControlCallMode::DirectRegion)
                    .unwrap_or(ControlCallMode::Dispatch)
            } else if let Some(callee) = optimizations.direct_target(site) {
                ControlCallMode::Direct(callee)
            } else {
                ControlCallMode::Dispatch
            };
            modes.insert(site, mode);
        }

        let direct_graph = direct_graph(control, &modes);
        debug_assert!(is_acyclic(
            &direct_graph,
            control.functions.iter().map(|function| function.id)
        ));
        let function_entries = control
            .functions
            .iter()
            .map(|function| (function.id, function.entry))
            .collect::<HashMap<_, _>>();
        let common_regions = regions
            .ids()
            .filter(|region| {
                region_requires_common_control(control, &function_entries, regions, &modes, *region)
            })
            .collect();
        let forwarded_self_arguments = optimizations.forwarded_self_arguments().clone();
        Self {
            modes,
            common_regions,
            forwarded_self_arguments,
        }
    }

    pub(crate) fn mode(&self, site: StateId) -> Option<ControlCallMode> {
        self.modes.get(&site).copied()
    }

    pub(crate) fn requires_common_control(&self, region: ControlRegionId) -> bool {
        self.common_regions.contains(&region)
    }

    pub(crate) fn forwarded_self_argument(&self, site: StateId) -> Option<&closure::Atom> {
        self.forwarded_self_arguments.get(&site)
    }

    pub(crate) fn is_valid(
        &self,
        control: &control::Program,
        applications: &ApplicationGraph,
        optimizations: &OptimizationPlan,
        regions: &ControlRegionPlan,
    ) -> bool {
        let expected = Self::new(control, applications, optimizations, regions);
        self.modes == expected.modes
            && self.common_regions == expected.common_regions
            && self.forwarded_self_arguments == expected.forwarded_self_arguments
    }
}

fn region_requires_common_control(
    program: &control::Program,
    function_entries: &HashMap<FunctionId, StateId>,
    regions: &ControlRegionPlan,
    modes: &HashMap<StateId, ControlCallMode>,
    region: ControlRegionId,
) -> bool {
    regions.functions(region).iter().any(|function| {
        let entry = *function_entries
            .get(function)
            .expect("region function has a control entry");
        reachable_states(program, entry).into_iter().any(|site| {
            regions.site_region(site) == Some(region)
                && match modes.get(&site) {
                    Some(ControlCallMode::Dispatch) => true,
                    Some(ControlCallMode::DirectRegion(_)) => {
                        !is_direct_self_call(&program.states[site.0].terminator, *function)
                    }
                    _ => false,
                }
        })
    })
}

fn is_direct_self_call(terminator: &Terminator, caller: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::Call {
            callee:
                closure::Atom {
                    kind: AtomKind::Reference(Reference::SelfClosure(target)),
                    ..
                },
            ..
        } if *target == caller
    )
}

#[cfg(test)]
mod tests;
