//! Slot states: loading, initializing, vacating, and replacing a place, and releasing slots whose owners die.

use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn load_slot(
        &mut self,
        slot: &super::super::Slot,
    ) -> Option<String> {
        let value_type = self.types.value(&slot.ty)?;
        let value = self.register();
        emit_instruction! {
            self;
            let #{ value.clone() } = load {
                ty: #{ value_type.llvm },
                pointer: #{ format!("%mal_slot_{}", slot.index) },
                alignment: #{ value_type.alignment },
                metadata: [],
            };
        };
        Some(value)
    }

    pub(in crate::backend::llvm::body) fn initialize_slot(
        &mut self,
        slot: &super::super::Slot,
        value: &EmittedValue,
    ) -> Option<()> {
        if slot.ty != value.ty {
            return None;
        }
        let value_type = self.types.value(&slot.ty)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, #{ value.representation.as_str() }),
                pointer: #{ format!("%mal_slot_{}", slot.index) },
                alignment: #{ value_type.alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn vacate_slot(
        &mut self,
        slot: &super::super::Slot,
    ) -> Option<()> {
        let value_type = self.types.value(&slot.ty)?;
        self.vacate_place(
            &slot.ty,
            &format!("%mal_slot_{}", slot.index),
            value_type.alignment,
        )
    }

    pub(in crate::backend::llvm::body) fn vacate_place(
        &mut self,
        ty: &Type,
        pointer: &str,
        alignment: usize,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(ty) {
            return None;
        }
        let value_type = self.types.value(ty)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, "zeroinitializer"),
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn replace_managed_place(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        alignment: usize,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&value.ty) {
            return None;
        }
        let value_type = self.types.value(&value.ty)?;
        self.retain_value(&value.ty, &value.representation)?;
        let previous = self.register();
        emit_instruction! {
            self;
            let #{ previous.clone() } = load {
                ty: #{ value_type.llvm.clone() },
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        self.release_value(&value.ty, &previous)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, #{ value.representation.as_str() }),
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: [],
            };
        };
        Some(())
    }

    pub(in crate::backend::llvm::body) fn retain_if_borrowed(
        &mut self,
        value: &mut EmittedValue,
    ) -> Option<()> {
        if crate::execution::ownership::is_managed(&value.ty) && !value.owned {
            value.representation = self.retain_value(&value.ty, &value.representation)?;
            value.owned = true;
        }
        Some(())
    }

    pub(in crate::backend::llvm::body) fn release_dead_slot(
        &mut self,
        id: crate::anf::ast::ValueId,
    ) -> Option<()> {
        let Some(slot) = self.slots.get(&id).cloned() else {
            return Some(());
        };
        if !crate::execution::ownership::is_managed(&slot.ty) {
            return Some(());
        }
        self.release_slot(&slot)
    }

    pub(in crate::backend::llvm::body) fn emit_edge_drops(
        &mut self,
        site: crate::control::ast::StateId,
        path: crate::execution::ownership::ControlPath,
    ) -> Option<()> {
        let mut drops = self.ownership.drops_on_edge(site, path).to_vec();
        drops.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
        for id in drops {
            self.release_dead_slot(id)?;
        }
        Some(())
    }

    pub(super) fn release_slot(&mut self, slot: &super::super::Slot) -> Option<()> {
        let value = self.load_slot(slot)?;
        self.release_value(&slot.ty, &value)?;
        self.vacate_slot(slot)
    }
}
