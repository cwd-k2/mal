//! Which call sites of a top-level function should get a function of their own.
//!
//! Two call sites that pass different closures make the callee's parameters context insensitive: the callee cannot
//! call one closure directly, and code that reaches it through those closures can look recursive. A function is
//! copied for each set of closures that reaches its call sites, keeping the first set on the original.

use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomId, AtomKind, FunctionId, Program, Reference};
use crate::control::ast::{Program as Control, StateId, Terminator};
use crate::flow::{ClosureFlow, holds_function};

/// The call sites that should call a copy of the top-level function bound to `binding`.
pub(super) struct Request {
    pub(super) binding: ValueId,
    pub(super) function: FunctionId,
    pub(super) sites: Vec<AtomId>,
}

pub(super) fn requests(program: &Program, control: &Control, flow: &ClosureFlow) -> Vec<Request> {
    let known = program
        .bindings
        .iter()
        .filter_map(|binding| binding.known_function())
        .collect::<HashMap<_, _>>();
    let takes_functions = program
        .functions
        .iter()
        .filter(|function| holds_function(&function.parameter.ty))
        .map(|function| function.id)
        .collect::<std::collections::HashSet<_>>();

    // Per callee binding, the closure sets in first-seen order with the callee atoms that pass each.
    let mut groups = HashMap::<ValueId, Vec<(Vec<u32>, Vec<AtomId>)>>::new();
    let mut order = Vec::new();
    for (index, state) in control.states.iter().enumerate() {
        let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
            &state.terminator
        else {
            continue;
        };
        let AtomKind::Reference(Reference::Binding(binding)) = callee.kind else {
            continue;
        };
        let Some(function) = known.get(&binding) else {
            continue;
        };
        let Some(reaching) = flow.argument(StateId(index)) else {
            continue;
        };
        if !takes_functions.contains(function) {
            continue;
        }
        let mut key = reaching
            .iter()
            .map(|function| number(*function))
            .collect::<Vec<_>>();
        key.sort_unstable();
        let entry = groups.entry(binding).or_default();
        if entry.is_empty() {
            order.push(binding);
        }
        match entry.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, sites)) => sites.push(callee.id),
            None => entry.push((key, vec![callee.id])),
        }
    }
    order
        .into_iter()
        .flat_map(|binding| {
            let function = known[&binding];
            groups
                .remove(&binding)
                .unwrap_or_default()
                .into_iter()
                .skip(1)
                .map(move |(_, sites)| Request {
                    binding,
                    function,
                    sites,
                })
        })
        .collect()
}

fn number(function: FunctionId) -> u32 {
    let FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) = function;
    number
}
