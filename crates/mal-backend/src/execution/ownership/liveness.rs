use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, Pattern};
use crate::control::ast::Terminator;

use super::lifecycle::is_managed;

pub(super) fn insert_managed_binding(atom: &Atom, live: &mut HashSet<ValueId>) {
    if let Some(id) = managed_binding_id(atom) {
        live.insert(id);
    }
}

pub(super) fn managed_binding_id(atom: &Atom) -> Option<ValueId> {
    is_managed(&atom.ty).then(|| atom.binding()).flatten()
}

pub(super) fn remove_pattern_bindings(pattern: &Pattern, live: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            live.remove(id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                remove_pattern_bindings(element, live);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

pub(super) fn terminator_live(
    terminator: &Terminator,
    live_in: &[HashSet<ValueId>],
) -> HashSet<ValueId> {
    let mut live = HashSet::new();
    for successor in terminator.successors() {
        live.extend(live_in[successor.0].iter().copied());
    }
    terminator.for_each_atom(|atom| insert_managed_binding(atom, &mut live));
    live
}

pub(super) fn collect_pattern_binding_order(pattern: &Pattern, bindings: &mut Vec<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => bindings.push(*id),
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_binding_order(element, bindings);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

pub(super) fn insert_pattern_bindings(pattern: &Pattern, bindings: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            bindings.insert(*id);
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                insert_pattern_bindings(element, bindings);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

/// Every value the control program binds: function parameters, state inputs, and binding patterns, in program order.
pub(super) fn local_binding_order(control: &crate::control::ast::Program) -> Vec<ValueId> {
    let mut order = Vec::new();
    for function in &control.functions {
        if let Some(binding) = function.parameter.binding {
            order.push(binding);
        }
    }
    for state in &control.states {
        if let Some(input) = &state.input {
            collect_pattern_binding_order(input, &mut order);
        }
        for binding in &state.bindings {
            collect_pattern_binding_order(&binding.pattern, &mut order);
        }
    }
    order
}

pub(super) fn local_bindings(control: &crate::control::ast::Program) -> HashSet<ValueId> {
    local_binding_order(control).into_iter().collect()
}
