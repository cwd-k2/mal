use crate::backend::llvm::syntax::CastOperator;
use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
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
        let register = self.load_slot(&slot)?;
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
        if effect == UseEffect::Consume
            && let AtomKind::Reference(Reference::Capture(index)) = atom.kind
        {
            return self.prepare_unique_capture(atom, index);
        }
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

    fn prepare_unique_capture(&mut self, atom: &Atom, index: usize) -> Option<PreparedValue> {
        let mut value = self.atom(atom)?;
        let environment = self.active_environment();
        let unique = self.register();
        emit_instruction! {
            self;
            let { unique.clone() } = call {
                tail: false,
                result_type: int(8_u16),
                callee: direct("mal_runtime_owner_is_unique"),
                arguments: [(ptr, { environment })],
            };
        };
        let condition = self.register();
        emit_instruction! {
            self;
            let { condition.clone() } = cast {
                operator: { CastOperator::Trunc },
                value: (int(8_u16), { unique }),
                to: int(1_u16),
            };
        };
        let label = self.label_id();
        emit_terminator! {
            self;
            branch {
                condition: { condition },
                then: { format!("mal_capture_take_{label}") },
                otherwise: { format!("mal_capture_share_{label}") },
            };
        };
        self.block(format!("mal_capture_take_{label}"));
        let pointer = self.capture_pointer(index, &atom.ty)?;
        let value_type = self.types.value(&atom.ty)?;
        self.vacate_place(&atom.ty, &pointer, value_type.alignment)?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_capture_ready_{label}") },
            };
        };
        self.block(format!("mal_capture_share_{label}"));
        self.retain_if_borrowed(&mut value)?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_capture_ready_{label}") },
            };
        };
        self.block(format!("mal_capture_ready_{label}"));
        value.owned = true;
        Some(PreparedValue {
            value,
            consumed_slots: Vec::new(),
        })
    }

    pub(in crate::backend::llvm::body) fn commit_consumes(
        &mut self,
        prepared: &PreparedValue,
    ) -> Option<()> {
        for slot in &prepared.consumed_slots {
            self.vacate_slot(slot)?;
        }
        Some(())
    }
}

impl FunctionEmitter<'_> {
    /// Moves the value of a binding that dies at this operation out of its slot. The slot is left vacant, so the
    /// release that follows the operation does nothing and the caller owns the value.
    pub(in crate::backend::llvm::body) fn take_binding(
        &mut self,
        atom: &Atom,
        ty: &mal_frontend::check::ast::Type,
    ) -> Option<EmittedValue> {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return None;
        };
        let slot = self.slots.get(&id)?.clone();
        if slot.ty != *ty {
            return None;
        }
        let value = self.atom(atom)?;
        self.vacate_slot(&slot)?;
        Some(EmittedValue {
            owned: true,
            ..value
        })
    }
}
