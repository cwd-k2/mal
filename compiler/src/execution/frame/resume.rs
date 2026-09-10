use std::collections::{HashMap, HashSet};

use crate::check::ast::Type;
use crate::closure::ast::{FunctionId, Pattern};
use crate::control::ast::{self as control, StateId, Terminator};

use super::ControlFrame;
use crate::execution::{ControlCallMode, ControlCallPlan, ControlRegionPlan};

pub(super) struct Plan {
    pub(super) pairs: HashSet<(StateId, StateId)>,
    pub(super) compatible: HashSet<(StateId, StateId)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FrameResume {
    Resume,
    Unreachable,
}

impl Plan {
    pub(super) fn new(
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        frames: &HashMap<StateId, ControlFrame>,
    ) -> Self {
        let state_functions = program
            .functions
            .iter()
            .flat_map(|function| {
                crate::execution::application::reachable_states(program, function.entry)
                    .into_iter()
                    .map(|site| (site, function.id))
            })
            .collect::<HashMap<_, _>>();
        let mut pairs = HashSet::new();
        let mut compatible = HashSet::new();
        for index in 0..program.states.len() {
            let exit = StateId(index);
            let Some(result) = continuation_result_type(program, calls, frames, exit) else {
                continue;
            };
            for (site, frame) in frames {
                if !same_machine(exit, *site, &state_functions, regions, calls) {
                    continue;
                }
                pairs.insert((exit, *site));
                let Some(input) = program.states[frame.resume.0].input.as_ref() else {
                    continue;
                };
                if result == pattern_type(input) {
                    compatible.insert((exit, *site));
                }
            }
        }
        Self { pairs, compatible }
    }

    pub(super) fn disposition(&self, exit: StateId, frame: StateId) -> Option<FrameResume> {
        self.pairs.contains(&(exit, frame)).then(|| {
            if self.compatible.contains(&(exit, frame)) {
                FrameResume::Resume
            } else {
                FrameResume::Unreachable
            }
        })
    }

    pub(super) fn is_valid(
        &self,
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        frames: &HashMap<StateId, ControlFrame>,
    ) -> bool {
        let expected = Self::new(program, regions, calls, frames);
        self.pairs == expected.pairs && self.compatible == expected.compatible
    }
}

fn same_machine(
    exit: StateId,
    frame: StateId,
    state_functions: &HashMap<StateId, FunctionId>,
    regions: &ControlRegionPlan,
    calls: &ControlCallPlan,
) -> bool {
    let Some(exit_function) = state_functions.get(&exit) else {
        return false;
    };
    let Some(frame_function) = state_functions.get(&frame) else {
        return false;
    };
    let Some(region) = regions.site_region(frame) else {
        return false;
    };
    if calls.requires_common_control(region) {
        regions.function_region(*exit_function) == Some(region)
    } else {
        exit_function == frame_function
    }
}

fn continuation_result_type<'a>(
    program: &'a control::Program,
    calls: &ControlCallPlan,
    frames: &HashMap<StateId, ControlFrame>,
    site: StateId,
) -> Option<&'a Type> {
    let terminator = &program.states[site.0].terminator;
    match terminator {
        Terminator::Return(value) => Some(&value.ty),
        Terminator::Call { callee, .. }
            if calls.mode(site) == Some(ControlCallMode::Dispatch)
                && frames.contains_key(&site) =>
        {
            let Type::Function { result, .. } = &callee.ty else {
                return None;
            };
            Some(result)
        }
        Terminator::TailCall { callee, .. }
            if matches!(
                calls.mode(site),
                Some(ControlCallMode::Direct(_) | ControlCallMode::Dispatch)
            ) =>
        {
            let Type::Function { result, .. } = &callee.ty else {
                return None;
            };
            Some(result)
        }
        _ => None,
    }
}

fn pattern_type(pattern: &Pattern) -> &Type {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => ty,
    }
}
