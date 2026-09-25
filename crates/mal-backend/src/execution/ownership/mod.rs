use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

use super::{
    ApplicationGraph, ControlCallPlan, ControlFramePlan, ControlRegionPlan, EnvironmentAliasPlan,
    OptimizationPlan, ParameterPlan, SelfTailParameterPlan,
};

mod authority;
mod borrow;
mod convention;
mod destination;
mod drop_plan;
mod identity;
mod liveness;
mod managed;
mod operand;
mod parameter;
mod use_plan;

#[cfg(test)]
mod construction_tests;
#[cfg(test)]
mod environment_tests;
#[cfg(test)]
mod parameter_tests;
#[cfg(test)]
mod successor_tests;
#[cfg(test)]
mod tests;

use borrow::BorrowPlan;
pub(crate) use destination::PatternDestination;
use destination::plan_borrowed_pattern;
use drop_plan::collect_edge_drops;
pub(crate) use identity::{
    BindingOperand, ControlPath, EdgeId, TerminatorOperand, UseEffect, UseId, UseLocation,
};
use liveness::{managed_binding_id, remove_pattern_bindings};
pub(crate) use managed::is_managed;
use parameter::{ParameterBorrows, collect_parameter_effects};
pub(crate) use parameter::{ParameterEffect, ParameterEntry};
use use_plan::{UseInputs, collect_use_effects, exclude_consumed_sources};

/// The position of every binding in control order, to release values in a deterministic order.
fn binding_order(control: &crate::control::ast::Program) -> HashMap<ValueId, usize> {
    liveness::local_binding_order(control)
        .into_iter()
        .enumerate()
        .map(|(position, id)| (id, position))
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Plan {
    input_destinations: HashMap<StateId, PatternDestination>,
    binding_destinations: HashMap<(StateId, usize), PatternDestination>,
    drops_after_binding: HashMap<(StateId, usize), Vec<ValueId>>,
    drops_on_edge: HashMap<EdgeId, Vec<ValueId>>,
    uses: HashMap<UseId, UseEffect>,
    parameters: HashMap<(FunctionId, ParameterEntry), ParameterEffect>,
    borrowed_bindings: HashSet<ValueId>,
    owned_functions: HashSet<FunctionId>,
    owned_sites: HashSet<StateId>,
}

pub(crate) struct Inputs<'a> {
    control: &'a crate::control::ast::Program,
    applications: &'a ApplicationGraph,
    optimizations: &'a OptimizationPlan,
    parameters: &'a ParameterPlan,
    calls: &'a ControlCallPlan,
    regions: &'a ControlRegionPlan,
    frames: &'a ControlFramePlan,
}

impl<'a> Inputs<'a> {
    pub(crate) fn new(
        control: &'a crate::control::ast::Program,
        applications: &'a ApplicationGraph,
        optimizations: &'a OptimizationPlan,
        parameters: &'a ParameterPlan,
        calls: &'a ControlCallPlan,
        regions: &'a ControlRegionPlan,
        frames: &'a ControlFramePlan,
    ) -> Self {
        Self {
            control,
            applications,
            optimizations,
            parameters,
            calls,
            regions,
            frames,
        }
    }
}

/// The control program as the ownership plan reads it: a tail call fused into a self transition passes the forwarded
/// argument, not the product that carried it to the forwarder, so liveness and use effects see the value the
/// transition hands over.
fn fused_arguments<'a>(
    control: &'a crate::control::ast::Program,
    calls: &ControlCallPlan,
) -> std::borrow::Cow<'a, crate::control::ast::Program> {
    let forwarded = (0..control.states.len())
        .filter_map(|index| {
            calls
                .forwarded_self_argument(StateId(index))
                .map(|atom| (index, atom.clone()))
        })
        .collect::<Vec<_>>();
    if forwarded.is_empty() {
        return std::borrow::Cow::Borrowed(control);
    }
    let mut program = control.clone();
    for (index, atom) in forwarded {
        if let crate::control::ast::Terminator::TailCall { argument, .. } =
            &mut program.states[index].terminator
        {
            *argument = atom;
        }
    }
    std::borrow::Cow::Owned(program)
}

