use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::StateId;

pub(in crate::backend::llvm::optimization) fn plan(
    execution: &crate::execution::Program,
) -> HashSet<FunctionId> {
    let framed_regions = execution
        .control
        .states
        .iter()
        .enumerate()
        .filter_map(|(index, _)| {
            let site = StateId(index);
            execution
                .control_frames
                .frame(site)
                .and_then(|_| execution.control_regions.site_region(site))
        })
        .collect::<HashSet<_>>();

    execution
        .control
        .functions
        .iter()
        .filter_map(|function| {
            execution
                .control_regions
                .function_region(function.id)
                .is_some_and(|region| framed_regions.contains(&region))
                .then_some(function.id)
        })
        .collect()
}
