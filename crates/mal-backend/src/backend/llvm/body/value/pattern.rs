use crate::closure::ast::Pattern;
use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn store_self_tail_pattern(
        &mut self,
        pattern: &Pattern,
        value: &EmittedValue,
    ) -> Option<()> {
        match pattern {
            Pattern::Binding { ty, .. }
                if crate::execution::ownership::is_managed(ty) && value.ty == *ty => {}
            Pattern::Binding { id, ty }
                if !crate::execution::ownership::is_managed(ty) && value.ty == *ty =>
            {
                let slot = self.slots.get(id)?.clone();
                self.initialize_slot(&slot, value)?;
            }
            Pattern::Product { elements, ty, .. } if value.ty == *ty => {
                let Type::Product(element_types) = ty else {
                    return None;
                };
                if elements.len() != element_types.len() {
                    return None;
                }
                let aggregate_type = self.types.value(ty)?;
                for (index, (element, element_type)) in
                    elements.iter().zip(element_types.iter()).enumerate()
                {
                    let register = self.register();
                    emit_instruction! {
                        self;
                        let #{ register.clone() } = extract_value {
                            aggregate: typed(#{ aggregate_type.llvm.clone() }, #{ value.representation.clone() }),
                            indices: [#{ index }],
                        };
                    };
                    self.store_self_tail_pattern(
                        element,
                        &EmittedValue {
                            ty: element_type.clone(),
                            representation: register,
                            owned: false,
                        },
                    )?;
                }
            }
            Pattern::Wildcard { ty, .. }
                if !crate::execution::ownership::is_managed(ty) && value.ty == *ty => {}
            _ => return None,
        }
        Some(())
    }

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
        use crate::execution::ownership::PatternDestination;
        match (pattern, destination) {
            (Pattern::Binding { id, ty }, destination) => {
                self.store_binding_to_destination(*id, ty, value?, destination)
            }
            (Pattern::Product { elements, ty, .. }, PatternDestination::Product(destinations)) => {
                self.store_product_to_destinations(elements, ty, value?, destinations)
            }
            (Pattern::Wildcard { ty, .. }, PatternDestination::Discard)
                if crate::execution::ownership::is_managed(ty) =>
            {
                let value = value?;
                if value.ty != *ty {
                    return None;
                }
                if value.owned {
                    self.release_value(ty, &value.representation)?;
                }
                Some(())
            }
            (Pattern::Wildcard { ty, .. }, PatternDestination::Unmanaged)
                if !crate::execution::ownership::is_managed(ty) =>
            {
                Some(())
            }
            _ => None,
        }
    }

    fn store_binding_to_destination(
        &mut self,
        id: crate::anf::ast::ValueId,
        ty: &Type,
        value: &EmittedValue,
        destination: &crate::execution::ownership::PatternDestination,
    ) -> Option<()> {
        use crate::execution::ownership::PatternDestination;
        if value.ty != *ty {
            return None;
        }
        match destination {
            PatternDestination::Discard if crate::execution::ownership::is_managed(ty) => {
                if value.owned {
                    self.release_value(ty, &value.representation)?;
                }
            }
            PatternDestination::Initialize(target)
                if *target == id && crate::execution::ownership::is_managed(ty) =>
            {
                let mut value = value.clone();
                self.retain_if_borrowed(&mut value)?;
                let slot = self.slots.get(&id)?.clone();
                self.initialize_slot(&slot, &value)?;
            }
            PatternDestination::Borrow(target)
                if *target == id && crate::execution::ownership::is_managed(ty) =>
            {
                let slot = self.slots.get(&id)?.clone();
                self.initialize_slot(&slot, value)?;
            }
            PatternDestination::Unmanaged if self.types.value(ty).is_some() => {
                let slot = self.slots.get(&id)?.clone();
                self.initialize_slot(&slot, value)?;
            }
            _ => return None,
        }
        Some(())
    }

    fn store_product_to_destinations(
        &mut self,
        elements: &[Pattern],
        ty: &Type,
        value: &EmittedValue,
        destinations: &[crate::execution::ownership::PatternDestination],
    ) -> Option<()> {
        let Type::Product(element_types) = ty else {
            return None;
        };
        if value.ty != *ty
            || elements.len() != element_types.len()
            || destinations.len() != elements.len()
        {
            return None;
        }
        let aggregate_type = self.types.value(ty)?;
        for (index, ((element, element_type), destination)) in elements
            .iter()
            .zip(element_types.iter())
            .zip(destinations)
            .enumerate()
        {
            let register = self.register();
            emit_instruction! {
                self;
                let #{ register.clone() } = extract_value {
                    aggregate: typed(#{ aggregate_type.llvm.clone() }, #{ value.representation.clone() }),
                    indices: [#{ index }],
                };
            };
            self.store_pattern_to_destination(
                element,
                Some(&EmittedValue {
                    ty: element_type.clone(),
                    representation: register,
                    owned: value.owned && crate::execution::ownership::is_managed(element_type),
                }),
                destination,
            )?;
        }
        Some(())
    }
}
