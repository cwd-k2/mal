use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

use super::{ApplicationGraph, TailCallPlan};

pub(crate) struct ContinuationGraph<'a> {
    applications: &'a ApplicationGraph,
    tail_calls: &'a TailCallPlan,
}

impl<'a> ContinuationGraph<'a> {
    pub(crate) fn new(applications: &'a ApplicationGraph, tail_calls: &'a TailCallPlan) -> Self {
        Self {
            applications,
            tail_calls,
        }
    }

    pub(crate) fn caller(&self, site: StateId) -> Option<FunctionId> {
        (!self.tail_calls.is_fused(site))
            .then(|| self.applications.caller(site))
            .flatten()
    }

    pub(crate) fn targets(&self, site: StateId) -> Option<&[FunctionId]> {
        (!self.tail_calls.is_fused(site))
            .then(|| self.applications.targets(site))
            .flatten()
    }

    pub(crate) fn targets_from(
        &self,
        function: FunctionId,
    ) -> impl Iterator<Item = FunctionId> + '_ {
        self.applications
            .sites_from(function)
            .filter(|(site, _)| !self.tail_calls.is_fused(*site))
            .flat_map(|(_, targets)| targets.iter().copied())
    }
}
