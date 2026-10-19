//! Responsibility conservation checks independent from ownership-plan reconstruction.

use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;
use crate::control::ast::{StateId, Terminator};

use super::identity::{BindingSite, UseId, UseLocation};
use super::liveness::local_bindings;
use super::operand::{OperationEffect, binding_operands, terminator_argument, terminator_operands};
use super::{Inputs, ParameterEffect, PatternDestination, Plan, UseEffect, fused_arguments};

/// Checks conservation rules without deriving the expected plan again.
///
/// Exact validation still compares the plan with a reconstruction from its authority. These checks
/// cover invariants that should remain true if the planner's derivation changes: destinations agree
/// with pattern lifecycle, every managed operand has one effect, observations do not take ownership,
/// and every live owner is consumed, dropped, or preserved exactly once along each control path.
pub(super) fn preserves_responsibilities(plan: &Plan, inputs: &Inputs<'_>) -> bool {
    let effective = fused_arguments(inputs.control, inputs.calls);
    let control = &*effective;
    destinations_are_valid(plan, control)
        && binding_uses_are_valid(plan, control)
        && terminator_uses_are_total(plan, control)
        && drops_are_valid(plan)
        && parameters_are_valid(plan, control)
        && responsibilities_end_once_per_path(plan, inputs, control)
}

fn responsibilities_end_once_per_path(
    plan: &Plan,
    inputs: &Inputs<'_>,
    control: &crate::control::ast::Program,
) -> bool {
    let locals = local_bindings(control);
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        let Some(mut active) = plan.live_owners.get(state_index).cloned() else {
            return false;
        };
        if let Some(destination) = plan.input_destinations.get(&site)
            && !initialize_destination(destination, &mut active)
        {
            return false;
        }

        for (binding_index, binding) in state.bindings.iter().enumerate() {
            for (operand, atom, _) in binding_operands(&binding.operation) {
                if super::is_managed(&atom.ty)
                    && !apply_use(
                        &mut active,
                        &locals,
                        &plan.borrowed_bindings,
                        atom.binding(),
                        plan.uses.get(&UseId {
                            state: site,
                            location: UseLocation::Binding {
                                binding: binding_index,
                                operand,
                            },
                        }),
                    )
                {
                    return false;
                }
            }
            let Some(destination) = plan
                .binding_destinations
                .get(&BindingSite::new(site, binding_index))
            else {
                return false;
            };
            if !initialize_destination(destination, &mut active) {
                return false;
            }
            if !apply_drops(
                &mut active,
                plan.drops_after_binding
                    .get(&BindingSite::new(site, binding_index))
                    .map_or(&[], Vec::as_slice),
            ) {
                return false;
            }
        }

        if let Some(frame) = inputs.frames.frame(site) {
            for (field_index, field) in frame.fields.iter().enumerate() {
                if super::is_managed(&field.ty)
                    && !plan.borrowed_bindings.contains(&field.id)
                    && !apply_use(
                        &mut active,
                        &locals,
                        &plan.borrowed_bindings,
                        Some(field.id),
                        plan.uses.get(&UseId {
                            state: site,
                            location: UseLocation::FrameField(field_index),
                        }),
                    )
                {
                    return false;
                }
            }
        }
        for (operand, atom, _) in
            terminator_operands(&state.terminator, terminator_argument(&state.terminator))
        {
            if super::is_managed(&atom.ty)
                && !apply_use(
                    &mut active,
                    &locals,
                    &plan.borrowed_bindings,
                    atom.binding(),
                    plan.uses.get(&UseId {
                        state: site,
                        location: UseLocation::Terminator(operand),
                    }),
                )
            {
                return false;
            }
        }

        for (path, successor) in
            super::drop_plan::control_paths(site, &state.terminator, inputs.calls, inputs.frames)
        {
            let mut remaining = active.clone();
            if let (super::ControlPath::CaseArm(arm), Terminator::Case { scrutinee, .. }) =
                (path, &state.terminator)
                && let Some(effect) = plan.uses.get(&UseId {
                    state: site,
                    location: UseLocation::CasePayload(arm),
                })
                && !apply_use(
                    &mut remaining,
                    &locals,
                    &plan.borrowed_bindings,
                    scrutinee.binding(),
                    Some(effect),
                )
            {
                return false;
            }
            if !apply_drops(
                &mut remaining,
                plan.drops_on_edge
                    .get(&super::EdgeId { state: site, path })
                    .map_or(&[], Vec::as_slice),
            ) {
                return false;
            }
            if !remaining.iter().all(|id| {
                successor.is_some_and(|target| {
                    plan.live_owners
                        .get(target.0)
                        .is_some_and(|live| live.contains(id))
                })
            }) {
                return false;
            }
        }
    }
    true
}

fn apply_use(
    active: &mut HashSet<ValueId>,
    locals: &HashSet<ValueId>,
    borrowed: &HashSet<ValueId>,
    source: Option<ValueId>,
    effect: Option<&UseEffect>,
) -> bool {
    let Some(effect) = effect else {
        return false;
    };
    let Some(source) = source.filter(|source| locals.contains(source)) else {
        return true;
    };
    if borrowed.contains(&source) {
        return *effect != UseEffect::Consume;
    }
    match effect {
        UseEffect::Borrow | UseEffect::Share => active.contains(&source),
        UseEffect::Consume => active.remove(&source),
    }
}

fn initialize_destination(destination: &PatternDestination, active: &mut HashSet<ValueId>) -> bool {
    match destination {
        PatternDestination::Initialize(id) => active.insert(*id),
        PatternDestination::Product(elements) => elements
            .iter()
            .all(|element| initialize_destination(element, active)),
        PatternDestination::Unmanaged
        | PatternDestination::Borrow(_)
        | PatternDestination::Discard => true,
    }
}

fn apply_drops(active: &mut HashSet<ValueId>, drops: &[ValueId]) -> bool {
    drops.iter().all(|id| active.remove(id))
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
