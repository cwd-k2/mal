use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    dead_values: HashMap<(StateId, usize), Vec<ValueId>>,
}

impl Plan {
    pub(crate) fn new(control: &crate::control::ast::Program) -> Self {
        let mut live_in = vec![HashSet::new(); control.states.len()];
        for (index, state) in control.states.iter().enumerate() {
            debug_assert!(successors(&state.terminator).all(|successor| successor.0 < index));
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
            live_in[index] = live;
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
                    dead_values.insert((StateId(state_index), binding_index), dead);
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                visit_operation_atoms(&binding.operation, |atom| {
                    insert_managed_binding(atom, &mut live)
                });
            }
        }
        Self { dead_values }
    }

    pub(crate) fn is_valid(&self, control: &crate::control::ast::Program) -> bool {
        *self == Self::new(control)
    }

    pub(crate) fn dead_values(&self, state: StateId, binding: usize) -> &[ValueId] {
        self.dead_values
            .get(&(state, binding))
            .map_or(&[], Vec::as_slice)
    }
}

pub(crate) fn is_managed(ty: &Type) -> bool {
    ty.data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Packed(_) | Type::Function { .. }))
}

fn successors(terminator: &Terminator) -> impl Iterator<Item = StateId> + '_ {
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

fn binding_id(atom: &Atom) -> Option<ValueId> {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) => Some(id),
        _ => None,
    }
}

fn insert_managed_binding(atom: &Atom, live: &mut HashSet<ValueId>) {
    if is_managed(&atom.ty)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    #[test]
    fn classifies_shared_type_dags_once_per_node() {
        let mut unmanaged = Type::Unit;
        for _ in 0..64 {
            unmanaged = Type::Product(vec![unmanaged.clone(), unmanaged].into());
        }
        assert!(!is_managed(&unmanaged));

        let managed = Type::Product(vec![unmanaged, Type::Symbol].into());
        assert!(is_managed(&managed));
    }

    #[test]
    fn validates_the_exact_dead_value_facts() {
        let source = SourceFile::new(
            FileId::new(90),
            "execution-ownership-plan.mal",
            "main :: Unit -> Int32 := () -> { value := \"a\" + \"b\"; length := #value; length.i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check ownership fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize ownership fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let mut plan = Plan::new(&control);

        assert!(plan.is_valid(&control));
        let point = *plan
            .dead_values
            .keys()
            .next()
            .expect("dead managed value fact");
        plan.dead_values.remove(&point);
        assert!(!plan.is_valid(&control));
    }
}
