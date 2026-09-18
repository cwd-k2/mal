use crate::check::ast::Type;
use crate::closure::ast::Pattern;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn store_binding_pattern(
        &mut self,
        state: crate::control::ast::StateId,
        binding: usize,
        pattern: &Pattern,
        value: Option<&EmittedValue>,
    ) -> Option<()> {
        let destination = self.ownership.binding_destination(state, binding)?.clone();
        self.store_pattern_to_destination(pattern, value, &destination)
    }

    pub(in crate::backend::llvm::body) fn store_input_pattern(
        &mut self,
        state: crate::control::ast::StateId,
        value: Option<&EmittedValue>,
    ) -> Option<()> {
        let pattern = self.control.states[state.0].input.as_ref()?.clone();
        let destination = self.ownership.input_destination(state)?.clone();
        self.store_pattern_to_destination(&pattern, value, &destination)
    }

    fn store_pattern_to_destination(
        &mut self,
        pattern: &Pattern,
        value: Option<&EmittedValue>,
        destination: &crate::execution::ownership::PatternDestination,
    ) -> Option<()> {
        match pattern {
            Pattern::Binding { ty, .. }
                if crate::execution::ownership::is_managed(ty)
                    && matches!(
                        destination,
                        crate::execution::ownership::PatternDestination::Discard
                    ) =>
            {
                let value = value?;
                if value.ty != *ty {
                    return None;
                }
                if value.owned {
                    self.release_value(ty, &value.representation)?;
                }
            }
            Pattern::Binding { id, ty }
                if crate::execution::ownership::is_managed(ty)
                    && matches!(
                        destination,
                        crate::execution::ownership::PatternDestination::Initialize(target)
                            if target == id
                    ) =>
            {
                let mut value = value?.clone();
                if value.ty != *ty {
                    return None;
                }
                self.retain_if_borrowed(&mut value)?;
                let slot = self.slots.get(id)?.clone();
                let value_type = self.types.value(ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
                ));
            }
            Pattern::Binding { id, ty }
                if self.types.value(ty).is_some()
                    && matches!(
                        destination,
                        crate::execution::ownership::PatternDestination::Unmanaged
                    ) =>
            {
                let value = value?;
                if value.ty != *ty {
                    return None;
                }
                let slot = self.slots.get(id)?.clone();
                let value_type = self.types.value(ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
                ));
            }
            Pattern::Product { elements, ty, .. } => {
                let value = value?;
                let Type::Product(element_types) = ty else {
                    return None;
                };
                if value.ty != *ty || elements.len() != element_types.len() {
                    return None;
                }
                let aggregate_type = self.types.value(ty)?;
                let destinations = match destination {
                    crate::execution::ownership::PatternDestination::Product(
                        destination_elements,
                    ) if destination_elements.len() == elements.len() => {
                        destination_elements.as_slice()
                    }
                    _ => return None,
                };
                for (index, (element, element_type)) in
                    elements.iter().zip(element_types.iter()).enumerate()
                {
                    let register = self.register();
                    self.line(format!(
                        "  {register} = extractvalue {} {}, {index}",
                        aggregate_type.llvm, value.representation
                    ));
                    self.store_pattern_to_destination(
                        element,
                        Some(&EmittedValue {
                            ty: element_type.clone(),
                            representation: register,
                            owned: value.owned
                                && crate::execution::ownership::is_managed(element_type),
                        }),
                        &destinations[index],
                    )?;
                }
            }
            Pattern::Wildcard { ty, .. }
                if crate::execution::ownership::is_managed(ty)
                    && matches!(
                        destination,
                        crate::execution::ownership::PatternDestination::Discard
                    ) =>
            {
                let value = value?;
                if value.owned {
                    self.release_value(ty, &value.representation)?;
                }
            }
            Pattern::Wildcard { ty, .. }
                if !crate::execution::ownership::is_managed(ty)
                    && matches!(
                        destination,
                        crate::execution::ownership::PatternDestination::Unmanaged
                    ) => {}
            _ => return None,
        }
        Some(())
    }
}
