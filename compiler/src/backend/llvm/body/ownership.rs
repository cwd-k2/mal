use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

pub(super) struct Plan {
    consumptions: HashMap<(StateId, usize), Consumption>,
    dead_values: HashMap<(StateId, usize), Vec<ValueId>>,
}

#[derive(Clone, Copy)]
pub(super) enum Consumption {
    Left,
    Right,
}

impl Plan {
    pub(super) fn new(control: &crate::control::ast::Program) -> Self {
        let mut live_in = vec![HashSet::new(); control.states.len()];
        loop {
            let mut changed = false;
            for (index, state) in control.states.iter().enumerate().rev() {
                let mut live = terminator_live(&state.terminator, &live_in);
                for binding in state.bindings.iter().rev() {
                    remove_pattern_bindings(&binding.pattern, &mut live);
                    visit_operation_atoms(&binding.operation, |atom| {
                        insert_binding(atom, &mut live)
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

        let mut consumptions = HashMap::new();
        let mut dead_values = HashMap::new();
        for (state_index, state) in control.states.iter().enumerate() {
            let mut live = terminator_live(&state.terminator, &live_in);
            for (binding_index, binding) in state.bindings.iter().enumerate().rev() {
                let mut used = HashSet::new();
                visit_operation_atoms(&binding.operation, |atom| insert_binding(atom, &mut used));
                let dead = used
                    .iter()
                    .filter(|id| !live.contains(id))
                    .copied()
                    .collect::<Vec<_>>();
                if !dead.is_empty() {
                    dead_values.insert((StateId(state_index), binding_index), dead.clone());
                }
                if let Operation::PrimitiveBinary {
                    operator: crate::core::ast::BinaryPrimitive::Add,
                    left,
                    right,
                } = &binding.operation
                    && left.ty == Type::Symbol
                    && right.ty == Type::Symbol
                {
                    let left_dead = binding_id(left).is_some_and(|id| dead.contains(&id));
                    let right_dead = binding_id(right).is_some_and(|id| dead.contains(&id));
                    let consumption = if left_dead && binding_id(left) != binding_id(right) {
                        Some(Consumption::Left)
                    } else if right_dead && binding_id(left) != binding_id(right) {
                        Some(Consumption::Right)
                    } else {
                        None
                    };
                    if let Some(consumption) = consumption {
                        consumptions.insert((StateId(state_index), binding_index), consumption);
                    }
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                visit_operation_atoms(&binding.operation, |atom| insert_binding(atom, &mut live));
            }
        }
        Self {
            consumptions,
            dead_values,
        }
    }

    pub(super) fn consumption(&self, state: StateId, binding: usize) -> Option<Consumption> {
        self.consumptions.get(&(state, binding)).copied()
    }

    pub(super) fn dead_values(&self, state: StateId, binding: usize) -> &[ValueId] {
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

fn insert_binding(atom: &Atom, live: &mut HashSet<ValueId>) {
    if let Some(id) = binding_id(atom) {
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
    for successor in terminator_successors(terminator) {
        live.extend(live_in[successor.0].iter().copied());
    }
    visit_terminator_atoms(terminator, |atom| insert_binding(atom, &mut live));
    live
}

fn terminator_successors(terminator: &Terminator) -> Vec<StateId> {
    match terminator {
        Terminator::Goto(target) | Terminator::Jump { target, .. } => vec![*target],
        Terminator::Call { resume, .. } => vec![*resume],
        Terminator::Case { arms, .. } => arms.iter().map(|arm| arm.target).collect(),
        Terminator::PrimitiveBranch {
            otherwise, then, ..
        } => vec![*otherwise, *then],
        Terminator::Return(_) | Terminator::TailCall { .. } => Vec::new(),
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
