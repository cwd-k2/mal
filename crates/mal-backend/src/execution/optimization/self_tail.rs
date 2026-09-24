use std::collections::HashSet;

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::super::ApplicationGraph;

pub(super) fn plan(
    control: &control::Program,
    applications: &ApplicationGraph,
) -> HashSet<StateId> {
    control
        .functions
        .iter()
        .flat_map(|function| {
            applications
                .sites_from(function.id)
                .filter_map(|(site, _)| {
                    applies(&control.states[site.0].terminator, function.id).then_some(site)
                })
        })
        .collect()
}

fn applies(terminator: &Terminator, function: FunctionId) -> bool {
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
