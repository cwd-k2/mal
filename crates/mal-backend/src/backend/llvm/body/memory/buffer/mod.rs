use crate::backend::llvm::syntax::emit_instruction;
mod access;
mod runtime_owned;

use crate::backend::llvm::syntax::{BinaryOperator, MetadataAttachment};
use crate::core::ast::BufferOperation;
use crate::execution::ownership::{Lifecycle, lifecycle};
use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};
pub(in crate::backend::llvm::body) use runtime_owned::OwnedBufferElements;

/// How a Buffer keeps one element. Layout is independent from whether the place must preserve an
/// owned lifecycle through callbacks collected by [`OwnedBufferElements`].
#[derive(Clone, Copy)]
pub(in crate::backend::llvm::body) struct ElementStorage {
    stride: usize,
    alignment: usize,
    lifecycle: Lifecycle,
}

impl ElementStorage {
    fn stride(self) -> usize {
        self.stride
    }

    fn lifecycle(self) -> Lifecycle {
        self.lifecycle
    }

    /// The runtime function for `operation`. A buffer with managed elements must go through the variant that accounts
    /// for the references its elements own; the plain function stays free of that cost.
    fn runtime(self, operation: &str) -> String {
        match self.lifecycle {
            Lifecycle::Trivial => format!("mal_runtime_buffer_{operation}"),
            Lifecycle::Owned => format!("mal_runtime_buffer_{operation}_managed"),
        }
    }
}

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_buffer(
        &mut self,
        operation: BufferOperation,
        element: &Type,
        operands: &[EmittedValue],
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let storage = self.buffer_element_storage(element)?;
        let stride = storage.stride();
        let buffer_type = Type::Buffer(element.clone().into());
        match operation {
            BufferOperation::Make => {
                let [capacity] = operands else {
                    return None;
                };
                if capacity.ty != Type::USize || *result_type != buffer_type {
                    return None;
                }
                let buffer = self.register();
                if storage.lifecycle() == Lifecycle::Owned {
                    let number = self.index.owned_buffer_elements.number(element)?;
                    emit_instruction! {
                        self;
                        let { buffer.clone() } = call {
                            tail: false,
                            result_type: ptr,
                            callee: direct("mal_runtime_buffer_make_managed"),
                            arguments: [
                                (ptr, "%mal_context"),
                                ({ self.types.index_llvm_type() }, { stride.to_string() }),
                                ({ self.types.index_llvm_type() }, { capacity.representation.clone() }),
                                (ptr, { format!("@mal_buffer_retain_{number}") }),
                                (ptr, { format!("@mal_buffer_release_{number}") }),
                            ],
                        };
                    };
                } else {
                    emit_instruction! {
                        self;
                        let { buffer.clone() } = call {
                            tail: false,
                            result_type: ptr,
                            callee: direct("mal_runtime_buffer_make"),
                            arguments: [
                                (ptr, "%mal_context"),
                                ({ self.types.index_llvm_type() }, { stride.to_string() }),
                                ({ self.types.index_llvm_type() }, { capacity.representation.clone() }),
                            ],
                        };
                    };
                }
                Some(emitted_buffer(buffer, buffer_type))
            }
            BufferOperation::New => {
                let [buffer, value] = operands else {
                    return None;
                };
                if buffer.ty != buffer_type || value.ty != *element || *result_type != Type::USize {
                    return None;
                }
                if storage.lifecycle() == Lifecycle::Owned && !value.owned {
                    return None;
                }
                let value_pointer = self.buffer_value_pointer(value, storage)?;
                let index = self.register();
                let function = match storage.lifecycle() {
                    Lifecycle::Trivial => "mal_runtime_buffer_new",
                    Lifecycle::Owned => "mal_runtime_buffer_new_managed_move",
                };
                emit_instruction! {
                    self;
                    let { index.clone() } = call {
                        tail: false,
                        result_type: { self.types.index_llvm_type() },
                        callee: direct({ function }),
                        arguments: [
                            (ptr, "%mal_context"),
                            (ptr, { buffer.representation.clone() }),
                            (ptr, { value_pointer }),
                            ({ self.types.index_llvm_type() }, { stride.to_string() }),
                        ],
                    };
                };
                Some(EmittedValue {
                    ty: Type::USize,
                    representation: index,
                    owned: false,
                })
            }
            BufferOperation::Get => {
                let [buffer, index] = operands else {
                    return None;
                };
                if buffer.ty != buffer_type || index.ty != Type::USize || result_type != element {
                    return None;
                }
                if *element == Type::Unit {
                    return Some(emitted_unit());
                }
                let data = self.active_buffer_data(buffer);
                let pointer = self.buffer_element_pointer(&data, index, stride)?;
                self.emit_runtime_element_get(
                    &pointer,
                    element,
                    storage.alignment,
                    storage.lifecycle,
                )
            }
            BufferOperation::Put => {
                let [buffer, index, value] = operands else {
                    return None;
                };
                if buffer.ty != buffer_type
                    || index.ty != Type::USize
                    || value.ty != *element
                    || *result_type != Type::Unit
                {
                    return None;
                }
                if stride != 0 {
                    let data = self.active_buffer_data(buffer);
                    let pointer = self.buffer_element_pointer(&data, index, stride)?;
                    self.emit_runtime_element_put(
                        &pointer,
                        value,
                        storage.alignment,
                        storage.lifecycle,
                    )?;
                }
                Some(emitted_unit())
            }
            BufferOperation::Fill => {
                let [buffer, offset, length, value] = operands else {
                    return None;
                };
                if buffer.ty != buffer_type
                    || offset.ty != Type::USize
                    || length.ty != Type::USize
                    || value.ty != *element
                    || *result_type != Type::Unit
                {
                    return None;
                }
                let value_pointer = self.buffer_value_pointer(value, storage)?;
                let function = storage.runtime("fill");
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: void,
                        callee: direct({ function }),
                        arguments: [
                            (ptr, "%mal_context"),
                            (ptr, { buffer.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { offset.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { length.representation.clone() }),
                            (ptr, { value_pointer }),
                            ({ self.types.index_llvm_type() }, { stride.to_string() }),
                        ],
                    };
                };
                Some(emitted_unit())
            }
            BufferOperation::Copy => {
                let [
                    destination,
                    destination_offset,
                    source,
                    source_offset,
                    length,
                ] = operands
                else {
                    return None;
                };
                if destination.ty != buffer_type
                    || destination_offset.ty != Type::USize
                    || source.ty != buffer_type
                    || source_offset.ty != Type::USize
                    || length.ty != Type::USize
                    || *result_type != Type::Unit
                {
                    return None;
                }
                let function = storage.runtime("copy");
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: void,
                        callee: direct({ function }),
                        arguments: [
                            (ptr, "%mal_context"),
                            (ptr, { destination.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { destination_offset.representation.clone() }),
                            (ptr, { source.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { source_offset.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { length.representation.clone() }),
                            ({ self.types.index_llvm_type() }, { stride.to_string() }),
                        ],
                    };
                };
                Some(emitted_unit())
            }
        }
    }

    pub(in crate::backend::llvm::body) fn buffer_element_storage(
        &self,
        element: &Type,
    ) -> Option<ElementStorage> {
        let value = self.types.value(element)?;
        Some(ElementStorage {
            stride: value.size,
            alignment: value.alignment,
            lifecycle: lifecycle(element),
        })
    }
}

pub(super) fn emitted_buffer(representation: String, ty: Type) -> EmittedValue {
    EmittedValue {
        ty,
        representation,
        owned: true,
    }
}

pub(in crate::backend::llvm::body) fn emitted_unit() -> EmittedValue {
    EmittedValue {
        ty: Type::Unit,
        representation: "0".into(),
        owned: false,
    }
}
