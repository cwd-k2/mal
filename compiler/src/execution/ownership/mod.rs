use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, FunctionId, Pattern, Reference};
use crate::control::ast::{Operation, StateId, Terminator};

use super::{
    ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan, ParameterDestination,
    ParameterPlan,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    drops_after_binding: HashMap<(StateId, usize), Vec<ValueId>>,
    uses: HashMap<UseId, UseEffect>,
    parameters: HashMap<(FunctionId, ParameterEntry), ParameterEffect>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ParameterEntry {
    BorrowedAbi,
    OwnedHandoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterEffect {
    ShareInto(ValueId),
    ConsumeInto(ValueId),
    Drop,
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
    FrameField(usize),
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
    pub(crate) fn new(
        control: &crate::control::ast::Program,
        parameters: &ParameterPlan,
        calls: &ControlCallPlan,
        regions: &ControlRegionPlan,
        frames: &ControlFramePlan,
    ) -> Self {
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

        let mut drops_after_binding = HashMap::new();
        for (state_index, state) in control.states.iter().enumerate() {
            let mut live = terminator_live(&state.terminator, &live_in);
            for (binding_index, binding) in state.bindings.iter().enumerate().rev() {
                let mut drops = Vec::new();
                collect_dead_pattern_bindings(&binding.pattern, &live, &mut drops);
                let mut used = Vec::new();
                visit_operation_atoms(&binding.operation, |atom| {
                    if let Some(id) = managed_binding_id(atom)
                        && !used.contains(&id)
                    {
                        used.push(id);
                    }
                });
                drops.extend(used.into_iter().filter(|id| !live.contains(id)));
                if !drops.is_empty() {
                    drops_after_binding.insert((StateId(state_index), binding_index), drops);
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                visit_operation_atoms(&binding.operation, |atom| {
                    insert_managed_binding(atom, &mut live)
                });
            }
        }
        let uses = collect_use_effects(
            control,
            calls,
            regions,
            frames,
            &live_in,
            &drops_after_binding,
        );
        exclude_consumed_sources(control, &uses, &mut drops_after_binding);
        let parameters = collect_parameter_effects(control, parameters);
        Self {
            drops_after_binding,
            uses,
            parameters,
        }
    }

    pub(crate) fn is_valid(
        &self,
        control: &crate::control::ast::Program,
        parameters: &ParameterPlan,
        calls: &ControlCallPlan,
        regions: &ControlRegionPlan,
        frames: &ControlFramePlan,
    ) -> bool {
        *self == Self::new(control, parameters, calls, regions, frames)
    }

    pub(crate) fn drops_after_binding(&self, state: StateId, binding: usize) -> &[ValueId] {
        self.drops_after_binding
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

    pub(crate) fn frame_field_use(&self, state: StateId, field: usize) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::FrameField(field),
            })
            .copied()
    }

    pub(crate) fn parameter_effect(
        &self,
        function: FunctionId,
        entry: ParameterEntry,
    ) -> Option<ParameterEffect> {
        self.parameters.get(&(function, entry)).copied()
    }
}

fn collect_parameter_effects(
    control: &crate::control::ast::Program,
    parameters: &ParameterPlan,
) -> HashMap<(FunctionId, ParameterEntry), ParameterEffect> {
    let mut effects = HashMap::new();
    for function in &control.functions {
        if !is_managed(&function.parameter.ty) {
            continue;
        }
        match parameters
            .destination(function.id)
            .expect("every control function has a parameter destination")
        {
            ParameterDestination::Bind(binding)
                if control.states[function.entry.0]
                    .live
                    .iter()
                    .any(|value| value.id == binding) =>
            {
                effects.insert(
                    (function.id, ParameterEntry::BorrowedAbi),
                    ParameterEffect::ShareInto(binding),
                );
                effects.insert(
                    (function.id, ParameterEntry::OwnedHandoff),
                    ParameterEffect::ConsumeInto(binding),
                );
            }
            ParameterDestination::Bind(_) | ParameterDestination::Discard => {
                effects.insert(
                    (function.id, ParameterEntry::OwnedHandoff),
                    ParameterEffect::Drop,
                );
            }
        }
    }
    effects
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

fn collect_dead_pattern_bindings(
    pattern: &Pattern,
    live_after: &HashSet<ValueId>,
    drops: &mut Vec<ValueId>,
) {
    match pattern {
        Pattern::Binding { id, ty } => {
            if is_managed(ty) && !live_after.contains(id) {
                drops.push(*id);
            }
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_dead_pattern_bindings(element, live_after, drops);
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
    calls: &ControlCallPlan,
    regions: &ControlRegionPlan,
    frames: &ControlFramePlan,
    live_in: &[HashSet<ValueId>],
    drop_candidates: &HashMap<(StateId, usize), Vec<ValueId>>,
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
            let dead = drop_candidates
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
        let effective_argument = calls
            .forwarded_self_argument(site)
            .or_else(|| terminator_argument(&state.terminator));
        let mut terminator_uses = terminator_operands(&state.terminator, effective_argument);
        let mode = calls.mode(site);
        let uses_common_control = regions
            .site_region(site)
            .is_some_and(|region| calls.requires_common_control(region));
        let common_region_transition =
            mode == Some(ControlCallMode::Dispatch) && uses_common_control;
        let frame = frames.frame(site);
        let callee_is_successor = uses_common_control
            && matches!(
                mode,
                Some(ControlCallMode::DirectRegion(_) | ControlCallMode::Dispatch)
            );
        let argument_is_successor = frame.is_some()
            || matches!(
                mode,
                Some(ControlCallMode::DirectSelfTail | ControlCallMode::DirectRegion(_))
            )
            || common_region_transition;
        for (operand, _, effect) in &mut terminator_uses {
            if matches!(
                operand,
                TerminatorOperand::CallCallee | TerminatorOperand::TailCallee
            ) && callee_is_successor
            {
                *effect = UseEffect::Share;
            }
            if matches!(
                operand,
                TerminatorOperand::CallArgument | TerminatorOperand::TailArgument
            ) && argument_is_successor
            {
                *effect = UseEffect::Share;
            }
        }

        let mut owner_successors = Vec::new();
        if let Some(frame) = frame {
            for (field_index, field) in frame.fields.iter().enumerate() {
                if is_managed(&field.ty) {
                    owner_successors.push((
                        UseId {
                            state: site,
                            location: UseLocation::FrameField(field_index),
                        },
                        Some(field.id),
                    ));
                }
            }
        }
        for (operand, atom, effect) in &terminator_uses {
            if *effect == UseEffect::Share && is_managed(&atom.ty) {
                owner_successors.push((
                    UseId {
                        state: site,
                        location: UseLocation::Terminator(*operand),
                    },
                    binding_id(atom).filter(|id| local_bindings.contains(id)),
                ));
            }
        }
        let mut seen_sources = HashSet::new();
        let mut owner_effects = HashMap::new();
        for (use_id, source) in owner_successors.into_iter().rev() {
            let effect = match source {
                Some(id) if seen_sources.insert(id) => UseEffect::Consume,
                _ => UseEffect::Share,
            };
            owner_effects.insert(use_id, effect);
        }
        for (use_id, effect) in &owner_effects {
            if matches!(use_id.location, UseLocation::FrameField(_)) {
                uses.insert(*use_id, *effect);
            }
        }

        for (operand, atom, mut effect) in terminator_uses {
            if is_managed(&atom.ty) {
                effect = owner_effects
                    .get(&UseId {
                        state: site,
                        location: UseLocation::Terminator(operand),
                    })
                    .copied()
                    .unwrap_or(effect);
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

fn exclude_consumed_sources(
    control: &crate::control::ast::Program,
    uses: &HashMap<UseId, UseEffect>,
    drops: &mut HashMap<(StateId, usize), Vec<ValueId>>,
) {
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let consumed = binding_operands(&binding.operation)
                .into_iter()
                .filter_map(|(operand, atom, _)| {
                    (uses.get(&UseId {
                        state: site,
                        location: UseLocation::Binding {
                            binding: binding_index,
                            operand,
                        },
                    }) == Some(&UseEffect::Consume))
                    .then(|| binding_id(atom))
                    .flatten()
                })
                .collect::<HashSet<_>>();
            if let Some(binding_drops) = drops.get_mut(&(site, binding_index)) {
                binding_drops.retain(|id| !consumed.contains(id));
            }
        }
    }
    drops.retain(|_, values| !values.is_empty());
}

fn terminator_argument(terminator: &Terminator) -> Option<&Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
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

fn terminator_operands<'a>(
    terminator: &'a Terminator,
    effective_argument: Option<&'a Atom>,
) -> Vec<(TerminatorOperand, &'a Atom, UseEffect)> {
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
            (
                TerminatorOperand::CallArgument,
                effective_argument.unwrap_or(argument),
                UseEffect::Borrow,
            ),
        ],
        Terminator::TailCall { callee, argument } => vec![
            (TerminatorOperand::TailCallee, callee, UseEffect::Borrow),
            (
                TerminatorOperand::TailArgument,
                effective_argument.unwrap_or(argument),
                UseEffect::Borrow,
            ),
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
    fn validates_the_exact_binding_drop_facts() {
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
        let mut execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());

        assert!(execution.ownership.is_valid(
            &execution.control,
            &execution.parameters,
            &execution.control_calls,
            &execution.control_regions,
            &execution.control_frames,
        ));
        let point = *execution
            .ownership
            .drops_after_binding
            .keys()
            .next()
            .expect("dead managed value fact");
        execution.ownership.drops_after_binding.remove(&point);
        assert!(!execution.ownership.is_valid(
            &execution.control,
            &execution.parameters,
            &execution.control_calls,
            &execution.control_regions,
            &execution.control_frames,
        ));
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
        let execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());
        let control = &execution.control;
        let plan = &execution.ownership;

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
        let Operation::Product(elements) = &control.states[site.0].bindings[binding].operation
        else {
            unreachable!();
        };
        let source = binding_id(&elements[0]).expect("local duplicate source");
        assert!(!plan.drops_after_binding(site, binding).contains(&source));
    }

    #[test]
    fn drops_an_unused_managed_binding_immediately() {
        let source = SourceFile::new(
            FileId::new(95),
            "execution-ownership-unused-result.mal",
            "main :: Unit -> Int32 := () -> { unused := \"a\" + \"b\"; 0i32; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check unused result fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize unused result fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());

        let (site, binding, id) = execution
            .control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                state
                    .bindings
                    .iter()
                    .enumerate()
                    .find_map(|(binding_index, binding)| match binding.pattern {
                        Pattern::Binding { id, ref ty }
                            if is_managed(ty)
                                && matches!(binding.operation, Operation::Atom(_)) =>
                        {
                            Some((StateId(state_index), binding_index, id))
                        }
                        _ => None,
                    })
            })
            .expect("unused managed binding");
        assert_eq!(
            execution.ownership.drops_after_binding(site, binding),
            &[id]
        );
    }

    #[test]
    fn distinguishes_borrowed_and_owned_parameter_entries() {
        let source = SourceFile::new(
            FileId::new(96),
            "execution-ownership-parameters.mal",
            "keep :: Symbol -> Symbol := (value) -> { value; };\nignore :: Symbol -> Int32 := (value) -> { 0i32; };\ndiscard :: Symbol -> Int32 := (_) -> { 0i32; };\nmain :: Unit -> Int32 := () -> { discard(keep(\"x\")) + ignore(\"y\"); };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check parameter ownership fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize parameter ownership fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());

        for function in execution
            .control
            .functions
            .iter()
            .filter(|function| is_managed(&function.parameter.ty))
        {
            match execution
                .parameters
                .destination(function.id)
                .expect("parameter destination")
            {
                ParameterDestination::Bind(binding)
                    if execution.control.states[function.entry.0]
                        .live
                        .iter()
                        .any(|value| value.id == binding) =>
                {
                    assert_eq!(
                        execution
                            .ownership
                            .parameter_effect(function.id, ParameterEntry::BorrowedAbi),
                        Some(ParameterEffect::ShareInto(binding))
                    );
                    assert_eq!(
                        execution
                            .ownership
                            .parameter_effect(function.id, ParameterEntry::OwnedHandoff),
                        Some(ParameterEffect::ConsumeInto(binding))
                    );
                }
                ParameterDestination::Bind(_) | ParameterDestination::Discard => {
                    assert_eq!(
                        execution
                            .ownership
                            .parameter_effect(function.id, ParameterEntry::BorrowedAbi),
                        None
                    );
                    assert_eq!(
                        execution
                            .ownership
                            .parameter_effect(function.id, ParameterEntry::OwnedHandoff),
                        Some(ParameterEffect::Drop)
                    );
                }
            }
        }
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
        let execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());
        let control = &execution.control;
        let plan = &execution.ownership;

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
        assert!(!plan.drops_after_binding(site, binding).is_empty());
    }

    #[test]
    fn shares_a_frame_field_before_consuming_the_same_next_argument() {
        let source = SourceFile::new(
            FileId::new(94),
            "execution-ownership-frame-argument.mal",
            "extern choose :: Unit -> Bool;\nwalk :: Symbol -> Symbol := (value) -> {\n  if (choose()) then { value } else {\n    child := walk(value);\n    if (#value == 0usize) then { child } else { child };\n  };\n};\nmain :: Unit -> Int32 := () -> { result := walk(\"x\"); (#result).i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check frame ownership fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize frame ownership fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, super::super::OptimizationSet::production());
        let (site, field_index) = execution
            .control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                let site = StateId(state_index);
                let frame = execution.control_frames.frame(site)?;
                let Terminator::Call { argument, .. } = &state.terminator else {
                    return None;
                };
                let argument_id = binding_id(argument)?;
                frame
                    .fields
                    .iter()
                    .position(|field| field.id == argument_id)
                    .map(|field| (site, field))
            })
            .expect("frame field and argument share a source");

        assert_eq!(
            execution.ownership.frame_field_use(site, field_index),
            Some(UseEffect::Share)
        );
        assert_eq!(
            execution
                .ownership
                .terminator_use(site, TerminatorOperand::CallArgument),
            Some(UseEffect::Consume)
        );
    }
}
