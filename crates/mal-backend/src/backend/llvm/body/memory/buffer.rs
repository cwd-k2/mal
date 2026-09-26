use crate::core::ast::BufferOperation;
use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

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

/// The element types of the program's Buffers that own managed values. Each has one retain and one release callback,
/// numbered by position.
pub(in crate::backend::llvm::body) struct ManagedBufferElements(Vec<Type>);

impl ManagedBufferElements {
    pub(in crate::backend::llvm::body) fn collect(execution: &crate::execution::Program) -> Self {
        let mut elements: Vec<Type> = Vec::new();
        for binding in execution
            .control
            .states
            .iter()
            .flat_map(|state| &state.bindings)
        {
            if let crate::control::ast::Operation::Buffer {
                operation: BufferOperation::Make,
                element,
                ..
            } = &binding.operation
                && crate::execution::ownership::is_managed(element)
                && !elements.contains(element)
            {
                elements.push(element.clone());
            }
        }
        Self(elements)
    }

    fn number(&self, element: &Type) -> Option<usize> {
        self.0.iter().position(|candidate| candidate == element)
    }
}

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_buffer_from_address(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Buffer(element) = result_type else {
            return None;
        };
        let argument_type = Type::Product(vec![Type::Address, Type::USize, Type::USize].into());
        if argument.ty != argument_type {
            return None;
        }
        let [address, offset, length] =
            self.product_fields(argument, [&Type::Address, &Type::USize, &Type::USize])?;
        let stride = self.source_layouts.layout(element)?.stride;
        let representation = self.register();
        self.direct_call(
            Some(representation.clone()),
            false,
            crate::backend::llvm::syntax::Type::Pointer,
            "mal_runtime_buffer_from",
            [
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    "%mal_context".into(),
                ),
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    address.representation,
                ),
                (self.types.index_llvm_type(), offset.representation),
                (self.types.index_llvm_type(), length.representation),
                (self.types.index_llvm_type(), stride.to_string()),
            ],
        );
        Some(emitted_buffer(representation, result_type.clone()))
    }

    pub(in crate::backend::llvm::body) fn emit_buffer_into_address(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(elements) = &argument.ty else {
            return None;
        };
        let [buffer_type, address_type, offset_type, length_type] = elements.as_ref() else {
            return None;
        };
        let Type::Buffer(element) = buffer_type else {
            return None;
        };
        if address_type != &Type::Address
            || offset_type != &Type::USize
            || length_type != &Type::USize
            || result_type != &Type::Unit
        {
            return None;
        }
        let [buffer, address, offset, length] = self.product_fields(
            argument,
            [buffer_type, address_type, offset_type, length_type],
        )?;
        let stride = self.source_layouts.layout(element)?.stride;
        self.direct_call(
            None,
            false,
            crate::backend::llvm::syntax::Type::Void,
            "mal_runtime_buffer_into",
            [
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    "%mal_context".into(),
                ),
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    buffer.representation,
                ),
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    address.representation,
                ),
                (self.types.index_llvm_type(), offset.representation),
                (self.types.index_llvm_type(), length.representation),
                (self.types.index_llvm_type(), stride.to_string()),
            ],
        );
        Some(EmittedValue {
            ty: Type::Unit,
            representation: "0".into(),
            owned: false,
        })
    }

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
                    self.direct_call(
                        Some(buffer.clone()),
                        false,
                        crate::backend::llvm::syntax::Type::Pointer,
                        "mal_runtime_buffer_make_managed",
                        [
                            (
                                crate::backend::llvm::syntax::Type::Pointer,
                                "%mal_context".into(),
                            ),
                            (self.types.index_llvm_type(), stride.to_string()),
                            (
                                self.types.index_llvm_type(),
                                capacity.representation.clone(),
                            ),
                            (
                                crate::backend::llvm::syntax::Type::Pointer,
                                format!("@mal_buffer_retain_{number}"),
                            ),
                            (
                                crate::backend::llvm::syntax::Type::Pointer,
                                format!("@mal_buffer_release_{number}"),
                            ),
                        ],
                    );
                } else {
                    self.direct_call(
                        Some(buffer.clone()),
                        false,
                        crate::backend::llvm::syntax::Type::Pointer,
                        "mal_runtime_buffer_make",
                        [
                            (
                                crate::backend::llvm::syntax::Type::Pointer,
                                "%mal_context".into(),
                            ),
                            (self.types.index_llvm_type(), stride.to_string()),
                            (
                                self.types.index_llvm_type(),
                                capacity.representation.clone(),
                            ),
                        ],
                    );
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
                self.direct_call(
                    Some(index.clone()),
                    false,
                    self.types.index_llvm_type(),
                    function,
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            buffer.representation.clone(),
                        ),
                        (crate::backend::llvm::syntax::Type::Pointer, value_pointer),
                        (self.types.index_llvm_type(), stride.to_string()),
                    ],
                );
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
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    function,
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            buffer.representation.clone(),
                        ),
                        (self.types.index_llvm_type(), offset.representation.clone()),
                        (self.types.index_llvm_type(), length.representation.clone()),
                        (crate::backend::llvm::syntax::Type::Pointer, value_pointer),
                        (self.types.index_llvm_type(), stride.to_string()),
                    ],
                );
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
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    function,
                    [
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            "%mal_context".into(),
                        ),
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            destination.representation.clone(),
                        ),
                        (
                            self.types.index_llvm_type(),
                            destination_offset.representation.clone(),
                        ),
                        (
                            crate::backend::llvm::syntax::Type::Pointer,
                            source.representation.clone(),
                        ),
                        (
                            self.types.index_llvm_type(),
                            source_offset.representation.clone(),
                        ),
                        (self.types.index_llvm_type(), length.representation.clone()),
                        (self.types.index_llvm_type(), stride.to_string()),
                    ],
                );
                Some(emitted_unit())
            }
        }
    }

    pub(in crate::backend::llvm::body) fn active_buffer_data(
        &mut self,
        buffer: &EmittedValue,
    ) -> String {
        let slot = self.register();
        self.direct_call(
            Some(slot.clone()),
            false,
            crate::backend::llvm::syntax::Type::Pointer,
            "mal_runtime_buffer_data_slot",
            [(
                crate::backend::llvm::syntax::Type::Pointer,
                buffer.representation.clone(),
            )],
        );
        let data = self.register();
        self.load(
            data.clone(),
            crate::backend::llvm::syntax::Type::Pointer,
            slot,
            self.types.pointer_alignment(),
            [
                crate::backend::llvm::syntax::MetadataAttachment::Tbaa(8),
                crate::backend::llvm::syntax::MetadataAttachment::AliasScope(6),
            ],
        );
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
                self.store(
                    crate::backend::llvm::syntax::Type::array(
                        stride,
                        crate::backend::llvm::syntax::Type::integer(8_u16),
                    ),
                    "zeroinitializer",
                    storage,
                    layout.alignment,
                    [],
                );
                self.emit_aligned_source_store_at(storage, value)?;
            }
            ElementStorage::Managed { alignment, .. } => {
                let value_type = self.types.value(&value.ty)?;
                self.store(
                    value_type.llvm,
                    value.representation.as_str(),
                    storage,
                    alignment,
                    [],
                );
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
        self.load(loaded.clone(), value_type.llvm, pointer, alignment, []);
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
        self.load(
            previous.clone(),
            value_type.llvm.clone(),
            pointer,
            alignment,
            [],
        );
        self.release_value(&value.ty, &previous)?;
        self.store(
            value_type.llvm,
            value.representation.as_str(),
            pointer,
            alignment,
            [],
        );
        Some(())
    }

    /// Defines the callbacks that let the runtime retain and release one stored element of each managed element type.
    pub(in crate::backend::llvm::body) fn emit_managed_buffer_element_callbacks(
        &mut self,
    ) -> Option<super::super::EmittedFunction> {
        let index = self.index;
        for (number, element) in index.managed_buffer_elements.0.iter().enumerate() {
            self.emit_managed_element_callback(number, element, true)?;
            self.emit_managed_element_callback(number, element, false)?;
        }
        (!self.emission_failed).then(|| super::super::EmittedFunction {
            globals: std::mem::take(&mut self.globals),
            definitions: std::mem::take(&mut self.definitions),
        })
    }

    fn emit_managed_element_callback(
        &mut self,
        number: usize,
        element: &Type,
        retain: bool,
    ) -> Option<()> {
        let value_type = self.types.value(element)?;
        let signature = if retain {
            crate::backend::llvm::syntax::FunctionSignature::new(
                "void",
                format!("mal_buffer_retain_{number}"),
                ["ptr %mal_context", "ptr %mal_element"],
            )
        } else {
            crate::backend::llvm::syntax::FunctionSignature::new(
                "void",
                format!("mal_buffer_release_{number}"),
                ["ptr %mal_element"],
            )
        };
        self.begin_function(signature.with_linkage("internal"));
        self.block("entry");
        let value = self.register();
        self.load(
            value.clone(),
            value_type.llvm,
            "%mal_element",
            value_type.alignment,
            [],
        );
        if retain {
            self.retain_value(element, &value)?;
        } else {
            self.release_value(element, &value)?;
        }
        self.return_void();
        self.finish_function()
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
        self.line(format!(
            "  {offset} = mul {} {}, {stride}",
            self.types.index_integer(),
            index.representation
        ));
        let pointer = self.register();
        self.line(format!(
            "  {pointer} = getelementptr i8, ptr {data}, {} {offset}",
            self.types.index_integer()
        ));
        Some(pointer)
    }
}

fn emitted_buffer(representation: String, ty: Type) -> EmittedValue {
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
