use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    dead_values: HashMap<(StateId, usize), Vec<ValueId>>,
    uses: HashMap<UseId, UseEffect>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UseId {
    pub(crate) state: StateId,
    pub(crate) location: UseLocation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UseLocation {
    Binding {
        binding: usize,
        operand: BindingOperand,
    },
    Terminator(TerminatorOperand),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BindingOperand {
    Atom,
    Capture(usize),
    ProductElement(usize),
    SumValue,
    SymbolLength,
    SymbolAt,
    MemoryArgument,
    ExternalArgument,
    NumericOperand,
    UnaryOperand,
    BinaryLeft,
    BinaryRight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TerminatorOperand {
    Return,
    JumpValue,
    CallCallee,
    CallArgument,
    TailCallee,
    TailArgument,
    CaseScrutinee,
    BranchLeft,
    BranchRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UseEffect {
    Borrow,
    Share,
    Consume,
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
                let mut used = Vec::new();
                visit_operation_atoms(&binding.operation, |atom| {
                    if let Some(id) = managed_binding_id(atom)
                        && !used.contains(&id)
                    {
                        used.push(id);
                    }
                });
                let dead = used
                    .into_iter()
                    .filter(|id| !live.contains(id))
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
        let uses = collect_use_effects(control, &live_in, &dead_values);
        Self { dead_values, uses }
    }

    pub(crate) fn is_valid(&self, control: &crate::control::ast::Program) -> bool {
        *self == Self::new(control)
    }

    pub(crate) fn dead_values(&self, state: StateId, binding: usize) -> &[ValueId] {
        self.dead_values
            .get(&(state, binding))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn binding_use(
        &self,
        state: StateId,
        binding: usize,
        operand: BindingOperand,
    ) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::Binding { binding, operand },
            })
            .copied()
    }

    pub(crate) fn terminator_use(
        &self,
        state: StateId,
        operand: TerminatorOperand,
    ) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::Terminator(operand),
            })
            .copied()
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
    if let Some(id) = managed_binding_id(atom) {
        live.insert(id);
    }
}

fn managed_binding_id(atom: &Atom) -> Option<ValueId> {
    is_managed(&atom.ty).then(|| binding_id(atom)).flatten()
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

fn collect_use_effects(
    control: &crate::control::ast::Program,
    live_in: &[HashSet<ValueId>],
    dead_values: &HashMap<(StateId, usize), Vec<ValueId>>,
) -> HashMap<UseId, UseEffect> {
    let mut uses = HashMap::new();
    let mut local_bindings = HashSet::new();
    for function in &control.functions {
        if let Some(binding) = function.parameter.binding {
            local_bindings.insert(binding);
        }
    }
    for state in &control.states {
        if let Some(input) = &state.input {
            insert_pattern_bindings(input, &mut local_bindings);
        }
        for binding in &state.bindings {
            insert_pattern_bindings(&binding.pattern, &mut local_bindings);
        }
    }
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let operands = binding_operands(&binding.operation);
            let dead = dead_values
                .get(&(site, binding_index))
                .map_or(&[][..], Vec::as_slice);
            for (operand_index, (operand, atom, owner_successor)) in operands.iter().enumerate() {
                if !is_managed(&atom.ty) {
                    continue;
                }
                let effect = if !owner_successor {
                    UseEffect::Borrow
                } else if let Some(id) = binding_id(atom) {
                    let has_later_same_source = operands[operand_index + 1..]
                        .iter()
                        .any(|(_, later, _)| binding_id(later) == Some(id));
                    if local_bindings.contains(&id) && dead.contains(&id) && !has_later_same_source
                    {
                        UseEffect::Consume
                    } else {
                        UseEffect::Share
                    }
                } else {
                    UseEffect::Share
                };
                uses.insert(
                    UseId {
                        state: site,
                        location: UseLocation::Binding {
                            binding: binding_index,
                            operand: *operand,
                        },
                    },
                    effect,
                );
            }
        }
        for (operand, atom, mut effect) in terminator_operands(&state.terminator) {
            if is_managed(&atom.ty) {
                if operand == TerminatorOperand::Return
                    && binding_id(atom).is_some_and(|id| local_bindings.contains(&id))
                {
                    effect = UseEffect::Consume;
                }
                if operand == TerminatorOperand::JumpValue
                    && binding_id(atom).is_some_and(|id| {
                        local_bindings.contains(&id)
                            && match &state.terminator {
                                Terminator::Jump { target, .. } => !live_in[target.0].contains(&id),
                                _ => false,
                            }
                    })
                {
                    effect = UseEffect::Consume;
                }
                uses.insert(
                    UseId {
                        state: site,
                        location: UseLocation::Terminator(operand),
                    },
                    effect,
                );
            }
        }
    }
    uses
}

