use std::collections::HashSet;

use crate::closure::ast::FunctionId;

use super::ControlRegionPlan;

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
}
