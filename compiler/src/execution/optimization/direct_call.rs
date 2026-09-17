use crate::closure::ast::FunctionId;
use std::collections::HashMap;

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
