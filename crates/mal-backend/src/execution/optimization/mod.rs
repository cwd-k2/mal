//! Optional execution techniques represented as validated decisions rather than rewritten semantics.

use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomId, FunctionId};
use crate::control::ast::{self as control, StateId};

use super::ApplicationGraph;

mod direct_call;
mod frame_pass_through;
mod self_tail;
mod tail_forwarder;
mod unique_capture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    CallPattern,
    ContinuationSpecialization,
    NativeRecursion,
    DirectCall,
    SelfTail,
    TailForwarder,
    UniqueCapture,
    FramePassThrough,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptimizationSet(u8);

impl OptimizationSet {
    pub(crate) const fn none() -> Self {
        Self(0)
    }

    pub(crate) const fn production() -> Self {
        Self::none()
            .with(Technique::CallPattern)
            .with(Technique::ContinuationSpecialization)
            .with(Technique::NativeRecursion)
            .with(Technique::DirectCall)
            .with(Technique::SelfTail)
            .with(Technique::TailForwarder)
            .with(Technique::UniqueCapture)
            .with(Technique::FramePassThrough)
    }

    pub(crate) const fn with(self, technique: Technique) -> Self {
        Self(self.0 | (1 << technique as u8))
    }

    #[cfg(test)]
    pub(crate) const fn without(self, technique: Technique) -> Self {
        Self(self.0 & !(1 << technique as u8))
    }

    pub(crate) const fn contains(self, technique: Technique) -> bool {
        self.0 & (1 << technique as u8) != 0
    }
}

pub(crate) struct OptimizationPlan {
    fused_sites: HashSet<StateId>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
    direct_targets: HashMap<StateId, FunctionId>,
    code_pointer_free_functions: HashSet<FunctionId>,
    unique_captures: HashSet<AtomId>,
    frame_pass_through: HashMap<StateId, HashSet<crate::anf::ast::ValueId>>,
}

impl OptimizationPlan {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
        enabled: OptimizationSet,
    ) -> Self {
        let forwarded_self_arguments = if enabled.contains(Technique::TailForwarder) {
            tail_forwarder::plan(closure, control, applications)
        } else {
            HashMap::new()
        };
        let mut fused_sites = forwarded_self_arguments
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        if enabled.contains(Technique::SelfTail) {
            fused_sites.extend(self_tail::plan(control, applications));
        }
        let direct_targets = if enabled.contains(Technique::DirectCall) {
            direct_call::plan(applications)
        } else {
            HashMap::new()
        };
        let code_pointer_free_functions = if enabled.contains(Technique::DirectCall) {
            direct_call::code_pointer_free_functions(closure, applications, &direct_targets)
        } else {
            HashSet::new()
        };
        let unique_captures = if enabled.contains(Technique::UniqueCapture) {
            unique_capture::plan(closure, control, applications)
        } else {
            HashSet::new()
        };
        let frame_pass_through = if enabled.contains(Technique::FramePassThrough) {
            frame_pass_through::plan(control, applications)
        } else {
            HashMap::new()
        };
        Self {
            fused_sites,
            forwarded_self_arguments,
            direct_targets,
            code_pointer_free_functions,
            unique_captures,
            frame_pass_through,
        }
    }

    pub(crate) fn is_fused(&self, site: StateId) -> bool {
        self.fused_sites.contains(&site)
    }

    pub(crate) fn is_valid(
        &self,
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
        enabled: OptimizationSet,
    ) -> bool {
        let expected = Self::new(closure, control, applications, enabled);
        self.fused_sites == expected.fused_sites
            && self.forwarded_self_arguments == expected.forwarded_self_arguments
            && self.direct_targets == expected.direct_targets
            && self.code_pointer_free_functions == expected.code_pointer_free_functions
            && self.unique_captures == expected.unique_captures
            && self.frame_pass_through == expected.frame_pass_through
    }

    pub(super) fn forwarded_self_arguments(&self) -> &HashMap<StateId, closure::Atom> {
        &self.forwarded_self_arguments
    }

    pub(crate) fn direct_target(&self, site: StateId) -> Option<FunctionId> {
        self.direct_targets.get(&site).copied()
    }

    pub(crate) fn omits_code_pointer(&self, function: FunctionId) -> bool {
        self.code_pointer_free_functions.contains(&function)
    }

    pub(crate) fn takes_unique_capture(&self, atom: AtomId) -> bool {
        self.unique_captures.contains(&atom)
    }

    pub(crate) fn frame_pass_through(
        &self,
        site: StateId,
    ) -> Option<&HashSet<crate::anf::ast::ValueId>> {
        self.frame_pass_through.get(&site)
    }
}

#[cfg(test)]
mod tests;
