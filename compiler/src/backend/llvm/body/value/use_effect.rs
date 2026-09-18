use crate::closure::ast::{Atom, AtomKind, Reference};
use crate::execution::ownership::UseEffect;

use super::super::{EmittedValue, FunctionEmitter, PreparedValue};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn require_binding_borrow(
        &self,
        site: crate::control::ast::StateId,
        binding: usize,
        operand: crate::execution::ownership::BindingOperand,
        atom: &Atom,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&atom.ty) {
            return Some(());
        }
        (self.ownership.binding_use(site, binding, operand) == Some(UseEffect::Borrow))
            .then_some(())
    }

    pub(in crate::backend::llvm::body) fn require_terminator_borrow(
        &self,
        site: crate::control::ast::StateId,
        operand: crate::execution::ownership::TerminatorOperand,
        atom: &Atom,
    ) -> Option<()> {
        if !crate::execution::ownership::is_managed(&atom.ty) {
            return Some(());
        }
        (self.ownership.terminator_use(site, operand) == Some(UseEffect::Borrow)).then_some(())
    }

    pub(in crate::backend::llvm::body) fn prepare_binding_for_use(
        &mut self,
        id: crate::anf::ast::ValueId,
        effect: UseEffect,
    ) -> Option<PreparedValue> {
        let slot = self.slots.get(&id)?.clone();
        let value_type = self.types.value(&slot.ty)?;
        let register = self.register();
        self.line(format!(
            "  {register} = load {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        let mut value = EmittedValue {
            ty: slot.ty.clone(),
            representation: register,
            owned: false,
        };
        let consumed_slots = match effect {
            UseEffect::Borrow => Vec::new(),
            UseEffect::Share => {
                self.retain_if_borrowed(&mut value)?;
                Vec::new()
            }
            UseEffect::Consume => {
                value.owned = true;
                vec![slot]
            }
        };
        Some(PreparedValue {
            value,
            consumed_slots,
        })
    }

    pub(in crate::backend::llvm::body) fn prepare_atom_for_use(
        &mut self,
        atom: &Atom,
        effect: UseEffect,
    ) -> Option<PreparedValue> {
        let mut value = self.atom(atom)?;
        let mut consumed_slots = Vec::new();
        match effect {
            UseEffect::Borrow => {}
            UseEffect::Share => self.retain_if_borrowed(&mut value)?,
            UseEffect::Consume => {
                let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
                    return None;
                };
                let slot = self.slots.get(&id)?.clone();
                if slot.ty != atom.ty || !crate::execution::ownership::is_managed(&slot.ty) {
                    return None;
                }
                value.owned = true;
                consumed_slots.push(slot);
            }
        }
        Some(PreparedValue {
            value,
            consumed_slots,
        })
    }

    pub(in crate::backend::llvm::body) fn commit_consumes(
        &mut self,
        prepared: &PreparedValue,
    ) -> Option<()> {
        for slot in &prepared.consumed_slots {
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
                value_type.llvm, slot.index, value_type.alignment
            ));
        }
        Some(())
    }
}
