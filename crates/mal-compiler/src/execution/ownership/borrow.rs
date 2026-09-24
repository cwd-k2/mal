use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::Atom;
use crate::control::ast::{Program, Terminator};

use super::liveness::{
    managed_binding_id, remove_pattern_bindings, successors, terminator_live, visit_operation_atoms,
};
use super::parameter::ParameterBorrows;
use crate::execution::SelfTailParameterPlan;

#[derive(Default)]
pub(super) struct BorrowPlan {
    authorities: HashMap<ValueId, HashSet<ValueId>>,
}

impl BorrowPlan {
    pub(super) fn new(
        control: &Program,
        parameters: &ParameterBorrows,
        self_tail_parameters: &SelfTailParameterPlan,
    ) -> Self {
        let initial = Self::default();
        let live_in = initial.live_in(control);
        let authorities = super::authority::collect(
            control,
            parameters,
            &live_in,
            &self_tail_parameters.persistent_lenders(&parameters.functions),
        );
        Self { authorities }
    }

    pub(super) fn bindings(&self) -> HashSet<ValueId> {
        self.authorities.keys().copied().collect()
    }

    pub(super) fn live_in(&self, control: &Program) -> Vec<HashSet<ValueId>> {
        let mut live_in = vec![HashSet::new(); control.states.len()];
        for (index, state) in control.states.iter().enumerate() {
            debug_assert!(successors(&state.terminator).all(|successor| successor.0 < index));
            let mut live = self.body_live(state, &live_in);
            if let Some(input) = &state.input {
                remove_pattern_bindings(input, &mut live);
            }
            live_in[index] = live;
        }
        live_in
    }

    fn body_live(
        &self,
        state: &crate::control::ast::State,
        live_in: &[HashSet<ValueId>],
    ) -> HashSet<ValueId> {
        let mut live = self.terminator_live(&state.terminator, live_in);
        for binding in state.bindings.iter().rev() {
            remove_pattern_bindings(&binding.pattern, &mut live);
            visit_operation_atoms(&binding.operation, |atom| self.insert(atom, &mut live));
        }
        live
    }

    pub(super) fn terminator_live(
        &self,
        terminator: &Terminator,
        live_in: &[HashSet<ValueId>],
    ) -> HashSet<ValueId> {
        let mut live = terminator_live(terminator, live_in);
        for binding in live.iter().copied().collect::<Vec<_>>() {
            self.insert_authority(binding, &mut live);
        }
        live
    }

    pub(super) fn insert(&self, atom: &Atom, live: &mut HashSet<ValueId>) {
        if let Some(binding) = managed_binding_id(atom) {
            live.insert(binding);
            self.insert_authority(binding, live);
        }
    }

    fn insert_authority(&self, binding: ValueId, live: &mut HashSet<ValueId>) {
        let mut pending = vec![binding];
        while let Some(binding) = pending.pop() {
            if let Some(sources) = self.authorities.get(&binding) {
                for source in sources {
                    if live.insert(*source) {
                        pending.push(*source);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::PatternDestination;
    use crate::check::ast::Type;
    use crate::closure::ast::Pattern;
    use crate::control::ast::{Operation, StateId, Terminator};
    use crate::source::{FileId, SourceFile};

    #[test]
    fn borrows_a_leaf_while_its_aggregate_owner_covers_the_lifetime() {
        let source = SourceFile::new(
            FileId::new(89),
            "borrowed-destructure.mal",
            "keep :: ((Symbol, Symbol), Bool) -> (Symbol, Symbol) := (argument) -> { (pair, condition) := argument; (left, _) := pair; if (condition) then { length := #left; pair } else { pair }; };\nmain :: Unit -> Int32 := () -> { result := keep(((\"a\", \"b\"), true)); 0i32; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check borrowed destructure fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize borrowed destructure fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let (site, binding, left) = execution
            .control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                state.bindings.iter().enumerate().find_map(
                    |(binding_index, binding)| match &binding.pattern {
                        Pattern::Product { elements, .. }
                            if elements.len() == 2
                                && matches!(binding.operation, Operation::Atom(_)) =>
                        {
                            match &elements[0] {
                                Pattern::Binding {
                                    id,
                                    ty: Type::Symbol,
                                } => Some((StateId(state_index), binding_index, *id)),
                                _ => None,
                            }
                        }
                        _ => None,
                    },
                )
            })
            .expect("Symbol product destructure");
        assert_eq!(
            execution.ownership.binding_destination(site, binding),
            Some(&PatternDestination::Product(vec![
                PatternDestination::Borrow(left),
                PatternDestination::Discard,
            ]))
        );
        assert!(execution.ownership.binding_is_borrowed(left));
    }

    #[test]
    fn propagates_parameter_authority_through_a_dead_intermediate_alias() {
        let source = SourceFile::new(
            FileId::new(105),
            "borrowed-nested-destructure.mal",
            "inspect :: ((Symbol, Int64), Bool) -> USize := (argument) -> { (pair, _) := argument; (text, _) := pair; #text; };\nmain :: Unit -> Int32 := () -> { inspect(((\"a\" + \"b\", 0i64), true)).i32; };"
                .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check nested destructure fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize nested destructure fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let text = execution
            .control
            .states
            .iter()
            .flat_map(|state| &state.bindings)
            .find_map(|binding| match &binding.pattern {
                Pattern::Product { elements, .. }
                    if matches!(binding.operation, Operation::Atom(_)) =>
                {
                    elements.iter().find_map(|element| match element {
                        Pattern::Binding {
                            id,
                            ty: Type::Symbol,
                        } => Some(*id),
                        _ => None,
                    })
                }
                _ => None,
            })
            .expect("nested Symbol binding");

        assert!(execution.ownership.binding_is_borrowed(text));
    }

    #[test]
    fn borrows_a_case_payload_while_the_sum_owner_covers_the_arm() {
        let source = SourceFile::new(
            FileId::new(88),
            "borrowed-case-payload.mal",
            "Choice :: [Symbol, Symbol];\nkeep :: Choice -> Choice := (choice) -> { choice[(value) -> { length := #value; choice }, (value) -> { length := #value; choice }] };\ncreate :: Symbol -> Choice := (value) -> [first, second] => { first(value) };\nmain :: Unit -> Int32 := () -> { result := keep(create(\"a\" + \"b\")); 0i32; };".into(),
        );
        let checked = crate::pipeline::check(&source).expect("check borrowed case fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize borrowed case fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let execution =
            crate::execution::lower(closure, crate::execution::OptimizationSet::production());
        let borrowed_inputs = execution
            .control
            .states
            .iter()
            .filter_map(|state| match &state.terminator {
                Terminator::Case { arms, .. } if arms.len() == 2 => Some(arms),
                _ => None,
            })
            .flat_map(|arms| arms.iter())
            .filter(|arm| {
                matches!(
                    execution.ownership.input_destination(arm.target),
                    Some(PatternDestination::Borrow(_))
                )
            })
            .count();
        assert_eq!(borrowed_inputs, 2);
        let case_payload_owners = execution
            .ownership
            .uses
            .keys()
            .filter(|use_id| matches!(use_id.location, super::super::UseLocation::CasePayload(_)))
            .count();
        assert_eq!(case_payload_owners, 0);
    }
}
