use std::collections::{HashMap, HashSet};

use crate::closure::ast::FunctionId;

mod analysis;
mod shape;

pub(super) use shape::{contains_buffer, has_single_buffer};

pub(super) fn plan(execution: &crate::execution::Program) -> HashSet<FunctionId> {
    let functions = execution
        .control
        .functions
        .iter()
        .map(|function| (function.id, function))
        .collect::<HashMap<_, _>>();
    let mut stable = functions
        .keys()
        .copied()
        .filter(|function| analysis::has_no_relocation(execution, functions[function].entry))
        .collect::<HashSet<_>>();
    analysis::close_stability_over_calls(execution, &mut stable);

    let mut direct = execution
        .lowered
        .functions
        .iter()
        .filter(|function| {
            stable.contains(&function.id)
                && shape::buffer_parameter_is_supported(&function.parameter.ty)
                && (shape::has_single_buffer(&function.parameter.ty)
                    || analysis::has_no_control_frame(execution, functions[&function.id].entry))
                && !contains_buffer(&function.body.result.ty)
                && !analysis::captures_buffer_in_nested_closure(
                    execution,
                    functions[&function.id].entry,
                )
                && function.kind.captures().is_some_and(|captures| {
                    captures.iter().all(|capture| !contains_buffer(&capture.ty))
                })
        })
        .map(|function| function.id)
        .collect::<HashSet<_>>();

    loop {
        let before = direct.len();
        analysis::close_regions(execution, &mut direct);
        analysis::close_buffer_calls(execution, &mut direct);
        if direct.len() == before {
            return direct;
        }
    }
}

#[cfg(test)]
mod tests;
