use std::collections::HashMap;

use crate::control::ast::{Operation, StateId};
use mal_frontend::check::ast::SymbolPrimitive;

use super::SymbolConcatMode;

pub(super) fn plan(
    control: &crate::control::ast::Program,
    ownership: &crate::execution::OwnershipPlan,
) -> HashMap<(StateId, usize), SymbolConcatMode> {
    let mut decisions = HashMap::new();
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let Operation::Symbol {
                primitive: SymbolPrimitive::Concatenate,
                operands,
            } = &binding.operation
            else {
                continue;
            };
            let [left, right] = operands.as_slice() else {
                continue;
            };
            let dead = ownership.drops_after_binding(site, binding_index);
            let left_id = left.binding();
            let right_id = right.binding();
            let mode = if left_id.is_some_and(|id| dead.contains(&id)) && left_id != right_id {
                Some(SymbolConcatMode::ConsumeLeft)
            } else if right_id.is_some_and(|id| dead.contains(&id)) && left_id != right_id {
                Some(SymbolConcatMode::ConsumeRight)
            } else {
                None
            };
            if let Some(mode) = mode {
                decisions.insert((site, binding_index), mode);
            }
        }
    }
    decisions
}
