use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;
use crate::control::ast::{self as control, StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::ControlFrame;
use crate::execution::{ControlCallMode, ControlCallPlan, ControlRegionId, ControlRegionPlan};

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
            .flat_map(|function| function.states.iter().map(|site| (*site, function.id)))
            .collect::<HashMap<_, _>>();
        let mut machines = HashMap::<Machine, Vec<StateId>>::new();
        for site in frames.keys() {
            if let Some(machine) = frame_machine(*site, &state_functions, regions, calls) {
                machines.entry(machine).or_default().push(*site);
            }
        }
        let mut pairs = HashSet::new();
        let mut compatible = HashSet::new();
        for index in 0..program.states.len() {
            let exit = StateId(index);
            let Some(result) = continuation_result_type(program, calls, frames, exit) else {
                continue;
            };
            let Some(exit_function) = state_functions.get(&exit) else {
                continue;
            };
            let common = regions
                .function_region(*exit_function)
                .filter(|region| calls.requires_common_control(*region))
                .map(Machine::Common);
            for machine in [Some(Machine::Local(*exit_function)), common]
                .into_iter()
                .flatten()
            {
                for site in machines.get(&machine).into_iter().flatten() {
                    pairs.insert((exit, *site));
                    let Some(input) = program.states[frames[site].resume.0].input.as_ref() else {
                        continue;
                    };
                    if result == input.ty() {
                        compatible.insert((exit, *site));
                    }
                }
            }
        }
        Self { pairs, compatible }
    }

    /// Classifies an exit against a suspended frame in the same control machine.
    ///
    /// `None` means the pair belongs to different machines. `Unreachable` means the machine is
    /// shared but the result and resume-input types differ; `Resume` means both machine and type
    /// are compatible.
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

/// The control machine an exit must belong to in order to resume a frame.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Machine {
    /// A common-control region forms one machine across function boundaries.
    Common(ControlRegionId),
    /// A local region is confined to the function that owns the frame.
    Local(FunctionId),
}

fn frame_machine(
    frame: StateId,
    state_functions: &HashMap<StateId, FunctionId>,
    regions: &ControlRegionPlan,
    calls: &ControlCallPlan,
) -> Option<Machine> {
    let frame_function = state_functions.get(&frame)?;
    let region = regions.site_region(frame)?;
    Some(if calls.requires_common_control(region) {
        Machine::Common(region)
    } else {
        Machine::Local(*frame_function)
    })
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
