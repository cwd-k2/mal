//! Managed values that may share their lifetime with a root value.
//!
//! A value bound from a root by an atom, a destructuring pattern, a construction, a join input, or a case payload
//! holds the root's referents, so it lives no longer than the root unless the holder takes a reference of its own.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::ownership::is_managed;

/// Adds to `values` every managed value the `states` derive from a value already in it, until nothing is added.
pub(super) fn close(program: &Program, states: &[StateId], values: &mut HashSet<ValueId>) {
    while derive(program, states, values) {}
}

pub(super) fn managed_leaves(pattern: &Pattern, leaves: &mut HashSet<ValueId>) {
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

pub(super) fn derived_from(atom: &Atom, values: &HashSet<ValueId>) -> bool {
    matches!(atom.kind, AtomKind::Reference(Reference::Binding(id)) if values.contains(&id))
}

fn derive(program: &Program, states: &[StateId], values: &mut HashSet<ValueId>) -> bool {
    let before = values.len();
    for site in states {
        let state = &program.states[site.0];
        for binding in &state.bindings {
            let derived = match &binding.operation {
                Operation::Atom(atom) | Operation::SumInjection { value: atom, .. } => {
                    derived_from(atom, values)
                }
                Operation::Product(atoms) => atoms.iter().any(|atom| derived_from(atom, values)),
                _ => false,
            };
            if derived {
                managed_leaves(&binding.pattern, values);
            }
        }
        match &state.terminator {
            Terminator::Jump { target, value } if derived_from(value, values) => {
                tie_input(program, *target, values);
            }
            Terminator::Case { scrutinee, arms } if derived_from(scrutinee, values) => {
                for arm in arms {
                    tie_input(program, arm.target, values);
                }
            }
            _ => {}
        }
    }
    values.len() != before
}

fn tie_input(program: &Program, target: StateId, values: &mut HashSet<ValueId>) {
    if let Some(input) = &program.states[target.0].input {
        managed_leaves(input, values);
    }
}
