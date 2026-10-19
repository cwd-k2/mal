//! Local responsibility conservation checks independent from ownership-plan reconstruction.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;

use super::identity::{BindingSite, UseId, UseLocation};
use super::operand::{OperationEffect, binding_operands, terminator_argument, terminator_operands};
use super::{Inputs, ParameterEffect, PatternDestination, Plan, UseEffect, fused_arguments};

/// Checks local conservation rules without deriving the expected plan again.
///
/// Exact validation still compares the plan with a reconstruction from its authority. These checks
/// cover invariants that should remain true if the planner's derivation changes: destinations agree
/// with pattern lifecycle, every managed operand has one effect, observations do not take ownership,
/// and one local responsibility is not both consumed and dropped at the same point.
pub(super) fn preserves_responsibilities(plan: &Plan, inputs: &Inputs<'_>) -> bool {
    let effective = fused_arguments(inputs.control, inputs.calls);
    let control = &*effective;
    destinations_are_valid(plan, control)
        && binding_uses_are_valid(plan, control)
        && terminator_uses_are_total(plan, control)
        && drops_are_valid(plan)
        && parameters_are_valid(plan, control)
}

fn destinations_are_valid(plan: &Plan, control: &crate::control::ast::Program) -> bool {
    for state_index in 0..control.states.len() {
        let state = crate::control::ast::StateId(state_index);
        if let Some(pattern) = &control.states[state.0].input {
            let Some(destination) = plan.input_destinations.get(&state) else {
                return false;
            };
            if !destination_matches(plan, pattern, destination) {
                return false;
            }
        }
        for (binding, value) in control.states[state.0].bindings.iter().enumerate() {
            let Some(destination) = plan
                .binding_destinations
                .get(&BindingSite::new(state, binding))
            else {
                return false;
            };
            if !destination_matches(plan, &value.pattern, destination) {
                return false;
            }
        }
    }
    true
}

fn destination_matches(plan: &Plan, pattern: &Pattern, destination: &PatternDestination) -> bool {
    match (pattern, destination) {
        (Pattern::Binding { ty, .. }, PatternDestination::Unmanaged) => !super::is_managed(ty),
        (Pattern::Binding { id, ty }, PatternDestination::Initialize(target)) => {
            super::is_managed(ty) && id == target && !plan.borrowed_bindings.contains(id)
        }
        (Pattern::Binding { id, ty }, PatternDestination::Borrow(target)) => {
            super::is_managed(ty) && id == target && plan.borrowed_bindings.contains(id)
        }
        (Pattern::Binding { ty, .. }, PatternDestination::Discard) => super::is_managed(ty),
        (Pattern::Wildcard { ty, .. }, PatternDestination::Unmanaged) => !super::is_managed(ty),
        (Pattern::Wildcard { ty, .. }, PatternDestination::Discard) => super::is_managed(ty),
        (Pattern::Product { elements, .. }, PatternDestination::Product(destinations)) => {
            elements.len() == destinations.len()
                && elements
                    .iter()
                    .zip(destinations)
                    .all(|(element, destination)| destination_matches(plan, element, destination))
        }
        _ => false,
    }
}

fn binding_uses_are_valid(plan: &Plan, control: &crate::control::ast::Program) -> bool {
    for (state_index, state) in control.states.iter().enumerate() {
        let state_id = crate::control::ast::StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let site = BindingSite::new(state_id, binding_index);
            for (operand, atom, operation_effect) in binding_operands(&binding.operation) {
                let id = UseId {
                    state: state_id,
                    location: UseLocation::Binding {
                        binding: binding_index,
                        operand,
                    },
                };
                let effect = plan.uses.get(&id).copied();
                if super::is_managed(&atom.ty) {
                    let Some(effect) = effect else {
                        return false;
                    };
                    if operation_effect == OperationEffect::Borrow && effect != UseEffect::Borrow {
                        return false;
                    }
                    if effect == UseEffect::Consume
                        && atom.binding().is_some_and(|source| {
                            plan.drops_after_binding
                                .get(&site)
                                .is_some_and(|drops| drops.contains(&source))
                        })
                    {
                        return false;
                    }
                } else if effect.is_some() {
                    return false;
                }
            }
        }
    }
    true
}

fn terminator_uses_are_total(plan: &Plan, control: &crate::control::ast::Program) -> bool {
    for (state_index, state) in control.states.iter().enumerate() {
        let state_id = crate::control::ast::StateId(state_index);
        for (operand, atom, _) in
            terminator_operands(&state.terminator, terminator_argument(&state.terminator))
        {
            let effect = plan.uses.get(&UseId {
                state: state_id,
                location: UseLocation::Terminator(operand),
            });
            if super::is_managed(&atom.ty) != effect.is_some() {
                return false;
            }
        }
    }
    true
}

fn drops_are_valid(plan: &Plan) -> bool {
    plan.drops_after_binding
        .values()
        .chain(plan.drops_on_edge.values())
        .all(|drops| {
            let mut distinct = HashSet::<ValueId>::new();
            drops
                .iter()
                .all(|id| !plan.borrowed_bindings.contains(id) && distinct.insert(*id))
        })
}

fn parameters_are_valid(plan: &Plan, control: &crate::control::ast::Program) -> bool {
    plan.parameters.iter().all(|((function, _), effect)| {
        let Some(parameter) = control
            .functions
            .iter()
            .find(|candidate| candidate.id == *function)
            .and_then(|function| function.parameter.binding)
        else {
            return matches!(effect, ParameterEffect::Drop);
        };
        match effect {
            ParameterEffect::BorrowInto(target) => {
                *target == parameter && plan.borrowed_bindings.contains(target)
            }
            ParameterEffect::ShareInto(target) | ParameterEffect::ConsumeInto(target) => {
                *target == parameter && !plan.borrowed_bindings.contains(target)
            }
            ParameterEffect::Drop => true,
        }
    })
}
