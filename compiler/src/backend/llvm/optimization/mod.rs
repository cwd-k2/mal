use std::collections::HashMap;

use crate::control::ast::StateId;

mod symbol_concat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    SymbolConcatReuse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptimizationSet(u8);

impl OptimizationSet {
    pub(crate) const fn none() -> Self {
        Self(0)
    }

    pub(crate) const fn production() -> Self {
        Self::none().with(Technique::SymbolConcatReuse)
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
        let symbol_concatenations = if enabled.contains(Technique::SymbolConcatReuse) {
            symbol_concat::plan(&execution.control, &execution.ownership)
        } else {
            HashMap::new()
        };
        Self {
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
    }
}
