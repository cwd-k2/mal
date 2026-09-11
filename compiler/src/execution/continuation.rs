use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

use super::{ApplicationGraph, OptimizationPlan};

pub(crate) struct ContinuationGraph<'a> {
    applications: &'a ApplicationGraph,
    optimizations: &'a OptimizationPlan,
}

impl<'a> ContinuationGraph<'a> {
    pub(crate) fn new(
        applications: &'a ApplicationGraph,
        optimizations: &'a OptimizationPlan,
    ) -> Self {
        Self {
            applications,
            optimizations,
        }
    }

    pub(crate) fn caller(&self, site: StateId) -> Option<FunctionId> {
        (!self.optimizations.is_fused(site))
            .then(|| self.applications.caller(site))
            .flatten()
    }

    pub(crate) fn targets(&self, site: StateId) -> Option<&[FunctionId]> {
        (!self.optimizations.is_fused(site))
            .then(|| self.applications.targets(site))
            .flatten()
    }

    pub(crate) fn targets_from(
        &self,
        function: FunctionId,
    ) -> impl Iterator<Item = FunctionId> + '_ {
        self.applications
            .sites_from(function)
            .filter(|(site, _)| !self.optimizations.is_fused(*site))
            .flat_map(|(_, targets)| targets.iter().copied())
    }
}
