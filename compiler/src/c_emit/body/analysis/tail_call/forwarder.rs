use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, Terminator};

pub(super) fn forwarded_self_tail_argument(
    program: &closure::Program,
    state: &control::State,
    terminator: &Terminator,
    caller: FunctionId,
    forwarder: FunctionId,
) -> Option<closure::Atom> {
    let Terminator::TailCall { argument, .. } = terminator else {
        return None;
    };
    let function = program
        .functions
        .iter()
        .find(|function| function.id == forwarder)?;
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