impl Plan {
    pub(crate) fn new(inputs: Inputs<'_>) -> Self {
        let Inputs {
            control,
            applications,
            optimizations,
            parameters,
            calls,
            regions,
            frames,
        } = inputs;
        let effective = fused_arguments(control, calls);
        let control = &*effective;
        let parameter_borrows =
            ParameterBorrows::new(control, applications, calls, regions, frames);
        let self_tail_parameters = SelfTailParameterPlan::candidates(control, applications, calls);
        let environment_aliases = EnvironmentAliasPlan::new(control, optimizations);
        let borrows = BorrowPlan::new(
            control,
            &parameter_borrows,
            &self_tail_parameters,
            &environment_aliases,
        );
        let live_in = borrows.live_in(control);
        let borrowed_bindings = borrows.bindings();
        let mut input_destinations = HashMap::new();
        for (index, state) in control.states.iter().enumerate() {
            let mut live = borrows.terminator_live(&state.terminator, &live_in);
            for binding in state.bindings.iter().rev() {
                remove_pattern_bindings(&binding.pattern, &mut live);
                binding
                    .operation
                    .for_each_atom(|atom| borrows.insert(atom, &mut live));
            }
            if let Some(input) = &state.input {
                input_destinations.insert(
                    StateId(index),
                    plan_borrowed_pattern(input, &live, &borrowed_bindings),
                );
            }
        }

        let binding_order = binding_order(control);
        let mut binding_destinations = HashMap::new();
        let mut drops_after_binding = HashMap::new();
        for (state_index, state) in control.states.iter().enumerate() {
            let mut live = borrows.terminator_live(&state.terminator, &live_in);
            for (binding_index, binding) in state.bindings.iter().enumerate().rev() {
                let destination =
                    plan_borrowed_pattern(&binding.pattern, &live, &borrowed_bindings);
                binding_destinations.insert((StateId(state_index), binding_index), destination);
                let mut used = Vec::new();
                binding.operation.for_each_atom(|atom| {
                    if let Some(id) = managed_binding_id(atom)
                        && !used.contains(&id)
                    {
                        used.push(id);
                    }
                });
                // A borrowed operand keeps its lenders alive, so when its last use ends here the lenders end
                // here too and must be released with the operand.
                let mut lenders = HashSet::new();
                binding
                    .operation
                    .for_each_atom(|atom| borrows.insert(atom, &mut lenders));
                let mut lenders = lenders
                    .into_iter()
                    .filter(|id| !used.contains(id))
                    .collect::<Vec<_>>();
                lenders.sort_by_key(|id| binding_order.get(id));
                let drops = used
                    .into_iter()
                    .chain(lenders)
                    .filter(|id| !live.contains(id))
                    .collect::<Vec<_>>();
                if !drops.is_empty() {
                    drops_after_binding.insert((StateId(state_index), binding_index), drops);
                }
                remove_pattern_bindings(&binding.pattern, &mut live);
                binding
                    .operation
                    .for_each_atom(|atom| borrows.insert(atom, &mut live));
            }
        }
        let uses = collect_use_effects(UseInputs {
            control,
            optimizations,
            calls,
            regions,
            frames,
            live_in: &live_in,
            input_destinations: &input_destinations,
            binding_destinations: &binding_destinations,
            drop_candidates: &drops_after_binding,
            borrowed_bindings: &borrowed_bindings,
            parameter_borrows: &parameter_borrows,
            borrows: &borrows,
        });
        exclude_consumed_sources(control, &uses, &mut drops_after_binding);
        for drops in drops_after_binding.values_mut() {
            drops.retain(|binding| !borrowed_bindings.contains(binding));
        }
        let drops_on_edge = collect_edge_drops(
            control,
            calls,
            frames,
            &borrows,
            &live_in,
            &uses,
            &borrowed_bindings,
        );
        let parameters = collect_parameter_effects(control, parameters, &parameter_borrows);
        Self {
            input_destinations,
            binding_destinations,
            drops_after_binding,
            drops_on_edge,
            uses,
            parameters,
            borrowed_bindings,
            owned_functions: parameter_borrows.owned.functions,
            owned_sites: parameter_borrows.owned.sites,
        }
    }

    pub(crate) fn is_valid(&self, inputs: Inputs<'_>) -> bool {
        *self == Self::new(inputs)
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

    /// How the native entry of `function` receives its managed argument.
    pub(crate) fn native_entry(&self, function: FunctionId) -> ParameterEntry {
        if self.owned_functions.contains(&function) {
            ParameterEntry::OwnedAbi
        } else {
            ParameterEntry::BorrowedAbi
        }
    }

    /// Whether the native call at `site` hands its managed argument to the callee.
    pub(crate) fn passes_owned_argument(&self, site: StateId) -> bool {
        self.owned_sites.contains(&site)
    }

    pub(crate) fn binding_is_borrowed(&self, binding: ValueId) -> bool {
        self.borrowed_bindings.contains(&binding)
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
