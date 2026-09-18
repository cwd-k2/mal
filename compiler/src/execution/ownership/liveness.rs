use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

use super::managed::is_managed;

pub(super) fn successors(terminator: &Terminator) -> impl Iterator<Item = StateId> + '_ {
    let mut states = [None; 2];
    match terminator {
        Terminator::Goto(target) | Terminator::Jump { target, .. } => states[0] = Some(*target),
        Terminator::Call { resume, .. } => states[0] = Some(*resume),
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => states = [Some(*otherwise), Some(*then)],
        Terminator::Case { .. } | Terminator::Return(_) | Terminator::TailCall { .. } => {}
    }
    let fixed = states.into_iter().flatten();
    let arms = match terminator {
        Terminator::Case { arms, .. } => Some(arms.iter().map(|arm| arm.target)),
        _ => None,
    };
    fixed.chain(arms.into_iter().flatten())
}

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
    visit_terminator_successors(terminator, |successor| {
        live.extend(live_in[successor.0].iter().copied());
    });
    visit_terminator_atoms(terminator, |atom| insert_managed_binding(atom, &mut live));
    live
}

fn visit_terminator_successors(terminator: &Terminator, mut visit: impl FnMut(StateId)) {
    match terminator {
        Terminator::Goto(target) | Terminator::Jump { target, .. } => visit(*target),
        Terminator::Call { resume, .. } => visit(*resume),
        Terminator::Case { arms, .. } => {
            for arm in arms {
                visit(arm.target);
            }
        }
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => {
            visit(*otherwise);
            visit(*then);
        }
        Terminator::Return(_) | Terminator::TailCall { .. } => {}
    }
}

pub(super) fn visit_operation_atoms(operation: &Operation, mut visit: impl FnMut(&Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::SymbolLength { value: atom }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::Memory { argument: atom, .. }
        | Operation::PackedBuilder { argument: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for atom in captures {
                visit(atom);
            }
        }
        Operation::MakePackedCapability { builder, .. } => visit(builder),
        Operation::SymbolAt { argument } => visit(argument),
        Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}

fn visit_terminator_atoms(terminator: &Terminator, mut visit: impl FnMut(&Atom)) {
    match terminator {
        Terminator::Return(atom)
        | Terminator::Jump { value: atom, .. }
        | Terminator::Case {
            scrutinee: atom, ..
        } => visit(atom),
        Terminator::Call {
            callee, argument, ..
        }
        | Terminator::TailCall { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Terminator::PrimitiveBranch { left, right, .. } => {
            visit(left);
            visit(right);
        }
        Terminator::Goto(_) => {}
    }
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
