use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

pub(in crate::backend::llvm) struct Plan {
    dead_values: HashMap<(StateId, usize), Vec<ValueId>>,
}

impl Plan {
    pub(in crate::backend::llvm) fn new(control: &crate::control::ast::Program) -> Self {
        let mut live_in = vec![HashSet::new(); control.states.len()];
        loop {
            let mut changed = false;
            for (index, state) in control.states.iter().enumerate().rev() {
                let mut live = terminator_live(&state.terminator, &live_in);
                for binding in state.bindings.iter().rev() {
                    remove_pattern_bindings(&binding.pattern, &mut live);
                    visit_operation_atoms(&binding.operation, |atom| {
                        insert_managed_binding(atom, &mut live)
                    });
                }
                if let Some(input) = &state.input {
                    remove_pattern_bindings(input, &mut live);
                }
                if live != live_in[index] {
                    live_in[index] = live;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut dead_values = HashMap::new();
        for (state_index, state) in control.states.iter().enumerate() {
            let mut live = terminator_live(&state.terminator, &live_in);
            for (binding_index, binding) in state.bindings.iter().enumerate().rev() {
                let mut used = HashSet::new();
                visit_operation_atoms(&binding.operation, |atom| {
                    insert_managed_binding(atom, &mut used)
                });
                let dead = used
                    .iter()
                    .filter(|id| !live.contains(id))
                    .copied()
                    .collect::<Vec<_>>();
                if !dead.is_empty() {
                    dead_values.insert((StateId(state_index), binding_index), dead.clone());
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                visit_operation_atoms(&binding.operation, |atom| {
                    insert_managed_binding(atom, &mut live)
                });
            }
        }
        Self { dead_values }
    }

    pub(in crate::backend::llvm) fn dead_values(
        &self,
        state: StateId,
        binding: usize,
    ) -> &[ValueId] {
        self.dead_values
            .get(&(state, binding))
            .map_or(&[], Vec::as_slice)
    }
}

fn binding_id(atom: &Atom) -> Option<ValueId> {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) => Some(id),
        _ => None,
    }
}

fn insert_managed_binding(atom: &Atom, live: &mut HashSet<ValueId>) {
    if crate::execution::ownership::is_managed(&atom.ty)
        && let Some(id) = binding_id(atom)
    {
        live.insert(id);
    }
}

fn remove_pattern_bindings(pattern: &Pattern, live: &mut HashSet<ValueId>) {
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

fn terminator_live(terminator: &Terminator, live_in: &[HashSet<ValueId>]) -> HashSet<ValueId> {
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

fn visit_operation_atoms(operation: &Operation, mut visit: impl FnMut(&Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::SymbolLength { value: atom }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::Memory { argument: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for atom in captures {
                visit(atom);
            }
        }
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
