use crate::check::ast::Type;
use crate::core::ast::BufferOperation;

use super::super::{EmittedValue, FunctionEmitter};

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
        let index = self.types.pointer_integer()?;
        self.line(format!(
            "  {representation} = call ptr @mal_runtime_buffer_from(ptr %mal_context, ptr {address}, {index} {offset}, {index} {length}, {index} {stride})",
            address = address.representation,
            offset = offset.representation,
            length = length.representation,
        ));
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
        let index = self.types.pointer_integer()?;
        self.line(format!(
            "  call void @mal_runtime_buffer_into(ptr %mal_context, ptr {buffer}, ptr {address}, {index} {offset}, {index} {length}, {index} {stride})",
            buffer = buffer.representation,
            address = address.representation,
            offset = offset.representation,
            length = length.representation,
        ));
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
        let stride = self.source_layouts.layout(element)?.stride;
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
                self.line(format!(
                    "  {buffer} = call ptr @mal_runtime_buffer_make(ptr %mal_context, {0} {stride}, {0} {1})",
                    self.types.pointer_integer()?, capacity.representation
                ));
                Some(emitted_buffer(buffer, buffer_type))
            }
            BufferOperation::New => {
                let [buffer, value] = operands else {
                    return None;
                };
                if buffer.ty != buffer_type || value.ty != *element || *result_type != Type::USize {
                    return None;
                }
                let value_pointer = self.buffer_value_pointer(value, stride)?;
                let index = self.register();
                self.line(format!(
                    "  {index} = call {0} @mal_runtime_buffer_new(ptr %mal_context, ptr {1}, ptr {value_pointer}, {0} {stride})",
                    self.types.pointer_integer()?, buffer.representation
                ));
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
                    return Some(EmittedValue {
                        ty: Type::Unit,
                        representation: "0".into(),
                        owned: false,
                    });
                }
                let data = self.active_buffer_data(buffer)?;
                let pointer = self.buffer_element_pointer(&data, index, stride)?;
                self.emit_aligned_buffer_load_at(&pointer, element)
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
                    let data = self.active_buffer_data(buffer)?;
                    let pointer = self.buffer_element_pointer(&data, index, stride)?;
                    self.emit_aligned_buffer_store_at(&pointer, value)?;
                }
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
        }
    }

    pub(in crate::backend::llvm::body) fn active_buffer_data(
        &mut self,
        buffer: &EmittedValue,
    ) -> Option<String> {
        let slot = self.register();
        self.line(format!(
            "  {slot} = call ptr @mal_runtime_buffer_data_slot(ptr {})",
            buffer.representation
        ));
        let data = self.register();
        self.line(format!(
            "  {data} = load ptr, ptr {slot}, align {}, !alias.scope !6",
            self.types.pointer_alignment()
        ));
        Some(data)
    }

    fn buffer_value_pointer(&mut self, value: &EmittedValue, stride: usize) -> Option<String> {
        if stride == 0 {
            return Some("null".into());
        }
        let storage = "%mal_buffer_new_value";
        let layout = self.source_layouts.layout(&value.ty)?;
        self.line(format!(
            "  store [{stride} x i8] zeroinitializer, ptr {storage}, align {}",
            layout.alignment
        ));
        self.emit_aligned_source_store_at(storage, value)?;
        Some(storage.into())
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
            self.types.pointer_integer()?,
            index.representation
        ));
        let pointer = self.register();
        self.line(format!(
            "  {pointer} = getelementptr i8, ptr {data}, {} {offset}",
            self.types.pointer_integer()?
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
