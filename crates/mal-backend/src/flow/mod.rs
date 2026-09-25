//! Closure flow: which functions can be the callee of each application site.
//!
//! A callee value can only come from a closure creation, so following creations through bindings, aggregates,
//! captures, parameters, results, and buffers bounds the callee far more tightly than its type does. The
//! analysis is context insensitive and field insensitive; every source of a function value is modeled, so a
//! callee that the analysis reaches with no function is dead code.

use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId};

pub(crate) use self::assign::holds_function;
pub(crate) use self::compatible::CompatibleTargets;

mod analysis;
mod assign;
mod compatible;
mod owner;
mod transfer;

pub(crate) struct ClosureFlow {
    callees: HashMap<StateId, HashSet<FunctionId>>,
    arguments: HashMap<StateId, HashSet<FunctionId>>,
}

impl ClosureFlow {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        compatible: &mut CompatibleTargets,
    ) -> Self {
        let (callees, arguments) = analysis::solve(closure, control, compatible);
        Self { callees, arguments }
    }

    /// The functions that can be the callee at `site`, or `None` when no function reaches it.
    pub(crate) fn callee(&self, site: StateId) -> Option<&HashSet<FunctionId>> {
        self.callees.get(&site)
    }

    /// The functions that can be inside the argument at `site`, or `None` when the argument holds no function.
    pub(crate) fn argument(&self, site: StateId) -> Option<&HashSet<FunctionId>> {
        self.arguments.get(&site)
    }
}

#[cfg(test)]
mod tests;
