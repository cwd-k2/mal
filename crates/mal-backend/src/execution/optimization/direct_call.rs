use crate::closure::ast::FunctionId;
use std::collections::{HashMap, HashSet};

use super::super::ApplicationGraph;

pub(super) fn plan(
    applications: &ApplicationGraph,
) -> HashMap<crate::control::ast::StateId, FunctionId> {
    applications
        .sites()
        .filter_map(|(site, _)| {
            applications
                .direct_target(site)
                .or_else(|| match applications.targets(site) {
                    Some([target]) => Some(*target),
                    _ => None,
                })
                .map(|target| (site, target))
        })
        .collect()
}

/// Functions whose code address is never needed by an indirect application.
///
/// A closure may still carry captures and participate in ordinary ownership, but function values
/// have no equality or representation observation. Once every application that can receive a
/// function is selected as a direct call to that function, its carrier does not need the code
/// pointer. Keeping this decision beside direct call selection avoids reconstructing closure flow
/// in the LLVM backend.
pub(super) fn code_pointer_free_functions(
    closure: &crate::closure::ast::Program,
    applications: &ApplicationGraph,
    direct_targets: &HashMap<crate::control::ast::StateId, FunctionId>,
) -> HashSet<FunctionId> {
    closure
        .functions
        .iter()
        .map(|function| function.id)
        .filter(|function| {
            applications.sites().all(|(site, _)| {
                applications.targets(site).is_none_or(|targets| {
                    !targets.contains(function) || direct_targets.get(&site) == Some(function)
                })
            })
        })
        .collect()
}
