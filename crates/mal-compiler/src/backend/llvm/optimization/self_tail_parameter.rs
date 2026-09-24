use std::collections::HashSet;

use crate::closure::ast::FunctionId;

pub(super) fn plan(execution: &crate::execution::Program) -> HashSet<FunctionId> {
    execution.self_tail_parameters.functions().collect()
}
