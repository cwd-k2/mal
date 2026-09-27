mod address;
mod managed;

use crate::backend::llvm::syntax::{BinaryOperator, MetadataAttachment, llvm_type};
use crate::core::ast::BufferOperation;
use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};
pub(in crate::backend::llvm::body) use managed::ManagedBufferElements;

/// How a Buffer keeps one element. An element with a canonical memory representation is stored in it, so that `from`
/// and `into` can copy the storage. An element that owns a `Symbol` has none and is stored as its runtime value; the
/// runtime then retains and releases stored elements through the callbacks of [`ManagedBufferElements`].
#[derive(Clone, Copy)]
pub(in crate::backend::llvm::body) enum ElementStorage {
    Canonical { stride: usize },
    Managed { stride: usize, alignment: usize },
}

impl ElementStorage {
    fn stride(self) -> usize {
        match self {
            Self::Canonical { stride } | Self::Managed { stride, .. } => stride,
        }
    }

    /// The runtime function for `operation`. A buffer with managed elements must go through the variant that accounts
    /// for the references its elements own; the plain function stays free of that cost.
    fn runtime(self, operation: &str) -> String {
        match self {
            Self::Canonical { .. } => format!("mal_runtime_buffer_{operation}"),
            Self::Managed { .. } => format!("mal_runtime_buffer_{operation}_managed"),
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
                if let ElementStorage::Managed { .. } = storage {
                    let number = self.index.managed_buffer_elements.number(element)?;
                    emit_instruction! {
                        self;
                        let {{ buffer.clone() }} = call {
                            tail: false,
                            result_type: (ptr),
                            callee: direct("mal_runtime_buffer_make_managed"),
                            arguments: [
                                typed((ptr), "%mal_context"),
                                typed({{ self.types.index_llvm_type() }}, {{ stride.to_string() }}),
                                typed({{ self.types.index_llvm_type() }}, {{ capacity.representation.clone() }}),
                                typed((ptr), {{ format!("@mal_buffer_retain_{number}") }}),
                                typed((ptr), {{ format!("@mal_buffer_release_{number}") }}),
                            ],
                        };
                    };
                } else {
                    emit_instruction! {
                        self;
                        let {{ buffer.clone() }} = call {
                            tail: false,
                            result_type: (ptr),
                            callee: direct("mal_runtime_buffer_make"),
                            arguments: [
                                typed((ptr), "%mal_context"),
                                typed({{ self.types.index_llvm_type() }}, {{ stride.to_string() }}),
                                typed({{ self.types.index_llvm_type() }}, {{ capacity.representation.clone() }}),
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
                let value_pointer = self.buffer_value_pointer(value, storage)?;
                let index = self.register();
                let function = storage.runtime("new");
                emit_instruction! {
                    self;
                    let {{ index.clone() }} = call {
                        tail: false,
                        result_type: {{ self.types.index_llvm_type() }},
                        callee: direct({{ function }}),
                        arguments: [
                            typed((ptr), "%mal_context"),
                            typed((ptr), {{ buffer.representation.clone() }}),
                            typed((ptr), {{ value_pointer }}),
                            typed({{ self.types.index_llvm_type() }}, {{ stride.to_string() }}),
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
                match storage {
                    ElementStorage::Managed { alignment, .. } => {
                        self.emit_managed_element_get(&pointer, element, alignment)
                    }
                    ElementStorage::Canonical { .. } => {
                        self.emit_aligned_buffer_load_at(&pointer, element)
                    }
                }
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
                    match storage {
                        ElementStorage::Managed { alignment, .. } => {
                            self.emit_managed_element_put(&pointer, value, alignment)?;
                        }
                        ElementStorage::Canonical { .. } => {
                            self.emit_aligned_buffer_store_at(&pointer, value)?;
                        }
                    }
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
                        result_type: (void),
                        callee: direct({{ function }}),
                        arguments: [
                            typed((ptr), "%mal_context"),
                            typed((ptr), {{ buffer.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ offset.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ length.representation.clone() }}),
                            typed((ptr), {{ value_pointer }}),
                            typed({{ self.types.index_llvm_type() }}, {{ stride.to_string() }}),
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
                        result_type: (void),
                        callee: direct({{ function }}),
                        arguments: [
                            typed((ptr), "%mal_context"),
                            typed((ptr), {{ destination.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ destination_offset.representation.clone() }}),
                            typed((ptr), {{ source.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ source_offset.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ length.representation.clone() }}),
                            typed({{ self.types.index_llvm_type() }}, {{ stride.to_string() }}),
                        ],
                    };
                };
                Some(emitted_unit())
            }
        }
    }

    pub(in crate::backend::llvm::body) fn active_buffer_data(
        &mut self,
        buffer: &EmittedValue,
    ) -> String {
        let slot = self.register();
        emit_instruction! {
            self;
            let {{ slot.clone() }} = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_buffer_data_slot"),
                arguments: [typed((ptr), {{ buffer.representation.clone() }})],
            };
        };
        let data = self.register();
        emit_instruction! {
            self;
            let {{ data.clone() }} = load {
                ty: (ptr),
                pointer: {{ slot }},
                alignment: {{ self.types.pointer_alignment() }},
                metadata: {{ [
                    MetadataAttachment::Tbaa(8),
                    MetadataAttachment::AliasScope(6),
                ] }},
            };
        };
        data
    }

    pub(in crate::backend::llvm::body) fn buffer_element_storage(
        &self,
        element: &Type,
    ) -> Option<ElementStorage> {
        if let Some(layout) = self.source_layouts.layout(element) {
            return Some(ElementStorage::Canonical {
                stride: layout.stride,
            });
        }
        if !crate::execution::ownership::is_managed(element) {
            return None;
        }
        let value = self.types.value(element)?;
        Some(ElementStorage::Managed {
            stride: value.size,
            alignment: value.alignment,
        })
    }

    fn buffer_value_pointer(
        &mut self,
        value: &EmittedValue,
        element_storage: ElementStorage,
    ) -> Option<String> {
        if element_storage.stride() == 0 {
            return Some("null".into());
        }
        let storage = "%mal_buffer_value";
        match element_storage {
            ElementStorage::Canonical { stride } => {
                let layout = self.source_layouts.layout(&value.ty)?;
                emit_instruction! {
                    self;
                    store {
                        value: typed({{ llvm_type!(array({{ stride }}, int(8_u16))) }}, "zeroinitializer"),
                        pointer: {{ storage }},
                        alignment: {{ layout.alignment }},
                        metadata: [],
                    };
                };
                self.emit_aligned_source_store_at(storage, value)?;
            }
            ElementStorage::Managed { alignment, .. } => {
                let value_type = self.types.value(&value.ty)?;
                emit_instruction! {
                    self;
                    store {
                        value: typed({{ value_type.llvm }}, {{ value.representation.as_str() }}),
                        pointer: {{ storage }},
                        alignment: {{ alignment }},
                        metadata: [],
                    };
                };
            }
        }
        Some(storage.into())
    }

    /// The buffer keeps its own reference to the element, and a later `put` can drop it while the result is live, so
    /// the result takes a reference of its own.
    fn emit_managed_element_get(
        &mut self,
        pointer: &str,
        element: &Type,
        alignment: usize,
    ) -> Option<EmittedValue> {
        let value_type = self.types.value(element)?;
        let loaded = self.register();
        emit_instruction! {
            self;
            let {{ loaded.clone() }} = load {
                ty: {{ value_type.llvm }},
                pointer: {{ pointer }},
                alignment: {{ alignment }},
                metadata: [],
            };
        };
        self.retain_value(element, &loaded)?;
        Some(EmittedValue {
            ty: element.clone(),
            representation: loaded,
            owned: true,
        })
    }

    /// The stored value operand keeps its own reference through the call, so the old element may be the same value
    /// without being freed; the new reference is still taken first so the buffer never holds an element without one.
    fn emit_managed_element_put(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        alignment: usize,
    ) -> Option<()> {
        let value_type = self.types.value(&value.ty)?;
        self.retain_value(&value.ty, &value.representation)?;
        let previous = self.register();
        emit_instruction! {
            self;
            let {{ previous.clone() }} = load {
                ty: {{ value_type.llvm.clone() }},
                pointer: {{ pointer }},
                alignment: {{ alignment }},
                metadata: [],
            };
        };
        self.release_value(&value.ty, &previous)?;
        emit_instruction! {
            self;
            store {
                value: typed({{ value_type.llvm }}, {{ value.representation.as_str() }}),
                pointer: {{ pointer }},
                alignment: {{ alignment }},
                metadata: [],
            };
        };
        Some(())
    }

    fn buffer_element_pointer(
        &mut self,
        data: &str,
        index: &EmittedValue,
        stride: usize,
    ) -> Option<String> {
        if index.ty != Type::USize {
            return None;
        }
        let offset = self.register();
        emit_instruction! {
            self;
            let {{ offset.clone() }} = binary {
                operator: {{ BinaryOperator::Mul }},
                ty: {{ self.types.index_llvm_type() }},
                left: {{ index.representation.clone() }},
                right: {{ stride.to_string() }},
            };
        };
        let pointer = self.register();
        emit_instruction! {
            self;
            let {{ pointer.clone() }} = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: {{ data }},
                indices: [typed({{ self.types.index_llvm_type() }}, {{ offset }})],
            };
        };
        Some(pointer)
    }
}

pub(super) fn emitted_buffer(representation: String, ty: Type) -> EmittedValue {
    EmittedValue {
        ty,
        representation,
        owned: true,
    }
}

fn emitted_unit() -> EmittedValue {
    EmittedValue {
        ty: Type::Unit,
        representation: "0".into(),
        owned: false,
    }
}
