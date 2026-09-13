use std::collections::HashMap;

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::super::ApplicationGraph;

pub(super) fn plan(
    program: &closure::Program,
    control: &control::Program,
    applications: &ApplicationGraph,
) -> HashMap<StateId, closure::Atom> {
    let functions = program
        .functions
        .iter()
        .map(|function| (function.id, function))
        .collect::<HashMap<_, _>>();
    control
        .functions
        .iter()
        .flat_map(|function| {
            applications
                .sites_from(function.id)
                .filter_map(|(site, _)| {
                    let state = &control.states[site.0];
                    applications
                        .direct_target(site)
                        .and_then(|forwarder| {
                            forwarded_self_tail_argument(
                                &functions,
                                state,
                                &state.terminator,
                                function.id,
                                forwarder,
                            )
                        })
                        .map(|argument| (site, argument))
                })
        })
        .collect()
}

fn forwarded_self_tail_argument(
    functions: &HashMap<FunctionId, &closure::Function>,
    state: &control::State,
    terminator: &Terminator,
    caller: FunctionId,
    forwarder: FunctionId,
) -> Option<closure::Atom> {
    let Terminator::TailCall { argument, .. } = terminator else {
        return None;
    };
    let function = functions.get(&forwarder)?;
    let [first, tail] = function.body.bindings.as_slice() else {
        return None;
    };
    let closure::Pattern::Product { elements, .. } = &first.pattern else {
        return None;
    };
    let [
        closure::Pattern::Binding { id: callee_id, .. },
        closure::Pattern::Binding {
            id: argument_id, ..
        },
    ] = elements.as_slice()
    else {
        return None;
    };
    let closure::Operation::Atom(closure::Atom {
        kind: AtomKind::Reference(Reference::Binding(parameter)),
        ..
    }) = &first.operation
    else {
        return None;
    };
    if Some(*parameter) != function.parameter.binding {
        return None;
    }
    let closure::Operation::Call {
        callee:
            closure::Atom {
                kind: AtomKind::Reference(Reference::Binding(indirect_callee)),
                ..
            },
        argument:
            closure::Atom {
                kind: AtomKind::Reference(Reference::Binding(indirect_argument)),
                ..
            },
    } = &tail.operation
    else {
        return None;
    };
    if indirect_callee != callee_id || indirect_argument != argument_id {
        return None;
    }
    let AtomKind::Reference(Reference::Binding(product)) = &argument.kind else {
        return None;
    };
    let binding = state.bindings.iter().find(
        |binding| matches!(binding.pattern, closure::Pattern::Binding { id, .. } if id == *product),
    )?;
    let control::Operation::Product(elements) = &binding.operation else {
        return None;
    };
    let [forwarded_callee, forwarded_argument] = elements.as_slice() else {
        return None;
    };
    matches!(
        &forwarded_callee.kind,
        AtomKind::Reference(Reference::SelfClosure(target)) if *target == caller
    )
    .then(|| forwarded_argument.clone())
}
