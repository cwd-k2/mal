use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, Terminator};

use super::control_call::reachable_states;
use super::{ControlCallMode, ControlCallPlan, ControlRegionPlan};

pub(in crate::c_emit::body) struct CommonControlPlan {
    functions: HashSet<FunctionId>,
}

impl CommonControlPlan {
    pub(in crate::c_emit::body) fn new(
        program: &Program,
        regions: &ControlRegionPlan,
        calls: &ControlCallPlan,
    ) -> Self {
        let mut functions = HashSet::new();
        for region in regions.ids() {
            let requires_common = regions.functions(region).iter().any(|function| {
                let entry = program
                    .functions
                    .iter()
                    .find(|candidate| candidate.id == *function)
                    .expect("region function has a control entry")
                    .entry;
                reachable_states(program, entry).into_iter().any(|site| {
                    regions.site_region(site) == Some(region)
                        && calls.mode(site) == Some(ControlCallMode::Dispatch)
                        && !is_direct_self_call(&program.states[site.0].terminator, *function)
                })
            });
            if requires_common {
                functions.extend(regions.functions(region));
            }
        }
        Self { functions }
    }

    pub(in crate::c_emit::body) fn contains(&self, function: FunctionId) -> bool {
        self.functions.contains(&function)
    }
}

fn is_direct_self_call(terminator: &Terminator, caller: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::Call {
            callee:
                crate::closure::ast::Atom {
                    kind: crate::closure::ast::AtomKind::Reference(
                        crate::closure::ast::Reference::SelfClosure(target)
                    ),
                    ..
                },
            ..
        } if *target == caller
    )
}
