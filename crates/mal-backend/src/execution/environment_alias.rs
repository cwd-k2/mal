//! Values that may borrow from the active closure environment.
//!
//! A managed value read from a capture is held by the environment for as long as the environment lives, so the
//! activation can borrow it instead of taking its own reference. The values derived from such a read stay tied to
//! the environment: a frame that suspends an activation must keep the environment while one of them is live, and
//! ownership may borrow the roots but never lets a tail call outlive the environment they borrow from.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::OptimizationPlan;
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
        while derive(program, &mut tied) {}
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

/// Adds to `tied` every value a state derives from a tied one; reports whether anything was added.
fn derive(program: &Program, tied: &mut HashSet<ValueId>) -> bool {
    let before = tied.len();
    for state in &program.states {
        for binding in &state.bindings {
            let derived = match &binding.operation {
                Operation::Atom(atom) | Operation::SumInjection { value: atom, .. } => {
                    is_tied(atom, tied)
                }
                Operation::Product(atoms) => atoms.iter().any(|atom| is_tied(atom, tied)),
                _ => false,
            };
            if derived {
                managed_leaves(&binding.pattern, tied);
            }
        }
        match &state.terminator {
            Terminator::Jump { target, value } if is_tied(value, tied) => {
                tie_input(program, *target, tied);
            }
            Terminator::Case { scrutinee, arms } if is_tied(scrutinee, tied) => {
                for arm in arms {
                    tie_input(program, arm.target, tied);
                }
            }
            _ => {}
        }
    }
    tied.len() != before
}

fn tie_input(program: &Program, target: StateId, tied: &mut HashSet<ValueId>) {
    if let Some(input) = &program.states[target.0].input {
        managed_leaves(input, tied);
    }
}

fn is_tied(atom: &Atom, tied: &HashSet<ValueId>) -> bool {
    matches!(atom.kind, AtomKind::Reference(Reference::Binding(id)) if tied.contains(&id))
}

fn managed_leaves(pattern: &Pattern, leaves: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, ty } if is_managed(ty) => {
            leaves.insert(*id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                managed_leaves(element, leaves);
            }
        }
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => {}
    }
}
