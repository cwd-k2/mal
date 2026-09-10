use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::ApplicationGraph;

mod forwarder;

use forwarder::forwarded_self_tail_argument;

pub(crate) struct TailCallPlan {
    fused_sites: HashSet<StateId>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
}

impl TailCallPlan {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
    ) -> Self {
        let mut fused_sites = HashSet::new();
        let mut forwarded_self_arguments = HashMap::new();
        for function in &control.functions {
            for (site, _) in applications.sites_from(function.id) {
                let state = &control.states[site.0];
                let terminator = &state.terminator;
                if let Some(argument) = applications.direct_target(site).and_then(|forwarder| {
                    forwarded_self_tail_argument(closure, state, terminator, function.id, forwarder)
                }) {
                    fused_sites.insert(site);
                    forwarded_self_arguments.insert(site, argument);
                } else if is_direct_self_tail(terminator, function.id) {
                    fused_sites.insert(site);
                }
            }
        }
        Self {
            fused_sites,
            forwarded_self_arguments,
        }
    }

    pub(crate) fn is_fused(&self, site: StateId) -> bool {
        self.fused_sites.contains(&site)
    }

    pub(crate) fn forwarded_self_argument(&self, site: StateId) -> Option<&closure::Atom> {
        self.forwarded_self_arguments.get(&site)
    }
}

fn is_direct_self_tail(terminator: &Terminator, function: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::TailCall {
            callee:
                closure::Atom {
                    kind: AtomKind::Reference(Reference::SelfClosure(target)),
                    ..
                },
            ..
        } if *target == function
    )
}
