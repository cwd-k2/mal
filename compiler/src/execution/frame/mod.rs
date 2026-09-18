use std::collections::{HashMap, HashSet};

use crate::control::ast::{self as control, LiveValue, StateId, Terminator};

use super::{ControlCallPlan, ControlRegionPlan};

mod replacement;
mod resume;

pub(crate) use resume::FrameResume;

pub(crate) struct ControlFramePlan {
    frames: HashMap<StateId, ControlFrame>,
    resumes: resume::Plan,
    replacements: replacement::Plan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
        let resumes = resume::Plan::new(program, regions, calls, &frames);
        let replacements = replacement::Plan::new(program, &frames);
        Self {
            frames,
            resumes,
            replacements,
        }
    }

    pub(crate) fn frame(&self, site: StateId) -> Option<&ControlFrame> {
        self.frames.get(&site)
    }

    pub(crate) fn resume(&self, exit: StateId, frame: StateId) -> Option<FrameResume> {
        self.resumes.disposition(exit, frame)
    }

    pub(crate) fn replacement(&self, site: StateId) -> Option<StateId> {
        self.replacements.retired_frame(site)
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

        self.resumes.is_valid(program, regions, calls, &self.frames)
            && self.replacements.is_valid(program, &self.frames)
            && self.frames.iter().all(|(site, frame)| {
                let Terminator::Call { callee, resume, .. } = &program.states[site.0].terminator
                else {
                    return false;
                };
                let crate::check::ast::Type::Function { result, .. } = &callee.ty else {
                    return false;
                };
                *resume == frame.resume
                    && matches!(
                        calls.mode(*site),
                        Some(
                            super::ControlCallMode::DirectRegion(_)
                                | super::ControlCallMode::Dispatch
                        )
                    )
                    && program.states[frame.resume.0]
                        .input
                        .as_ref()
                        .is_some_and(|input| pattern_type(input) == result.as_ref())
                    && frame.fields.len() == program.states[frame.resume.0].live.len()
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

fn pattern_type(pattern: &crate::closure::ast::Pattern) -> &crate::check::ast::Type {
    match pattern {
        crate::closure::ast::Pattern::Binding { ty, .. }
        | crate::closure::ast::Pattern::Wildcard { ty, .. }
        | crate::closure::ast::Pattern::Product { ty, .. } => ty,
    }
}

#[cfg(test)]
mod tests;
