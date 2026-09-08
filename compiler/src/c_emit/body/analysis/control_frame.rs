use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::control::ast::{self as control, LiveValue, StateId, Terminator};

use super::ClosureUsePlan;
use super::control_call::{ControlCallMode, ControlCallPlan};
use crate::c_emit::types::TypeRegistry;

pub(in crate::c_emit::body) struct ControlFramePlan {
    frames: HashMap<StateId, ControlFrame>,
    closures_crossing_suspension: HashSet<ValueId>,
}

pub(in crate::c_emit::body) struct ControlFrame {
    pub(in crate::c_emit::body) resume: StateId,
    pub(in crate::c_emit::body) fields: Vec<ControlFrameField>,
    pub(in crate::c_emit::body) needs_environment: bool,
}

pub(in crate::c_emit::body) struct ControlFrameField {
    pub(in crate::c_emit::body) value: LiveValue,
    pub(in crate::c_emit::body) managed: bool,
}

impl ControlFramePlan {
    pub(in crate::c_emit::body) fn new(
        program: &control::Program,
        calls: &ControlCallPlan,
        types: &TypeRegistry,
        closure_uses: &ClosureUsePlan,
    ) -> Self {
        let mut frames = HashMap::new();
        let mut closures_crossing_suspension = HashSet::new();
        for (index, state) in program.states.iter().enumerate() {
            let site = StateId(index);
            if calls.mode(site) != Some(ControlCallMode::Dispatch) {
                continue;
            }
            let Terminator::Call { resume, .. } = state.terminator else {
                continue;
            };
            let resume_state = &program.states[resume.0];
            for value in &resume_state.live {
                if let Some(closure) = closure_uses.direct_closure(value.id) {
                    closures_crossing_suspension.insert(closure.creator);
                }
            }
            frames.insert(
                site,
                ControlFrame {
                    resume,
                    fields: resume_state
                        .live
                        .iter()
                        .map(|value| ControlFrameField {
                            value: value.clone(),
                            managed: types.contains_managed(&value.ty),
                        })
                        .collect(),
                    needs_environment: resume_state.needs_environment,
                },
            );
        }
        Self {
            frames,
            closures_crossing_suspension,
        }
    }

    pub(in crate::c_emit::body) fn frame(&self, site: StateId) -> Option<&ControlFrame> {
        self.frames.get(&site)
    }

    pub(in crate::c_emit::body) fn closure_crosses_suspension(&self, id: ValueId) -> bool {
        self.closures_crossing_suspension.contains(&id)
    }

    pub(in crate::c_emit::body) fn is_valid(
        &self,
        program: &control::Program,
        types: &TypeRegistry,
    ) -> bool {
        self.frames.iter().all(|(site, frame)| {
            matches!(
                program.states[site.0].terminator,
                Terminator::Call { resume, .. } if resume == frame.resume
            ) && frame.fields.len() == program.states[frame.resume.0].live.len()
                && frame
                    .fields
                    .iter()
                    .zip(&program.states[frame.resume.0].live)
                    .all(|(field, live)| {
                        field.value == *live && field.managed == types.contains_managed(&live.ty)
                    })
                && frame.needs_environment == program.states[frame.resume.0].needs_environment
        })
    }
}
