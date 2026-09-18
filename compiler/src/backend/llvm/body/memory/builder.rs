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
        match operation {
            PackedBuilderOperation::Start => {
                if argument.ty != Type::Unit || *result_type != Type::Address {
                    return None;
                }
                let builder = self.register();
                self.line(format!(
                    "  {builder} = call ptr @mal_runtime_packed_builder_start(ptr %mal_context, {} {stride})",
                    self.types.pointer_integer()?
                ));
                Some(address(builder))
            }
            PackedBuilderOperation::Edit => {
                let packed_type = Type::Packed(element.clone().into());
                if argument.ty != packed_type || *result_type != Type::Address {
                    return None;
                }
                let ByteViewFields { owner, data, count } = self.byte_view_fields(argument)?;
                let builder = self.register();
                self.line(format!(
                    "  {builder} = call ptr @mal_runtime_packed_builder_edit(ptr %mal_context, ptr {owner}, ptr {data}, {0} {count}, {0} {stride})",
                    self.types.pointer_integer()?
                ));
                Some(address(builder))
            }
            PackedBuilderOperation::New | PackedBuilderOperation::NewUnique => {
                let argument_type = Type::Product(vec![Type::Address, element.clone()].into());
                if argument.ty != argument_type || *result_type != Type::USize {
                    return None;
                }
                let [builder, value] = self.product_fields(argument, [&Type::Address, element])?;
                let value_pointer = self.builder_value_pointer(&value, stride)?;
                let index = self.register();
                let (operation, stride_argument) = if operation == PackedBuilderOperation::NewUnique
                {
                    (
                        "mal_runtime_packed_builder_new_unique",
                        format!(", {} {stride}", self.types.pointer_integer()?),
                    )
                } else {
                    ("mal_runtime_packed_builder_new", String::new())
                };
                self.line(format!(
                    "  {index} = call {} @{operation}(ptr %mal_context, ptr {}, ptr {value_pointer}{stride_argument})",
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
                let argument_type = Type::Product(vec![Type::Address, Type::USize].into());
                if argument.ty != argument_type || result_type != element {
                    return None;
                }
                let [builder, index] =
                    self.product_fields(argument, [&Type::Address, &Type::USize])?;
                if *element == Type::Unit {
                    return Some(EmittedValue {
                        ty: Type::Unit,
                        representation: "0".into(),
                        owned: false,
                    });
                }
                let pointer = self.register();
                self.line(format!(
                    "  {pointer} = call ptr @mal_runtime_packed_builder_get(ptr {}, {} {}, {} {stride})",
                    builder.representation,
                    self.types.pointer_integer()?,
                    index.representation,
                    self.types.pointer_integer()?
                ));
                self.emit_source_load_at(&pointer, element)
            }
            PackedBuilderOperation::Put => {
                let put_type = Type::Product(vec![Type::USize, element.clone()].into());
                let argument_type = Type::Product(vec![Type::Address, put_type.clone()].into());
                if argument.ty != argument_type || *result_type != Type::Unit {
                    return None;
                }
                let [builder, put] = self.product_fields(argument, [&Type::Address, &put_type])?;
                let [index, value] = self.product_fields(&put, [&Type::USize, element])?;
                let value_pointer = self.builder_value_pointer(&value, stride)?;
                self.line(format!(
                    "  call void @mal_runtime_packed_builder_put(ptr %mal_context, ptr {}, {} {}, ptr {value_pointer}, {} {stride})",
                    builder.representation,
                    self.types.pointer_integer()?,
                    index.representation,
                    self.types.pointer_integer()?
                ));
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
            PackedBuilderOperation::PutUnique => {
                let put_type = Type::Product(vec![Type::USize, element.clone()].into());
                let argument_type = Type::Product(vec![Type::Address, put_type.clone()].into());
                if argument.ty != argument_type || *result_type != Type::Unit {
                    return None;
                }
                let [builder, put] = self.product_fields(argument, [&Type::Address, &put_type])?;
                let [index, value] = self.product_fields(&put, [&Type::USize, element])?;
                let value_pointer = self.builder_value_pointer(&value, stride)?;
                self.line(format!(
                    "  call void @mal_runtime_packed_builder_put_unique(ptr {}, {} {}, ptr {value_pointer}, {} {stride})",
                    builder.representation,
                    self.types.pointer_integer()?,
                    index.representation,
                    self.types.pointer_integer()?
                ));
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
            PackedBuilderOperation::Finish => {
                let packed_type = Type::Packed(element.clone().into());
                if argument.ty != Type::Address || *result_type != packed_type {
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

    fn builder_value_pointer(&mut self, value: &EmittedValue, stride: usize) -> Option<String> {
        if stride == 0 {
            return Some("null".into());
        }
        let storage = self.register();
        self.line(format!(
            "  {storage} = alloca i8, {} {stride}, align 1",
            self.types.pointer_integer()?
        ));
        self.emit_source_store_at(&storage, value)?;
        Some(storage)
    }
}

fn address(representation: String) -> EmittedValue {
    EmittedValue {
        ty: Type::Address,
        representation,
        owned: false,
    }
}
