use std::collections::HashMap;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

use super::{ApplicationGraph, TailCallPlan};

pub(in crate::c_emit::body) struct ContinuationGraph {
    sites: HashMap<StateId, ContinuationSite>,
}

struct ContinuationSite {
    caller: FunctionId,
    targets: Vec<FunctionId>,
}

impl ContinuationGraph {
    pub(in crate::c_emit::body) fn new(
        program: &Program,
        applications: &ApplicationGraph,
        tail_calls: &TailCallPlan,
    ) -> Self {
        let mut sites = HashMap::new();
        for function in &program.functions {
            for (site, targets) in applications.sites_from(function.id) {
                if !tail_calls.is_fused(site) {
                    sites.insert(
                        site,
                        ContinuationSite {
                            caller: function.id,
                            targets: targets.to_vec(),
                        },
                    );
                }
            }
        }
        Self { sites }
    }

    pub(in crate::c_emit::body) fn caller(&self, site: StateId) -> Option<FunctionId> {
        self.sites.get(&site).map(|site| site.caller)
    }

    pub(in crate::c_emit::body) fn targets(&self, site: StateId) -> Option<&[FunctionId]> {
        self.sites.get(&site).map(|site| site.targets.as_slice())
    }

    pub(in crate::c_emit::body) fn targets_from(
        &self,
        function: FunctionId,
    ) -> impl Iterator<Item = FunctionId> + '_ {
        self.sites
            .values()
            .filter(move |site| site.caller == function)
            .flat_map(|site| site.targets.iter().copied())
    }
}
