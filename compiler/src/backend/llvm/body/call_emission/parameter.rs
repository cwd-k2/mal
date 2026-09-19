use crate::closure::ast::FunctionId;
use crate::execution::ParameterDestination;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_parameter_handoff(
        &mut self,
        target: FunctionId,
        value: &EmittedValue,
        entry: crate::execution::ownership::ParameterEntry,
    ) -> Option<()> {
        let function = *self.index.control_functions.get(&target)?;
        if function.parameter.ty != value.ty {
            return None;
        }
        let destination = self.execution.parameters.destination(target)?;
        if crate::execution::ownership::is_managed(&value.ty) {
            let effect = self.ownership.parameter_effect(target, entry);
            match (entry, effect, destination) {
                (
                    crate::execution::ownership::ParameterEntry::BorrowedAbi
                    | crate::execution::ownership::ParameterEntry::OwnedHandoff,
                    Some(crate::execution::ownership::ParameterEffect::BorrowInto(binding)),
                    _,
                ) if !value.owned => return self.store_parameter_binding(binding, value),
                (
                    crate::execution::ownership::ParameterEntry::BorrowedAbi,
                    Some(crate::execution::ownership::ParameterEffect::ShareInto(binding)),
                    _,
                ) if !value.owned => {
                    let mut value = value.clone();
                    self.retain_if_borrowed(&mut value)?;
                    return self.store_parameter_binding(binding, &value);
                }
                (crate::execution::ownership::ParameterEntry::BorrowedAbi, None, _)
                    if !value.owned =>
                {
                    return Some(());
                }
                (crate::execution::ownership::ParameterEntry::OwnedHandoff, None, _)
                    if !value.owned =>
                {
                    return Some(());
                }
                (
                    crate::execution::ownership::ParameterEntry::OwnedHandoff,
                    Some(crate::execution::ownership::ParameterEffect::ConsumeInto(binding)),
                    _,
                ) if value.owned => return self.store_parameter_binding(binding, value),
                (
                    crate::execution::ownership::ParameterEntry::OwnedHandoff,
                    Some(crate::execution::ownership::ParameterEffect::Drop),
                    _,
                ) if value.owned => {
                    return self.release_value(&value.ty, &value.representation);
                }
                _ => return None,
            }
        }
        match destination {
            ParameterDestination::Bind(binding) => {
                self.store_parameter_binding(binding, value)?;
            }
            ParameterDestination::Discard => {}
        }
        Some(())
    }

    fn store_parameter_binding(
        &mut self,
        binding: crate::anf::ast::ValueId,
        value: &EmittedValue,
    ) -> Option<()> {
        let slot = self.slots.get(&binding)?.clone();
        if slot.ty != value.ty {
            return None;
        }
        let value_type = self.types.value(&slot.ty)?;
        self.line(format!(
            "  store {} {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, value.representation, slot.index, value_type.alignment
        ));
        Some(())
    }
}