fn insert_pattern_bindings(pattern: &Pattern, bindings: &mut HashSet<ValueId>) {
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

fn binding_operands(operation: &Operation) -> Vec<(BindingOperand, &Atom, bool)> {
    match operation {
        Operation::Atom(atom) => vec![(BindingOperand::Atom, atom, true)],
        Operation::MakeClosure { captures, .. } => captures
            .iter()
            .enumerate()
            .map(|(index, atom)| (BindingOperand::Capture(index), atom, true))
            .collect(),
        Operation::Product(elements) => elements
            .iter()
            .enumerate()
            .map(|(index, atom)| (BindingOperand::ProductElement(index), atom, true))
            .collect(),
        Operation::SumInjection { value, .. } => {
            vec![(BindingOperand::SumValue, value, true)]
        }
        Operation::SymbolLength { value } => {
            vec![(BindingOperand::SymbolLength, value, false)]
        }
        Operation::SymbolAt { argument } => {
            vec![(BindingOperand::SymbolAt, argument, false)]
        }
        Operation::Memory { argument, .. } => {
            vec![(BindingOperand::MemoryArgument, argument, false)]
        }
        Operation::ExternalCall { argument, .. } => {
            vec![(BindingOperand::ExternalArgument, argument, false)]
        }
        Operation::NumericConversion { operand } => {
            vec![(BindingOperand::NumericOperand, operand, false)]
        }
        Operation::PrimitiveUnary { operand, .. } => {
            vec![(BindingOperand::UnaryOperand, operand, false)]
        }
        Operation::PrimitiveBinary { left, right, .. } => vec![
            (BindingOperand::BinaryLeft, left, false),
            (BindingOperand::BinaryRight, right, false),
        ],
    }
}

fn terminator_operands(terminator: &Terminator) -> Vec<(TerminatorOperand, &Atom, UseEffect)> {
    match terminator {
        Terminator::Return(atom) => {
            vec![(TerminatorOperand::Return, atom, UseEffect::Share)]
        }
        Terminator::Jump { value, .. } => {
            vec![(TerminatorOperand::JumpValue, value, UseEffect::Share)]
        }
        Terminator::Call {
            callee, argument, ..
        } => vec![
            (TerminatorOperand::CallCallee, callee, UseEffect::Borrow),
            (TerminatorOperand::CallArgument, argument, UseEffect::Borrow),
        ],
        Terminator::TailCall { callee, argument } => vec![
            (TerminatorOperand::TailCallee, callee, UseEffect::Borrow),
            (TerminatorOperand::TailArgument, argument, UseEffect::Borrow),
        ],
        Terminator::Case { scrutinee, .. } => vec![(
            TerminatorOperand::CaseScrutinee,
            scrutinee,
            UseEffect::Borrow,
        )],
        Terminator::PrimitiveBranch { left, right, .. } => vec![
            (TerminatorOperand::BranchLeft, left, UseEffect::Borrow),
            (TerminatorOperand::BranchRight, right, UseEffect::Borrow),
        ],
        Terminator::Goto(_) => Vec::new(),
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

    #[test]
    fn shares_duplicate_owner_successors_before_consuming_the_source() {
        let source = SourceFile::new(
            FileId::new(91),
            "execution-ownership-duplicate.mal",
            "duplicate :: Unit -> (Symbol, Symbol) := () -> { value := \"a\" + \"b\"; (value, value); };\nmain :: Unit -> Int32 := () -> { pair := duplicate(); 0i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check duplicate ownership fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize duplicate ownership fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let plan = Plan::new(&control);

        let (site, binding) = control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                state
                    .bindings
                    .iter()
                    .enumerate()
                    .find_map(|(binding_index, binding)| {
                        matches!(&binding.operation, Operation::Product(elements) if elements.len() == 2 && binding_id(&elements[0]) == binding_id(&elements[1]))
                            .then_some((StateId(state_index), binding_index))
                    })
            })
            .expect("duplicate product binding");
        assert_eq!(
            plan.binding_use(site, binding, BindingOperand::ProductElement(0)),
            Some(UseEffect::Share)
        );
        assert_eq!(
            plan.binding_use(site, binding, BindingOperand::ProductElement(1)),
            Some(UseEffect::Consume)
        );
    }

    #[test]
    fn borrows_a_final_observation_before_dropping_its_source() {
        let source = SourceFile::new(
            FileId::new(92),
            "execution-ownership-observation.mal",
            "observe :: Unit -> USize := () -> { value := \"a\" + \"b\"; #value; };\nmain :: Unit -> Int32 := () -> { observe().i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check observation fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize observation fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let plan = Plan::new(&control);

        let (site, binding) = control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                state
                    .bindings
                    .iter()
                    .position(|binding| matches!(binding.operation, Operation::SymbolLength { .. }))
                    .map(|binding| (StateId(state_index), binding))
            })
            .expect("Symbol length binding");
        assert_eq!(
            plan.binding_use(site, binding, BindingOperand::SymbolLength),
            Some(UseEffect::Borrow)
        );
        assert!(!plan.dead_values(site, binding).is_empty());
    }
}
