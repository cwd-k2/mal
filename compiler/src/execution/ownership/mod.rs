use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

use super::{ControlCallPlan, ControlFramePlan, ControlRegionPlan, ParameterPlan};

mod destination;
mod drop_plan;
mod identity;
mod liveness;
mod managed;
mod operand;
mod parameter;
mod use_plan;

#[cfg(test)]
mod tests;

pub(crate) use destination::PatternDestination;
use destination::plan_pattern;
use drop_plan::collect_edge_drops;
pub(crate) use identity::{
    BindingOperand, ControlPath, EdgeId, TerminatorOperand, UseEffect, UseId, UseLocation,
};
use liveness::{
    insert_managed_binding, managed_binding_id, remove_pattern_bindings, successors,
    terminator_live, visit_operation_atoms,
};
pub(crate) use managed::is_managed;
use parameter::collect_parameter_effects;
pub(crate) use parameter::{ParameterEffect, ParameterEntry};
use use_plan::{UseInputs, collect_use_effects, exclude_consumed_sources};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    input_destinations: HashMap<StateId, PatternDestination>,
    binding_destinations: HashMap<(StateId, usize), PatternDestination>,
    drops_after_binding: HashMap<(StateId, usize), Vec<ValueId>>,
    drops_on_edge: HashMap<EdgeId, Vec<ValueId>>,
    uses: HashMap<UseId, UseEffect>,
    parameters: HashMap<(FunctionId, ParameterEntry), ParameterEffect>,
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
        let mut input_destinations = HashMap::new();
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
                input_destinations.insert(StateId(index), plan_pattern(input, &live));
                remove_pattern_bindings(input, &mut live);
            }
            live_in[index] = live;
        }

        let mut binding_destinations = HashMap::new();
        let mut drops_after_binding = HashMap::new();
        for (state_index, state) in control.states.iter().enumerate() {
            let mut live = terminator_live(&state.terminator, &live_in);
            for (binding_index, binding) in state.bindings.iter().enumerate().rev() {
                binding_destinations.insert(
                    (StateId(state_index), binding_index),
                    plan_pattern(&binding.pattern, &live),
                );
                let mut used = Vec::new();
                visit_operation_atoms(&binding.operation, |atom| {
                    if let Some(id) = managed_binding_id(atom)
                        && !used.contains(&id)
                    {
                        used.push(id);
                    }
                });
                let drops = used
                    .into_iter()
                    .filter(|id| !live.contains(id))
                    .collect::<Vec<_>>();
                if !drops.is_empty() {
                    drops_after_binding.insert((StateId(state_index), binding_index), drops);
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                visit_operation_atoms(&binding.operation, |atom| {
                    insert_managed_binding(atom, &mut live)
                });
            }
        }
        let uses = collect_use_effects(UseInputs {
            control,
            calls,
            regions,
            frames,
            live_in: &live_in,
            input_destinations: &input_destinations,
            binding_destinations: &binding_destinations,
            drop_candidates: &drops_after_binding,
        });
        exclude_consumed_sources(control, &uses, &mut drops_after_binding);
        let drops_on_edge = collect_edge_drops(control, calls, frames, &live_in, &uses);
        let parameters = collect_parameter_effects(control, parameters);
        Self {
            input_destinations,
            binding_destinations,
            drops_after_binding,
            drops_on_edge,
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

    pub(crate) fn input_destination(&self, state: StateId) -> Option<&PatternDestination> {
        self.input_destinations.get(&state)
    }

    pub(crate) fn binding_destination(
        &self,
        state: StateId,
        binding: usize,
    ) -> Option<&PatternDestination> {
        self.binding_destinations.get(&(state, binding))
    }

    pub(crate) fn drops_on_edge(&self, state: StateId, path: ControlPath) -> &[ValueId] {
        self.drops_on_edge
            .get(&EdgeId { state, path })
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

    pub(crate) fn case_payload_use(&self, state: StateId, arm: usize) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::CasePayload(arm),
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
