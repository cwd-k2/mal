use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

mod control_storage;
mod control_top;
mod self_tail_parameter;
mod symbol_concat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    LocalControlStorage,
    LocalControlTop,
    SelfTailParameter,
    SymbolConcatReuse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptimizationSet(u8);

impl OptimizationSet {
    pub(crate) const fn none() -> Self {
        Self(0)
    }

    pub(crate) const fn production() -> Self {
        Self::none()
            .with(Technique::LocalControlStorage)
            .with(Technique::LocalControlTop)
            .with(Technique::SelfTailParameter)
            .with(Technique::SymbolConcatReuse)
    }

    pub(crate) const fn with(self, technique: Technique) -> Self {
        Self(self.0 | (1 << technique as u8))
    }

    const fn contains(self, technique: Technique) -> bool {
        self.0 & (1 << technique as u8) != 0
    }
}

#[derive(Eq, PartialEq)]
pub(super) struct OptimizationPlan {
    local_control_storage_functions: HashSet<FunctionId>,
    local_control_top_functions: HashSet<FunctionId>,
    self_tail_parameters: HashSet<FunctionId>,
    symbol_concatenations: HashMap<(StateId, usize), SymbolConcatMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SymbolConcatMode {
    Borrow,
    ConsumeLeft,
    ConsumeRight,
}

impl OptimizationPlan {
    pub(super) fn new(execution: &crate::execution::Program, enabled: OptimizationSet) -> Self {
        let local_control_storage_functions = if enabled.contains(Technique::LocalControlStorage) {
            control_storage::plan(execution)
        } else {
            HashSet::new()
        };
        let local_control_top_functions = if enabled.contains(Technique::LocalControlTop) {
            control_top::plan(execution)
        } else {
            HashSet::new()
        };
        let self_tail_parameters = if enabled.contains(Technique::SelfTailParameter) {
            self_tail_parameter::plan(execution)
        } else {
            HashSet::new()
        };
        let symbol_concatenations = if enabled.contains(Technique::SymbolConcatReuse) {
            symbol_concat::plan(&execution.control, &execution.ownership)
        } else {
            HashMap::new()
        };
        Self {
            local_control_storage_functions,
            local_control_top_functions,
            self_tail_parameters,
            symbol_concatenations,
        }
    }

    pub(super) fn is_valid(
        &self,
        execution: &crate::execution::Program,
        enabled: OptimizationSet,
    ) -> bool {
        self == &Self::new(execution, enabled)
    }

    pub(super) fn symbol_concat_mode(&self, site: StateId, binding: usize) -> SymbolConcatMode {
        self.symbol_concatenations
            .get(&(site, binding))
            .copied()
            .unwrap_or(SymbolConcatMode::Borrow)
    }

    pub(super) fn localizes_control_top(&self, function: FunctionId) -> bool {
        self.local_control_top_functions.contains(&function)
    }

    pub(super) fn localizes_control_storage(&self, function: FunctionId) -> bool {
        self.local_control_storage_functions.contains(&function)
    }

    pub(super) fn self_tail_parameter(&self, function: FunctionId) -> bool {
        self.self_tail_parameters.contains(&function)
    }

    pub(super) fn site_may_relocate_control_storage(
        &self,
        applications: &crate::execution::ApplicationGraph,
        site: StateId,
    ) -> bool {
        applications.targets(site).is_some_and(|targets| {
            targets
                .iter()
                .any(|target| self.localizes_control_storage(*target))
        })
    }
}

#[cfg(test)]
mod tests;
