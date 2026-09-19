use crate::check::ast::Type;
use crate::core::ast::PackedBuilderOperation;

use super::super::{EmittedValue, FunctionEmitter};
use super::ByteViewFields;

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_packed_builder(
        &mut self,
        operation: PackedBuilderOperation,
        element: &Type,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let stride = self.source_layouts.layout(element)?.stride;
        let buffer_type = Type::Buffer(element.clone().into());
        match operation {
            PackedBuilderOperation::Start => {
                if argument.ty != Type::Unit || *result_type != buffer_type {
                    return None;
                }
                let builder = self.register();
                self.line(format!(
                    "  {builder} = call ptr @mal_runtime_packed_builder_start(ptr %mal_context, {} {stride})",
                    self.types.pointer_integer()?
                ));
                Some(buffer(builder, buffer_type))
            }
            PackedBuilderOperation::Edit => {
                let packed_type = Type::Packed(element.clone().into());
                if argument.ty != packed_type || *result_type != buffer_type {
                    return None;
                }
                let ByteViewFields { owner, data, count } = self.byte_view_fields(argument)?;
                let builder = self.register();
                self.line(format!(
                    "  {builder} = call ptr @mal_runtime_packed_builder_edit(ptr %mal_context, ptr {owner}, ptr {data}, {0} {count}, {0} {stride})",
                    self.types.pointer_integer()?
                ));
                Some(buffer(builder, buffer_type))
            }
            PackedBuilderOperation::Prepare => {
                if argument.ty != buffer_type || *result_type != Type::Unit {
                    return None;
                }
                self.line(format!(
                    "  call ptr @mal_runtime_packed_builder_prepare_edit(ptr %mal_context, ptr {})",
                    argument.representation
                ));
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
            PackedBuilderOperation::New => {
                let argument_type =
                    Type::Product(vec![buffer_type.clone(), element.clone()].into());
                if argument.ty != argument_type || *result_type != Type::USize {
                    return None;
                }
                let [builder, value] = self.product_fields(argument, [&buffer_type, element])?;
                let value_pointer = self.builder_value_pointer(&value, stride)?;
                let index = self.register();
                self.line(format!(
                    "  {index} = call {} @mal_runtime_packed_builder_new(ptr %mal_context, ptr {}, ptr {value_pointer})",
                    self.types.pointer_integer()?,
                    builder.representation
                ));
                Some(EmittedValue {
                    ty: Type::USize,
                    representation: index,
                    owned: false,
                })
            }
            PackedBuilderOperation::Get => {
                let argument_type = Type::Product(vec![buffer_type.clone(), Type::USize].into());
                if argument.ty != argument_type || result_type != element {
                    return None;
                }
                let [builder, index] =
                    self.product_fields(argument, [&buffer_type, &Type::USize])?;
                if *element == Type::Unit {
                    return Some(EmittedValue {
                        ty: Type::Unit,
                        representation: "0".into(),
                        owned: false,
                    });
                }
                let data = self.active_buffer_data(&builder)?;
                let pointer = self.builder_element_pointer(&data, &index, stride)?;
                self.emit_aligned_builder_load_at(&pointer, element)
            }
            PackedBuilderOperation::Put => {
                let put_type = Type::Product(vec![Type::USize, element.clone()].into());
                let argument_type =
                    Type::Product(vec![buffer_type.clone(), put_type.clone()].into());
                if argument.ty != argument_type || *result_type != Type::Unit {
                    return None;
                }
                let [builder, put] = self.product_fields(argument, [&buffer_type, &put_type])?;
                let [index, value] = self.product_fields(&put, [&Type::USize, element])?;
                if stride != 0 {
                    let data = self.active_buffer_data(&builder)?;
                    let pointer = self.builder_element_pointer(&data, &index, stride)?;
                    self.emit_aligned_builder_store_at(&pointer, &value)?;
                }
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
            PackedBuilderOperation::Finish => {
                let packed_type = Type::Packed(element.clone().into());
                if argument.ty != buffer_type || *result_type != packed_type {
                    return None;
                }
                let runtime = self.types.value(&packed_type)?;
                let storage = self.register();
                self.line(format!(
                    "  {storage} = alloca {}, align {}",
                    runtime.llvm, runtime.alignment
                ));
                self.line(format!(
                    "  call void @mal_runtime_packed_builder_finish(ptr {storage}, ptr {})",
                    argument.representation
                ));
                let result = self.register();
                self.line(format!(
                    "  {result} = load {}, ptr {storage}, align {}",
                    runtime.llvm, runtime.alignment
                ));
                Some(EmittedValue {
                    ty: packed_type,
                    representation: result,
                    owned: true,
                })
            }
        }
    }

    fn active_buffer_data(&mut self, buffer: &EmittedValue) -> Option<String> {
        if self.optimizations.uses_direct_buffer(self.current_function) {
            return Some(buffer.representation.clone());
        }
        let slot = self.register();
        self.line(format!(
            "  {slot} = call ptr @mal_runtime_packed_builder_data_slot(ptr {})",
            buffer.representation
        ));
        let data = self.register();
        self.line(format!(
            "  {data} = load ptr, ptr {slot}, align {}, !tbaa !4",
            self.types.pointer_alignment()
        ));
        Some(data)
    }

    fn builder_value_pointer(&mut self, value: &EmittedValue, stride: usize) -> Option<String> {
        if stride == 0 {
            return Some("null".into());
        }
        let storage = "%mal_packed_new_value";
        self.emit_aligned_source_store_at(storage, value)?;
        Some(storage.into())
    }

    fn builder_element_pointer(
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

fn buffer(representation: String, ty: Type) -> EmittedValue {
    EmittedValue {
        ty,
        representation,
        owned: false,
    }
}
