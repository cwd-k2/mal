use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

mod buffer_abi;
mod control_storage;
mod control_top;
mod symbol_concat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    BufferDirectAbi,
    LocalControlStorage,
    LocalControlTop,
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
            .with(Technique::BufferDirectAbi)
            .with(Technique::LocalControlStorage)
            .with(Technique::LocalControlTop)
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
    direct_buffer_functions: HashSet<FunctionId>,
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
        let direct_buffer_functions = if enabled.contains(Technique::BufferDirectAbi) {
            buffer_abi::plan(execution)
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
            direct_buffer_functions,
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

    pub(super) fn uses_direct_buffer(&self, function: FunctionId) -> bool {
        self.direct_buffer_functions.contains(&function)
    }

    pub(super) fn localizes_control_top(&self, function: FunctionId) -> bool {
        self.local_control_top_functions.contains(&function)
    }

    pub(super) fn localizes_control_storage(&self, function: FunctionId) -> bool {
        self.local_control_storage_functions.contains(&function)
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

    pub(super) fn site_uses_direct_buffer(
        &self,
        applications: &crate::execution::ApplicationGraph,
        site: StateId,
    ) -> bool {
        applications.targets(site).is_some_and(|targets| {
            !targets.is_empty()
                && targets
                    .iter()
                    .all(|target| self.uses_direct_buffer(*target))
        })
    }
}

pub(super) fn type_contains_buffer(ty: &crate::check::ast::Type) -> bool {
    buffer_abi::contains_buffer(ty)
}

pub(super) fn type_has_single_buffer(ty: &crate::check::ast::Type) -> bool {
    buffer_abi::has_single_buffer(ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    #[test]
    fn validates_the_exact_symbol_concat_decisions() {
        let source = SourceFile::new(
            FileId::new(88),
            "llvm-optimization-plan.mal",
            "main :: Unit -> Int32 := () -> { left := \"a\" + \"b\"; result := left + \"c\"; (#result).i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check LLVM optimization fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let enabled = OptimizationSet::none().with(Technique::SymbolConcatReuse);
        let mut plan = OptimizationPlan::new(&execution, enabled);

        assert!(plan.is_valid(&execution, enabled));
        let decision = *plan
            .symbol_concatenations
            .keys()
            .next()
            .expect("consuming concat decision");
        plan.symbol_concatenations.remove(&decision);
        assert!(!plan.is_valid(&execution, enabled));
        assert!(
            OptimizationPlan::new(&execution, OptimizationSet::none())
                .symbol_concatenations
                .is_empty()
        );
        assert!(
            OptimizationPlan::new(&execution, OptimizationSet::none())
                .local_control_storage_functions
                .is_empty()
        );
        assert!(
            OptimizationPlan::new(&execution, OptimizationSet::none())
                .local_control_top_functions
                .is_empty()
        );
        assert!(
            OptimizationPlan::new(&execution, OptimizationSet::none())
                .direct_buffer_functions
                .is_empty()
        );
    }

    #[test]
    fn selects_recursive_functions_with_control_frames_for_local_top_storage() {
        let source = SourceFile::new(
            FileId::new(89),
            "llvm-local-control-top.mal",
            "sum :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { rest := sum(value - 1i32); value + rest; }; }; main :: Unit -> Int32 := () -> { sum(4i32) - 10i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check local control top fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize checked program"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let enabled = OptimizationSet::none().with(Technique::LocalControlTop);
        let mut plan = OptimizationPlan::new(&execution, enabled);

        assert_eq!(plan.local_control_top_functions.len(), 1);
        assert!(plan.is_valid(&execution, enabled));
        assert!(
            OptimizationPlan::new(&execution, OptimizationSet::none())
                .local_control_top_functions
                .is_empty()
        );
        plan.local_control_top_functions.clear();
        assert!(!plan.is_valid(&execution, enabled));

        let enabled = OptimizationSet::none().with(Technique::LocalControlStorage);
        let mut plan = OptimizationPlan::new(&execution, enabled);
        assert_eq!(plan.local_control_storage_functions.len(), 1);
        assert!(plan.local_control_top_functions.is_empty());
        assert!(plan.is_valid(&execution, enabled));
        plan.local_control_storage_functions.clear();
        assert!(!plan.is_valid(&execution, enabled));
    }
}
