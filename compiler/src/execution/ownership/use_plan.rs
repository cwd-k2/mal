use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::control::ast::{Operation, StateId, Terminator};

use super::super::{ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan};
use super::destination::PatternDestination;
use super::identity::{TerminatorOperand, UseEffect, UseId, UseLocation};
use super::liveness::{binding_id, insert_pattern_bindings};
use super::managed::is_managed;
use super::operand::{binding_operands, terminator_argument, terminator_operands};

pub(super) fn jump_value_effect(destination: &PatternDestination) -> UseEffect {
    if destination.has_owner_successor() {
        UseEffect::Share
    } else {
        UseEffect::Borrow
    }
}

pub(super) struct UseInputs<'a> {
    pub(super) control: &'a crate::control::ast::Program,
    pub(super) calls: &'a ControlCallPlan,
    pub(super) regions: &'a ControlRegionPlan,
    pub(super) frames: &'a ControlFramePlan,
    pub(super) live_in: &'a [HashSet<ValueId>],
    pub(super) input_destinations: &'a HashMap<StateId, PatternDestination>,
    pub(super) binding_destinations: &'a HashMap<(StateId, usize), PatternDestination>,
    pub(super) drop_candidates: &'a HashMap<(StateId, usize), Vec<ValueId>>,
    pub(super) borrowed_bindings: &'a HashSet<ValueId>,
}

pub(super) fn collect_use_effects(inputs: UseInputs<'_>) -> HashMap<UseId, UseEffect> {
    let UseInputs {
        control,
        calls,
        regions,
        frames,
        live_in,
        input_destinations,
        binding_destinations,
        drop_candidates,
        borrowed_bindings,
    } = inputs;
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
    local_bindings.retain(|binding| !borrowed_bindings.contains(binding));
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let operands = binding_operands(&binding.operation);
            let result_has_owner_successor = binding_destinations
                .get(&(site, binding_index))
                .is_some_and(PatternDestination::has_owner_successor);
            let operation_requires_owner_successors = !matches!(
                binding.operation,
                Operation::Atom(_) | Operation::Product(_) | Operation::SumInjection { .. }
            ) || result_has_owner_successor;
            let dead = drop_candidates
                .get(&(site, binding_index))
                .map_or(&[][..], Vec::as_slice);
            for (operand_index, (operand, atom, owner_successor)) in operands.iter().enumerate() {
                if !is_managed(&atom.ty) {
                    continue;
                }
                let effect = if !owner_successor || !operation_requires_owner_successors {
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
        if let Terminator::Jump { target, .. } = &state.terminator
            && let Some((_, _, effect)) = terminator_uses
                .iter_mut()
                .find(|(operand, _, _)| *operand == TerminatorOperand::JumpValue)
        {
            *effect = jump_value_effect(
                input_destinations
                    .get(target)
                    .expect("every jump target has an input handoff"),
            );
        }
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
                if is_managed(&field.ty) && !borrowed_bindings.contains(&field.id) {
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
                    && effect != UseEffect::Borrow
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
        if let Terminator::Case { scrutinee, arms } = &state.terminator
            && let Type::Sum(members) = &scrutinee.ty
        {
            for (arm_ordinal, arm) in arms.iter().enumerate() {
                let member = members.get(arm.index).expect("checked case member");
                if is_managed(member)
                    && input_destinations
                        .get(&arm.target)
                        .is_some_and(PatternDestination::has_owner_successor)
                {
                    let effect = if binding_id(scrutinee).is_some_and(|id| {
                        local_bindings.contains(&id) && !live_in[arm.target.0].contains(&id)
                    }) {
                        UseEffect::Consume
                    } else {
                        UseEffect::Share
                    };
                    uses.insert(
                        UseId {
                            state: site,
                            location: UseLocation::CasePayload(arm_ordinal),
                        },
                        effect,
                    );
                }
            }
        }
    }
    uses
}

pub(super) fn exclude_consumed_sources(
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
