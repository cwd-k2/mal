use std::collections::{HashMap, HashSet};

use crate::control::ast::{self as control, LiveValue, StateId, Terminator};

use super::{ControlCallPlan, ControlRegionPlan};

pub(crate) struct ControlFramePlan {
    frames: HashMap<StateId, ControlFrame>,
}

#[derive(Clone)]
pub(crate) struct ControlFrame {
    pub(crate) resume: StateId,
    pub(crate) fields: Vec<LiveValue>,
    pub(crate) carries_environment: bool,
}

impl ControlFramePlan {
    pub(crate) fn new(
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
    ) -> Self {
        let mut frames = HashMap::new();
        for (index, state) in program.states.iter().enumerate() {
            let site = StateId(index);
            let Some(region) = regions.site_region(site) else {
                continue;
            };
            let Terminator::Call { resume, .. } = state.terminator else {
                continue;
            };
            let resume_state = &program.states[resume.0];
            frames.insert(
                site,
                ControlFrame {
                    resume,
                    fields: resume_state.live.clone(),
                    carries_environment: resume_state.needs_environment
                        && calls.requires_common_control(region),
                },
            );
        }
        Self { frames }
    }

    pub(crate) fn frame(&self, site: StateId) -> Option<&ControlFrame> {
        self.frames.get(&site)
    }

    pub(crate) fn is_valid(
        &self,
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
    ) -> bool {
        let expected_sites = program
            .states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                let site = StateId(index);
                (regions.site_region(site).is_some()
                    && matches!(state.terminator, Terminator::Call { .. }))
                .then_some(site)
            })
            .collect::<HashSet<_>>();
        let frame_sites = self.frames.keys().copied().collect::<HashSet<_>>();
        if frame_sites != expected_sites {
            return false;
        }

        self.frames.iter().all(|(site, frame)| {
            matches!(
                program.states[site.0].terminator,
                Terminator::Call { resume, .. } if resume == frame.resume
            ) && frame.fields.len() == program.states[frame.resume.0].live.len()
                && frame
                    .fields
                    .iter()
                    .zip(&program.states[frame.resume.0].live)
                    .all(|(field, live)| field == live)
                && frame.carries_environment
                    == (program.states[frame.resume.0].needs_environment
                        && regions
                            .site_region(*site)
                            .is_some_and(|region| calls.requires_common_control(region)))
        })
    }
}

#[cfg(test)]
mod tests;
