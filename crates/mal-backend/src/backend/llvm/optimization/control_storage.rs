use std::collections::HashSet;

use crate::closure::ast::FunctionId;

pub(super) fn plan(execution: &crate::execution::Program) -> HashSet<FunctionId> {
    super::control_top::plan(execution)
}
