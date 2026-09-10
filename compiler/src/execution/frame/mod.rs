use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::control::ast::{self as control, LiveValue, StateId, Terminator};

use super::ownership::is_managed;
use super::{ClosureUsePlan, ControlCallPlan, ControlRegionId, ControlRegionPlan};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ControlArenaId(pub(crate) usize);

pub(crate) struct ControlFramePlan {
    frames: HashMap<StateId, ControlFrame>,
    closures_crossing_suspension: HashSet<ValueId>,
    region_arenas: HashMap<ControlRegionId, ControlArenaId>,
    homogeneous_regions: HashMap<ControlRegionId, StateId>,
}

#[derive(Clone)]
pub(crate) struct ControlFrame {
    pub(crate) resume: StateId,
    pub(crate) fields: Vec<ControlFrameField>,
    pub(crate) carries_environment: bool,
}

#[derive(Clone)]
pub(crate) struct ControlFrameField {
    pub(crate) value: LiveValue,
    pub(crate) managed: bool,
}

impl ControlFramePlan {
    pub(crate) fn new(
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        closure_uses: &ClosureUsePlan,
    ) -> Self {
        let mut frames = HashMap::new();
        let mut closures_crossing_suspension = HashSet::new();
        for (index, state) in program.states.iter().enumerate() {
            let site = StateId(index);
            let Some(region) = regions.site_region(site) else {
                continue;
            };
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
                            managed: is_managed(&value.ty),
                        })
                        .collect(),
                    carries_environment: resume_state.needs_environment
                        && calls.requires_common_control(region),
                },
            );
        }
        let region_arenas = regions
            .ids()
            .filter(|region| {
                frames
                    .keys()
                    .any(|site| regions.site_region(*site) == Some(*region))
            })
            .enumerate()
            .map(|(arena, region)| (region, ControlArenaId(arena)))
            .collect();
        let homogeneous_regions = regions
            .ids()
            .filter_map(|region| {
                let mut sites = frames
                    .keys()
                    .filter(|site| regions.site_region(**site) == Some(region));
                let site = *sites.next()?;
                sites.next().is_none().then_some((region, site))
            })
            .collect();
        Self {
            frames,
            closures_crossing_suspension,
            region_arenas,
            homogeneous_regions,
        }
    }

    pub(crate) fn frame(&self, site: StateId) -> Option<&ControlFrame> {
        self.frames.get(&site)
    }

    pub(crate) fn closure_crosses_suspension(&self, id: ValueId) -> bool {
        self.closures_crossing_suspension.contains(&id)
    }

    pub(crate) fn arena_count(&self) -> usize {
        self.region_arenas.len()
    }

    pub(crate) fn has_homogeneous_arenas(&self) -> bool {
        !self.homogeneous_regions.is_empty()
    }

    pub(crate) fn has_heterogeneous_arenas(&self) -> bool {
        self.region_arenas
            .keys()
            .any(|region| !self.homogeneous_regions.contains_key(region))
    }

    pub(crate) fn arena(&self, region: ControlRegionId) -> Option<ControlArenaId> {
        self.region_arenas.get(&region).copied()
    }

    pub(crate) fn homogeneous_frame(&self, region: ControlRegionId) -> Option<StateId> {
        self.homogeneous_regions.get(&region).copied()
    }

    pub(crate) fn frame_is_homogeneous(&self, site: StateId) -> bool {
        self.homogeneous_regions
            .values()
            .any(|candidate| *candidate == site)
    }

    pub(crate) fn is_valid(
        &self,
        program: &control::Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        closure_uses: &ClosureUsePlan,
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

        let frames_match_resume_live_ins = self.frames.iter().all(|(site, frame)| {
            matches!(
                program.states[site.0].terminator,
                Terminator::Call { resume, .. } if resume == frame.resume
            ) && frame.fields.len() == program.states[frame.resume.0].live.len()
                && frame
                    .fields
                    .iter()
                    .zip(&program.states[frame.resume.0].live)
                    .all(|(field, live)| {
                        field.value == *live && field.managed == is_managed(&live.ty)
                    })
                && frame.carries_environment
                    == (program.states[frame.resume.0].needs_environment
                        && regions
                            .site_region(*site)
                            .is_some_and(|region| calls.requires_common_control(region)))
        });
        let expected_closures = expected_sites
            .iter()
            .flat_map(|site| {
                let Terminator::Call { resume, .. } = program.states[site.0].terminator else {
                    unreachable!("expected frame sites are non-tail calls")
                };
                program.states[resume.0].live.iter()
            })
            .filter_map(|value| closure_uses.direct_closure(value.id))
            .map(|closure| closure.creator)
            .collect::<HashSet<_>>();
        let expected_arenas = regions
            .ids()
            .filter(|region| {
                expected_sites
                    .iter()
                    .any(|site| regions.site_region(*site) == Some(*region))
            })
            .enumerate()
            .map(|(arena, region)| (region, ControlArenaId(arena)))
            .collect::<HashMap<_, _>>();
        let expected_homogeneous_regions = regions
            .ids()
            .filter_map(|region| {
                let mut sites = expected_sites
                    .iter()
                    .filter(|site| regions.site_region(**site) == Some(region));
                let site = *sites.next()?;
                sites.next().is_none().then_some((region, site))
            })
            .collect::<HashMap<_, _>>();

        frames_match_resume_live_ins
            && self.closures_crossing_suspension == expected_closures
            && self.region_arenas == expected_arenas
            && self.homogeneous_regions == expected_homogeneous_regions
    }
}

#[cfg(test)]
mod tests;
