use std::collections::HashSet;

use crate::control::ast::{Operation, StateId};
use mal_frontend::check::ast::MemoryPrimitive;

/// Byte `*` conversions whose single operand dies at the conversion, so the conversion may take the operand's
/// reference and move its byte owner instead of copying.
pub(super) fn plan(
    control: &crate::control::ast::Program,
    ownership: &crate::execution::OwnershipPlan,
) -> HashSet<(StateId, usize)> {
    let mut decisions = HashSet::new();
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let Operation::Memory {
                primitive: MemoryPrimitive::BufferToSymbol | MemoryPrimitive::SymbolToBuffer,
                operands,
            } = &binding.operation
            else {
                continue;
            };
            let [operand] = operands.as_slice() else {
                continue;
            };
            let dead = ownership.drops_after_binding(site, binding_index);
            if operand.binding().is_some_and(|id| dead.contains(&id)) {
                decisions.insert((site, binding_index));
            }
        }
    }
    decisions
}
