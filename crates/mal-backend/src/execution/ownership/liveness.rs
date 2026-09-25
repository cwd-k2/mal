use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::Terminator;

use super::managed::is_managed;

pub(super) fn binding_id(atom: &Atom) -> Option<ValueId> {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) => Some(id),
        _ => None,
    }
}

pub(super) fn insert_managed_binding(atom: &Atom, live: &mut HashSet<ValueId>) {
    if let Some(id) = managed_binding_id(atom) {
        live.insert(id);
    }
}

pub(super) fn managed_binding_id(atom: &Atom) -> Option<ValueId> {
    is_managed(&atom.ty).then(|| binding_id(atom)).flatten()
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
