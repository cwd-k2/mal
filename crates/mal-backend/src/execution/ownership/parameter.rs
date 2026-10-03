use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::super::{
    ApplicationGraph, ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan,
    ParameterDestination, ParameterPlan, SelfTailParameterPlan,
};
use super::convention::OwnedConvention;
use super::lifecycle::is_managed;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ParameterEntry {
    /// The native entry of a function whose caller keeps the argument.
    BorrowedAbi,
    /// The native entry of a function whose caller hands the argument over.
    OwnedAbi,
    /// A transition inside a region or a self-tail edge.
    OwnedHandoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterEffect {
    BorrowInto(ValueId),
    ShareInto(ValueId),
    ConsumeInto(ValueId),
    Drop,
}

pub(super) struct ParameterBorrows {
    pub(super) functions: HashSet<FunctionId>,
    pub(super) bindings: HashSet<ValueId>,
    pub(super) call_sites: HashSet<StateId>,
    pub(super) owned: OwnedConvention,
}

impl ParameterBorrows {
    pub(super) fn new(
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
        regions: &ControlRegionPlan,
        frames: &ControlFramePlan,
        native_recursion: &super::super::NativeRecursionPlan,
        self_tail_parameters: &SelfTailParameterPlan,
    ) -> Self {
        let mut functions = control
            .functions
            .iter()
            .filter(|function| {
                regions.function_region(function.id).is_none()
                    && control.states[function.entry.0].input.is_none()
                    && (!function
                        .states
                        .iter()
                        .any(|site| calls.mode(*site) == Some(ControlCallMode::DirectSelfTail))
                        || parameter_is_bounded(control, function.id))
            })
            .map(|function| function.id)
            .collect::<HashSet<_>>();
        for region in regions.ids() {
            if regions
                .functions(region)
                .iter()
                .all(|function| parameter_is_bounded(control, *function))
                && regions.sites(region).iter().all(|site| {
                    applications.targets(*site).is_none_or(|targets| {
                        targets
                            .iter()
                            .all(|target| regions.function_region(*target) == Some(region))
                    })
                })
            {
                functions.extend(regions.functions(region));
            }
        }
        functions.extend(
            control
                .functions
                .iter()
                .map(|function| function.id)
                .filter(|function| native_recursion.borrows_parameter(*function)),
        );
        let self_tail_borrows = control
            .functions
            .iter()
            .filter_map(|function| {
                let admitted = self_tail_parameters.get(function.id).is_some()
                    && parameter_is_bounded(control, function.id)
                    && applications.sites().all(|(site, _)| {
                        !applications
                            .targets(site)
                            .is_some_and(|targets| targets.contains(&function.id))
                            || match calls.mode(site) {
                                Some(ControlCallMode::DirectSelfTail) => true,
                                Some(ControlCallMode::Direct(target)) if target == function.id => {
                                    frames.frame(site).is_none()
                                }
                                _ => false,
                            }
                    });
                admitted.then_some(function.id)
            })
            .collect::<HashSet<_>>();
        let mut owned = OwnedConvention::new(control, applications, calls, regions, frames);
        // A frame-free caller remains the lender for a bounded self-tail invocation.
        // OwnedConvention cannot infer that persistent authority from parameter use alone.
        owned
            .functions
            .retain(|function| !self_tail_borrows.contains(function));
        owned.sites.retain(|site| {
            !applications.targets(*site).is_some_and(|targets| {
                !targets.is_empty()
                    && targets
                        .iter()
                        .all(|target| self_tail_borrows.contains(target))
            })
        });
        functions.retain(|function| !owned.functions.contains(function));
        functions.extend(self_tail_borrows);
        let bindings = control
            .functions
            .iter()
            .filter(|function| functions.contains(&function.id))
            .filter(|function| is_managed(&function.parameter.ty))
            .filter_map(|function| function.parameter.binding)
            .collect();
        let call_sites = control
            .states
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let site = StateId(index);
                let targets = applications.targets(site)?;
                let borrowed = match calls.mode(site)? {
                    ControlCallMode::Direct(target) | ControlCallMode::DirectRegion(target) => {
                        functions.contains(&target)
                    }
                    ControlCallMode::DirectSelfTail | ControlCallMode::Dispatch => {
                        targets.iter().all(|target| functions.contains(target))
                    }
                };
                borrowed.then_some(site)
            })
            .collect();
        Self {
            functions,
            bindings,
            call_sites,
            owned,
        }
    }
}

fn parameter_is_bounded(control: &Program, function: FunctionId) -> bool {
    let function = control
        .functions
        .iter()
        .find(|candidate| candidate.id == function)
        .expect("control function");
    function.states.iter().all(|&site| {
        let state = &control.states[site.0];
        let bindings_are_bounded = state.bindings.iter().all(|binding| {
            !is_managed(binding.pattern.ty())
                || match &binding.operation {
                    Operation::Atom(_) | Operation::Product(_) | Operation::SumInjection { .. } => {
                        true
                    }
                    Operation::MakeClosure { captures, .. } => captures.is_empty(),
                    _ => false,
                }
        });
        let result_is_bounded = !matches!(
            &state.terminator,
            Terminator::Return(value) if is_managed(&value.ty)
        );
        let call_result_is_bounded = match &state.terminator {
            Terminator::Call { resume, .. } => control.states[resume.0]
                .input
                .as_ref()
                .is_none_or(|pattern| !is_managed(pattern.ty())),
            _ => true,
        };
        bindings_are_bounded && result_is_bounded && call_result_is_bounded
    })
}

pub(super) fn collect_parameter_effects(
    control: &crate::control::ast::Program,
    parameters: &ParameterPlan,
    borrows: &ParameterBorrows,
) -> HashMap<(FunctionId, ParameterEntry), ParameterEffect> {
    let mut effects = HashMap::new();
    for function in &control.functions {
        if !is_managed(&function.parameter.ty) {
            continue;
        }
        if borrows.owned.functions.contains(&function.id) {
            let effect = match parameters.destination(function.id) {
                Some(ParameterDestination::Bind(binding))
                    if control.states[function.entry.0]
                        .live
                        .iter()
                        .any(|value| value.id == binding) =>
                {
                    ParameterEffect::ConsumeInto(binding)
                }
                _ => ParameterEffect::Drop,
            };
            effects.insert((function.id, ParameterEntry::OwnedAbi), effect);
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
                if borrows.bindings.contains(&binding) {
                    effects.insert(
                        (function.id, ParameterEntry::BorrowedAbi),
                        ParameterEffect::BorrowInto(binding),
                    );
                    effects.insert(
                        (function.id, ParameterEntry::OwnedHandoff),
                        ParameterEffect::BorrowInto(binding),
                    );
                } else {
                    effects.insert(
                        (function.id, ParameterEntry::BorrowedAbi),
                        ParameterEffect::ShareInto(binding),
                    );
                    effects.insert(
                        (function.id, ParameterEntry::OwnedHandoff),
                        ParameterEffect::ConsumeInto(binding),
                    );
                }
            }
            ParameterDestination::Bind(_) | ParameterDestination::Discard
                if !borrows.functions.contains(&function.id) =>
            {
                effects.insert(
                    (function.id, ParameterEntry::OwnedHandoff),
                    ParameterEffect::Drop,
                );
            }
            ParameterDestination::Bind(_) | ParameterDestination::Discard => {}
        }
        if borrows.owned.functions.contains(&function.id) {
            effects.remove(&(function.id, ParameterEntry::BorrowedAbi));
        }
    }
    effects
}
