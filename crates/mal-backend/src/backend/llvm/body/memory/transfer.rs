//! Byte `*` at the operand's last use: the operand leaves its slot and the runtime takes its reference, moving the
//! byte owner to the result when nothing else holds it.

use crate::closure::ast::Atom;
use mal_frontend::check::ast::{MemoryPrimitive, Type};

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_byte_conversion_transfer(
        &mut self,
        primitive: MemoryPrimitive,
        operand: &Atom,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let byte_buffer = Type::Buffer(Type::UInt8.into());
        match primitive {
            MemoryPrimitive::BufferToSymbol if *result_type == Type::Symbol => {
                let buffer = self.take_binding(operand, &byte_buffer)?;
                // The count is read first because the runtime may release the buffer.
                let count = self.register();
                emit_instruction! {
                    self;
                    let #{ count.clone() } = call {
                        tail: false,
                        result_type: #{ self.types.index_llvm_type() },
                        callee: direct("mal_runtime_buffer_count"),
                        arguments: [typed((ptr), #{ buffer.representation.clone() })],
                    };
                };
                let owner = self.register();
                emit_instruction! {
                    self;
                    let #{ owner.clone() } = call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_buffer_into_symbol"),
                        arguments: [
                            typed((ptr), "%mal_context"),
                            typed((ptr), #{ buffer.representation }),
                        ],
                    };
                };
                let data = self.register();
                emit_instruction! {
                    self;
                    let #{ data.clone() } = call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_bytes_data"),
                        arguments: [typed((ptr), #{ owner.clone() })],
                    };
                };
                self.make_byte_view(&Type::Symbol, &owner, &data, &count, true)
            }
            MemoryPrimitive::SymbolToBuffer if *result_type == byte_buffer => {
                let symbol = self.take_binding(operand, &Type::Symbol)?;
                let fields = self.byte_view_fields(&symbol)?;
                let buffer = self.register();
                emit_instruction! {
                    self;
                    let #{ buffer.clone() } = call {
                        tail: false,
                        result_type: (ptr),
                        callee: direct("mal_runtime_symbol_into_buffer"),
                        arguments: [
                            typed((ptr), "%mal_context"),
                            typed((ptr), #{ fields.owner }),
                            typed((ptr), #{ fields.data }),
                            typed(#{ self.types.index_llvm_type() }, #{ fields.count }),
                        ],
                    };
                };
                Some(EmittedValue {
                    ty: byte_buffer,
                    representation: buffer,
                    owned: true,
                })
            }
            _ => None,
        }
    }
}
