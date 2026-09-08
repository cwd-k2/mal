use std::collections::HashSet;

use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId, Terminator};

use super::control_call::{ControlCallPlan, reachable_states};

pub(in crate::c_emit::body) struct CommonControlPlan {
    functions: HashSet<FunctionId>,
}

impl CommonControlPlan {
    pub(in crate::c_emit::body) fn new(program: &Program, calls: &ControlCallPlan) -> Self {
        let mut functions = HashSet::new();
        for function in &program.functions {
            if reachable_states(program, function.entry)
                .into_iter()
                .any(|site| {
                    calls.is_recursive_dispatch(site)
                        && !matches!(
                            program.states[site.0].terminator,
                            Terminator::Call {
                                ref callee,
                                ..
                            } if matches!(
                                callee.kind,
                                crate::closure::ast::AtomKind::Reference(
                                    crate::closure::ast::Reference::SelfClosure(target)
                                ) if target == function.id
                            )
                        )
                })
            {
                functions.insert(function.id);
            }
        }

        loop {
            let mut changed = false;
            for function in &program.functions {
                if !functions.contains(&function.id) {
                    continue;
                }
                for site in reachable_states(program, function.entry) {
                    if !calls.is_recursive_dispatch(site) {
                        continue;
                    }
                    for target in calls.recursive_dispatch_targets(site).unwrap_or_default() {
                        changed |= functions.insert(*target);
                    }
                }
            }
            if !changed {
                break;
            }
        }
        Self { functions }
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
