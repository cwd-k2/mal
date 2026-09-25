use crate::closure::ast::FunctionId;
use crate::control::ast::{Program, StateId};

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
        .map(|function| (&function.states, Owner::Function(function.id)))
        .chain(
            control
                .bindings
                .iter()
                .enumerate()
                .map(|(index, binding)| (&binding.states, Owner::Binding(index))),
        );
    for (states, owner) in entries {
        for StateId(state) in states {
            owners[*state] = Some(owner);
        }
    }
    owners
}
