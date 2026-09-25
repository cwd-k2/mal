use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

use crate::control::reachable_states;

/// The code a control state belongs to: a function body or a top-level binding's initializer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum Owner {
    Function(FunctionId),
    Binding(usize),
}

pub(super) fn state_owners(control: &Program) -> Vec<Option<Owner>> {
    let mut owners = vec![None; control.states.len()];
    let entries = control
        .functions
        .iter()
        .map(|function| (function.entry, Owner::Function(function.id)))
        .chain(
            control
                .bindings
                .iter()
                .enumerate()
                .map(|(index, binding)| (binding.entry, Owner::Binding(index))),
        );
    for (entry, owner) in entries {
        for StateId(state) in reachable_states(control, entry) {
            owners[state] = Some(owner);
        }
    }
    owners
}
