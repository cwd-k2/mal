use crate::backend::llvm::syntax::llvm_global;
use crate::closure::ast::{Atom, AtomKind, Reference};
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter, memory::ByteViewFields};

pub(super) const STATIC_OWNER_DATA_OFFSET: usize = 24;
const STATIC_OWNER_ALIGNMENT: usize = 8;

pub(super) fn literal_storage_size(length: usize) -> Option<usize> {
    super::types::align(
        STATIC_OWNER_DATA_OFFSET.checked_add(length)?,
        STATIC_OWNER_ALIGNMENT,
    )
}

pub(super) fn literal_definition(
    name: &str,
    bytes: &[u8],
) -> Option<crate::backend::llvm::syntax::GlobalDefinition> {
    llvm_global!(byte_owner {
        name: { { name } },
        bytes: { { bytes.to_vec() } },
        alignment: { { STATIC_OWNER_ALIGNMENT } },
    })
}

impl FunctionEmitter<'_> {
    pub(super) fn emit_symbol_length(&mut self, value: &Atom) -> Option<EmittedValue> {
        let value = self.atom(value)?;
        if value.ty != Type::Symbol {
            return None;
        }
        let length = self.byte_view_fields(&value)?.count;
        Some(EmittedValue {
            ty: Type::USize,
            representation: length,
            owned: false,
        })
    }

    pub(super) fn emit_symbol_at(&mut self, argument: &Atom) -> Option<EmittedValue> {
        let argument = self.atom(argument)?;
        let [symbol, index] = self.product_fields(&argument, [&Type::Symbol, &Type::USize])?;
        let data = self.byte_view_fields(&symbol)?.data;
        let result = self.register();
        emit_instruction!(
            self;
            let {{ result.clone() }} = call {
                tail: false,
                result_type: (int(8_u16)),
                callee: direct("mal_runtime_symbol_at"),
                arguments: [typed((ptr), {{ data }}), typed({{ self.types.index_llvm_type() }}, {{ index.representation }})],
            };
        );
        Some(EmittedValue {
            ty: Type::UInt8,
            representation: result,
            owned: false,
        })
    }

    pub(super) fn emit_symbol_concatenate(
        &mut self,
        left: &Atom,
        right: &Atom,
        mode: super::super::optimization::SymbolConcatMode,
    ) -> Option<EmittedValue> {
        use super::super::optimization::SymbolConcatMode;

        let consume_left = mode == SymbolConcatMode::ConsumeLeft && self.atom_has_slot(left);
        let consume_right = mode == SymbolConcatMode::ConsumeRight && self.atom_has_slot(right);
        let left = if consume_left {
            self.take_symbol(left)?
        } else {
            self.atom(left)?
        };
        let right = if consume_right {
            self.take_symbol(right)?
        } else {
            self.atom(right)?
        };
        if left.ty != Type::Symbol || right.ty != Type::Symbol {
            return None;
        }
        let ByteViewFields {
            owner: left_owner,
            data: left_data,
            count: left_length,
        } = self.byte_view_fields(&left)?;
        let ByteViewFields {
            owner: right_owner,
            data: right_data,
            count: right_length,
        } = self.byte_view_fields(&right)?;
        let result_type = self.types.value(&Type::Symbol)?;
        if !self.needs_symbol_result_slot {
            return None;
        }
        let result_storage = "%mal_symbol_result";
        let operation = if consume_left {
            "mal_runtime_symbol_concatenate_consuming_left"
        } else if consume_right {
            "mal_runtime_symbol_concatenate_consuming_right"
        } else {
            "mal_runtime_symbol_concatenate"
        };
        emit_instruction!(
            self;
            call {
                tail: false,
                result_type: (void),
                callee: direct({{ operation }}),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), {{ result_storage }}), typed((ptr), {{ left_owner }}), typed((ptr), {{ left_data }}), typed({{ self.types.index_llvm_type() }}, {{ left_length }}), typed((ptr), {{ right_owner }}), typed((ptr), {{ right_data }}), typed({{ self.types.index_llvm_type() }}, {{ right_length }})],
            };
        );
        let result = self.register();
        emit_instruction!(
            self;
            let {{ result.clone() }} = load {
                ty: {{ result_type.llvm }},
                pointer: {{ result_storage }},
                alignment: {{ result_type.alignment }},
                metadata: [],
            };
        );
        Some(EmittedValue {
            ty: Type::Symbol,
            representation: result,
            owned: true,
        })
    }

    fn atom_has_slot(&self, atom: &Atom) -> bool {
        matches!(atom.kind, AtomKind::Reference(Reference::Binding(id)) if self.slots.contains_key(&id))
    }

    fn take_symbol(&mut self, atom: &Atom) -> Option<EmittedValue> {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return None;
        };
        let slot = self.slots.get(&id)?;
        if slot.ty != Type::Symbol {
            return None;
        }
        let slot_index = slot.index;
        let value = self.atom(atom)?;
        let value_type = self.types.value(&Type::Symbol)?;
        emit_instruction!(
            self;
            store {
                value: typed({{ value_type.llvm }}, "zeroinitializer"),
                pointer: {{ format!("%mal_slot_{slot_index}") }},
                alignment: {{ value_type.alignment }},
                metadata: [],
            };
        );
        Some(EmittedValue {
            owned: true,
            ..value
        })
    }
}
