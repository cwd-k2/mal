use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

use super::ControlRegionPlan;
use super::control_call::reachable_states;

pub(in crate::c_emit::body) struct CommonControlPlan {
    functions: HashSet<FunctionId>,
}

impl CommonControlPlan {
    pub(in crate::c_emit::body) fn new(regions: &ControlRegionPlan) -> Self {
        Self {
            functions: regions.common_functions().collect(),
        }
    }

    pub(in crate::c_emit::body) fn contains(&self, function: FunctionId) -> bool {
        self.functions.contains(&function)
    }

    pub(in crate::c_emit::body) fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    pub(in crate::c_emit::body) fn contains_state(&self, program: &Program, site: StateId) -> bool {
        program.functions.iter().any(|function| {
            self.contains(function.id) && reachable_states(program, function.entry).contains(&site)
        })
    }
}
