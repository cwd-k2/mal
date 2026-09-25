//! Values that may borrow from the active closure environment.
//!
//! A managed value read from a capture is held by the environment for as long as the environment lives, so the
//! activation can borrow it instead of taking its own reference. The values derived from such a read stay tied to
//! the environment: a frame that suspends an activation must keep the environment while one of them is live, and
//! ownership may borrow the roots but never lets a tail call outlive the environment they borrow from.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomKind, Reference};
use crate::control::ast::{Operation, Program, StateId};

use super::OptimizationPlan;
use super::derived::{close, managed_leaves};
use super::ownership::is_managed;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct EnvironmentAliasPlan {
    roots: HashSet<ValueId>,
    tied: HashSet<ValueId>,
}

impl EnvironmentAliasPlan {
    pub(crate) fn new(program: &Program, optimizations: &OptimizationPlan) -> Self {
        let mut roots = HashSet::new();
        for state in &program.states {
            for binding in &state.bindings {
                if let Operation::Atom(atom) = &binding.operation
                    && matches!(atom.kind, AtomKind::Reference(Reference::Capture(_)))
                    && is_managed(&atom.ty)
                    && !optimizations.takes_unique_capture(atom.id)
                {
                    managed_leaves(&binding.pattern, &mut roots);
                }
            }
        }
        let mut tied = roots.clone();
        let states = (0..program.states.len()).map(StateId).collect::<Vec<_>>();
        close(program, &states, &mut tied);
        Self { roots, tied }
    }

    /// The bindings that read a capture directly; ownership may borrow them from the environment.
    pub(crate) fn is_root(&self, binding: ValueId) -> bool {
        self.roots.contains(&binding)
    }

    /// Whether the value may share its lifetime with the environment, conservatively for derived values.
    pub(crate) fn is_tied(&self, binding: ValueId) -> bool {
        self.tied.contains(&binding)
    }
}
