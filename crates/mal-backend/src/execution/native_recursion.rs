//! Functions that also get a native version for their recursion.
//!
//! A region function that recurses only into itself can run its recursion on the native stack while the stack has
//! room: the native version nests a native call for each recursive call and hands the activation to the frames
//! version, which keeps suspended callers in the control arena, once the native budget is used up. The frames
//! version always exists, so a region this plan does not admit simply keeps running on frames.

use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId, Terminator};

use super::optimization::{OptimizationSet, Technique};
use super::{ControlCallMode, ControlCallPlan, ControlFramePlan, ControlRegionPlan};

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct NativeRecursionPlan {
    functions: HashSet<FunctionId>,
}

impl NativeRecursionPlan {
    pub(crate) fn new(
        control: &Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
        frames: &ControlFramePlan,
        enabled: OptimizationSet,
    ) -> Self {
        if !enabled.contains(Technique::NativeRecursion) {
            return Self::default();
        }
        let functions = control
            .functions
            .iter()
            .map(|function| function.id)
            .filter(|function| {
                let Some(region) = regions.function_region(*function) else {
                    return false;
                };
                if regions.functions(region) != [*function] || calls.requires_common_control(region)
                {
                    return false;
                }
                let sites = (0..control.states.len())
                    .map(StateId)
                    .filter(|site| {
                        frames.frame(*site).is_some() && regions.site_region(*site) == Some(region)
                    })
                    .collect::<Vec<_>>();
                !sites.is_empty()
                    && sites.iter().all(|site| {
                        matches!(control.states[site.0].terminator, Terminator::Call { .. })
                            && calls.mode(*site) == Some(ControlCallMode::DirectRegion(*function))
                    })
            })
            .collect();
        Self { functions }
    }

    pub(crate) fn has_native_version(&self, function: FunctionId) -> bool {
        self.functions.contains(&function)
    }
}
