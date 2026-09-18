use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::super::{
    ApplicationGraph, ControlCallMode, ControlCallPlan, ControlRegionPlan, ParameterDestination,
    ParameterPlan,
};
use super::managed::is_managed;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ParameterEntry {
    BorrowedAbi,
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
}

impl ParameterBorrows {
    pub(super) fn new(
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
        regions: &ControlRegionPlan,
    ) -> Self {
        let mut functions = control
            .functions
            .iter()
            .filter(|function| {
                regions.function_region(function.id).is_none()
                    && control.states[function.entry.0].input.is_none()
                    && (!super::super::call::reachable_states(control, function.entry)
                        .into_iter()
                        .any(|site| calls.mode(site) == Some(ControlCallMode::DirectSelfTail))
                        || parameter_is_bounded(control, function.id))
            })
            .map(|function| function.id)
            .collect::<HashSet<_>>();
        for region in regions.ids() {
            if regions
                .functions(region)
                .iter()
                .all(|function| parameter_is_bounded(control, *function))
                && control.states.iter().enumerate().all(|(index, _)| {
                    let site = StateId(index);
                    regions.site_region(site) != Some(region)
                        || applications.targets(site).is_none_or(|targets| {
                            targets
                                .iter()
                                .all(|target| regions.function_region(*target) == Some(region))
                        })
                })
            {
                functions.extend(regions.functions(region));
            }
        }
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
        }
    }
}

fn parameter_is_bounded(control: &Program, function: FunctionId) -> bool {
    let function = control
        .functions
        .iter()
        .find(|candidate| candidate.id == function)
        .expect("control function");
    super::super::call::reachable_states(control, function.entry)
        .into_iter()
        .all(|site| {
            let state = &control.states[site.0];
            let bindings_are_bounded = state.bindings.iter().all(|binding| {
                !pattern_is_managed(&binding.pattern)
                    || matches!(
                        binding.operation,
                        Operation::Atom(_) | Operation::Product(_) | Operation::SumInjection { .. }
                    )
            });
            let result_is_bounded = !matches!(
                &state.terminator,
                Terminator::Return(value) if is_managed(&value.ty)
            );
            let call_result_is_bounded = match &state.terminator {
                Terminator::Call { resume, .. } => control.states[resume.0]
                    .input
                    .as_ref()
                    .is_none_or(|pattern| !pattern_is_managed(pattern)),
                _ => true,
            };
            bindings_are_bounded && result_is_bounded && call_result_is_bounded
        })
}

fn pattern_is_managed(pattern: &crate::closure::ast::Pattern) -> bool {
    match pattern {
        crate::closure::ast::Pattern::Binding { ty, .. }
        | crate::closure::ast::Pattern::Product { ty, .. }
        | crate::closure::ast::Pattern::Wildcard { ty, .. } => is_managed(ty),
    }
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
    }
    effects
}
