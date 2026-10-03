use std::collections::HashMap;

use crate::closure::ast::{Atom, Pattern};
use crate::control::ast::{Operation, StateId};
use crate::core::ast::BufferOperation;
use crate::execution::ownership::{BindingOperand, PatternDestination, UseEffect};

use super::BufferIdentityBinding;

/// Finds a `get`, move-only aliases of its result, and a `put` that returns the responsibility to
/// the same place without an intervening effect.
pub(super) fn plan(
    control: &crate::control::ast::Program,
    ownership: &crate::execution::OwnershipPlan,
) -> HashMap<(StateId, usize), BufferIdentityBinding> {
    let mut decisions = HashMap::new();
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (get_index, get) in state.bindings.iter().enumerate() {
            let Operation::Buffer {
                operation: BufferOperation::Get,
                element: get_element,
                operands: get_operands,
            } = &get.operation
            else {
                continue;
            };
            let Pattern::Binding { id, ty } = &get.pattern else {
                continue;
            };
            let [get_buffer, get_coordinate] = get_operands.as_slice() else {
                continue;
            };
            if ty != get_element
                || !crate::execution::ownership::is_managed(get_element)
                || ownership.binding_destination(site, get_index)
                    != Some(&PatternDestination::Initialize(*id))
            {
                continue;
            }

            let mut value = *id;
            let mut value_bindings = vec![get_index];
            let mut cursor = get_index + 1;
            while let Some(binding) = state.bindings.get(cursor) {
                if let (Pattern::Binding { id: next, .. }, Operation::Atom(atom)) =
                    (&binding.pattern, &binding.operation)
                    && atom.binding() == Some(value)
                    && ownership.binding_destination(site, cursor)
                        == Some(&PatternDestination::Initialize(*next))
                    && ownership.binding_use(site, cursor, BindingOperand::Atom)
                        == Some(UseEffect::Consume)
                {
                    value = *next;
                    value_bindings.push(cursor);
                    cursor += 1;
                    continue;
                }

                let Operation::Buffer {
                    operation: BufferOperation::Put,
                    element: put_element,
                    operands: put_operands,
                } = &binding.operation
                else {
                    break;
                };
                let [put_buffer, put_coordinate, put_value] = put_operands.as_slice() else {
                    break;
                };
                if get_element == put_element
                    && same_value(get_buffer, put_buffer)
                    && same_value(get_coordinate, put_coordinate)
                    && put_value.binding() == Some(value)
                    && ownership.binding_use(site, cursor, BindingOperand::BufferOperand(2))
                        == Some(UseEffect::Consume)
                {
                    for binding in value_bindings {
                        decisions.insert((site, binding), BufferIdentityBinding::Value);
                    }
                    decisions.insert((site, cursor), BufferIdentityBinding::Put);
                }
                break;
            }
        }
    }
    decisions
}

fn same_value(left: &Atom, right: &Atom) -> bool {
    left.ty == right.ty && left.kind == right.kind
}
