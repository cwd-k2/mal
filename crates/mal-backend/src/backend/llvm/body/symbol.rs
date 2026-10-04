use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::{
    BinaryOperator, ComparisonKind, ComparisonPredicate, llvm_global,
};
use crate::closure::ast::{Atom, AtomKind, Reference};
use mal_frontend::check::ast::{SymbolPrimitive, Type};

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
    llvm_global! {
        byte_owner {
            name: { name },
            bytes: { bytes.to_vec() },
            alignment: { STATIC_OWNER_ALIGNMENT },
        }
    }
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

    pub(super) fn emit_symbol_at(&mut self, symbol: &Atom, index: &Atom) -> Option<EmittedValue> {
        let symbol = self.atom(symbol)?;
        let index = self.atom(index)?;
        let data = self.byte_view_fields(&symbol)?.data;
        let result = self.register();
        emit_instruction! {
            self;
            let { result.clone() } = call {
                tail: false,
                result_type: int(8_u16),
                callee: direct("mal_runtime_symbol_at"),
                arguments: [
                    (ptr, { data }),
                    ({ self.types.index_llvm_type() }, { index.representation }),
                ],
            };
        };
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
        emit_instruction! {
            self;
            call {
                tail: false,
                result_type: void,
                callee: direct({ operation }),
                arguments: [
                    (ptr, "%mal_context"),
                    (ptr, { result_storage }),
                    (ptr, { left_owner }),
                    (ptr, { left_data }),
                    ({ self.types.index_llvm_type() }, { left_length }),
                    (ptr, { right_owner }),
                    (ptr, { right_data }),
                    ({ self.types.index_llvm_type() }, { right_length }),
                ],
            };
        };
        let result = self.register();
        emit_instruction! {
            self;
            let { result.clone() } = load {
                ty: { result_type.llvm },
                pointer: { result_storage },
                alignment: { result_type.alignment },
                metadata: [],
            };
        };
        Some(EmittedValue {
            ty: Type::Symbol,
            representation: result,
            owned: true,
        })
    }

    pub(super) fn emit_symbol_partition(
        &mut self,
        symbol: &Atom,
        index: &Atom,
        primitive: SymbolPrimitive,
    ) -> Option<EmittedValue> {
        let symbol = self.atom(symbol)?;
        let index = self.atom(index)?;
        if symbol.ty != Type::Symbol || index.ty != Type::USize {
            return None;
        }
        let ByteViewFields { owner, data, count } = self.byte_view_fields(&symbol)?;
        let (offset, length) = match primitive {
            SymbolPrimitive::Prefix => ("0".into(), index.representation),
            SymbolPrimitive::Suffix => {
                let length = self.register();
                emit_instruction! {
                    self;
                    let { length.clone() } = binary {
                        operator: { BinaryOperator::Sub },
                        ty: { self.types.index_llvm_type() },
                        left: { count },
                        right: { index.representation.clone() },
                    };
                };
                (index.representation, length)
            }
            _ => return None,
        };
        let result_type = self.types.value(&Type::Symbol)?;
        if !self.needs_symbol_result_slot {
            return None;
        }
        let result_storage = "%mal_symbol_result";
        emit_instruction! {
            self;
            call {
                tail: false,
                result_type: void,
                callee: direct("mal_runtime_symbol_slice"),
                arguments: [
                    (ptr, "%mal_context"),
                    (ptr, { result_storage }),
                    (ptr, { owner }),
                    (ptr, { data }),
                    ({ self.types.index_llvm_type() }, { offset }),
                    ({ self.types.index_llvm_type() }, { length }),
                ],
            };
        };
        let result = self.register();
        emit_instruction! {
            self;
            let { result.clone() } = load {
                ty: { result_type.llvm },
                pointer: { result_storage },
                alignment: { result_type.alignment },
                metadata: [],
            };
        };
        Some(EmittedValue {
            ty: Type::Symbol,
            representation: result,
            owned: true,
        })
    }

    pub(in crate::backend::llvm::body) fn atom_has_slot(&self, atom: &Atom) -> bool {
        matches!(atom.kind, AtomKind::Reference(Reference::Binding(id)) if self.slots.contains_key(&id))
    }

    fn take_symbol(&mut self, atom: &Atom) -> Option<EmittedValue> {
        self.take_binding(atom, &Type::Symbol)
    }
}

impl FunctionEmitter<'_> {
    /// Byte-wise `==` or `!=` of two Symbols as an LLVM `i1`.
    pub(super) fn emit_symbol_equality(
        &mut self,
        left: &Atom,
        right: &Atom,
        primitive: SymbolPrimitive,
    ) -> Option<EmittedValue> {
        let predicate = match primitive {
            SymbolPrimitive::Equal => ComparisonPredicate::Ne,
            SymbolPrimitive::NotEqual => ComparisonPredicate::Eq,
            _ => return None,
        };
        let left = self.atom(left)?;
        let right = self.atom(right)?;
        let left = self.byte_view_fields(&left)?;
        let right = self.byte_view_fields(&right)?;
        let equality = self.register();
        emit_instruction! {
            self;
            let { equality.clone() } = call {
                tail: false,
                result_type: int(8_u16),
                callee: direct("mal_runtime_symbol_equal"),
                arguments: [
                    (ptr, { left.data }),
                    ({ self.types.index_llvm_type() }, { left.count }),
                    (ptr, { right.data }),
                    ({ self.types.index_llvm_type() }, { right.count }),
                ],
            };
        };
        let register = self.register();
        emit_instruction! {
            self;
            let { register.clone() } = compare {
                kind: { ComparisonKind::Integer },
                predicate: { predicate },
                ty: int(8_u16),
                left: { equality },
                right: "0",
            };
        };
        Some(EmittedValue {
            ty: Type::Sum(vec![Type::Unit, Type::Unit].into()),
            representation: register,
            owned: false,
        })
    }
}
