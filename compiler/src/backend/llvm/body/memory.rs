use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::Atom;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_memory(
        &mut self,
        primitive: MemoryPrimitive,
        argument: &Atom,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if !matches!(
            primitive,
            MemoryPrimitive::Place
                | MemoryPrimitive::Region
                | MemoryPrimitive::ProjectAddress
                | MemoryPrimitive::Align
                | MemoryPrimitive::LoadValue
                | MemoryPrimitive::StoreValue
        ) {
            let (parameter_type, expected_result) = primitive.signature();
            if argument.ty != parameter_type || *result_type != expected_result {
                return None;
            }
        }
        let argument = self.atom(argument)?;
        match primitive {
            MemoryPrimitive::OffsetForward | MemoryPrimitive::OffsetBackward => {
                let [pointer, offset] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::UInt64])?;
                let offset = if primitive == MemoryPrimitive::OffsetBackward {
                    let negated = self.register();
                    self.line(format!(
                        "  {negated} = sub i64 0, {}",
                        offset.representation
                    ));
                    negated
                } else {
                    offset.representation
                };
                let result = self.register();
                self.line(format!(
                    "  {result} = getelementptr i8, ptr {}, i64 {offset}",
                    pointer.representation
                ));
                Some(EmittedValue {
                    ty: Type::Ptr,
                    representation: result,
                    owned: false,
                })
            }
            MemoryPrimitive::Load(scalar) => self.emit_load(&argument, scalar.ty()),
            MemoryPrimitive::LoadPtr => self.emit_load(&argument, Type::Ptr),
            MemoryPrimitive::Store(scalar) => self.emit_store(&argument, &scalar.ty()),
            MemoryPrimitive::StorePtr => self.emit_store(&argument, &Type::Ptr),
            MemoryPrimitive::LoadSymbol => {
                let [pointer, length] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::UInt64])?;
                let result = self.register();
                self.line(format!(
                    "  {result} = call ptr @mal_runtime_symbol_read(ptr %mal_context, ptr {}, i64 {})",
                    pointer.representation, length.representation
                ));
                Some(EmittedValue {
                    ty: Type::Symbol,
                    representation: result,
                    owned: true,
                })
            }
            MemoryPrimitive::StoreSymbol => {
                let [pointer, symbol] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::Symbol])?;
                self.line(format!(
                    "  call void @mal_runtime_symbol_write(ptr {}, ptr {})",
                    pointer.representation, symbol.representation
                ));
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
            MemoryPrimitive::Place => {
                let Type::Cursor(element) = result_type else {
                    return None;
                };
                if argument.ty != Type::Address || self.types.source_layout(element).is_none() {
                    return None;
                }
                Some(EmittedValue {
                    ty: result_type.clone(),
                    representation: argument.representation,
                    owned: false,
                })
            }
            MemoryPrimitive::Region => {
                let Type::Region(element) = result_type else {
                    return None;
                };
                let cursor = Type::Cursor(element.clone());
                let [address, count] = self.product_fields(&argument, [&cursor, &Type::USize])?;
                let region_type = self.types.value(result_type)?;
                let with_address = self.register();
                self.line(format!(
                    "  {with_address} = insertvalue {} poison, ptr {}, 0",
                    region_type.llvm, address.representation
                ));
                let region = self.register();
                self.line(format!(
                    "  {region} = insertvalue {} {with_address}, {} {}, 1",
                    region_type.llvm,
                    self.types.pointer_integer()?,
                    count.representation
                ));
                Some(EmittedValue {
                    ty: result_type.clone(),
                    representation: region,
                    owned: false,
                })
            }
            MemoryPrimitive::ProjectAddress => {
                let address = match &argument.ty {
                    Type::Cursor(_) => argument.representation,
                    Type::Region(_) => {
                        let region_type = self.types.value(&argument.ty)?;
                        let address = self.register();
                        self.line(format!(
                            "  {address} = extractvalue {} {}, 0",
                            region_type.llvm, argument.representation
                        ));
                        address
                    }
                    _ => return None,
                };
                (*result_type == Type::Address).then_some(EmittedValue {
                    ty: Type::Address,
                    representation: address,
                    owned: false,
                })
            }
            MemoryPrimitive::LoadValue => self.emit_cursor_load(&argument, result_type),
            MemoryPrimitive::StoreValue => self.emit_cursor_store(&argument, result_type),
            MemoryPrimitive::Align => None,
        }
    }

    fn emit_cursor_load(
        &mut self,
        cursor: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Cursor(element) = &cursor.ty else {
            return None;
        };
        let expected = Type::Product(vec![(**element).clone(), cursor.ty.clone()].into());
        if *result_type != expected {
            return None;
        }
        let value = self.emit_source_load(cursor, element)?;
        let next = self.offset_cursor(cursor, element)?;
        let product_type = self.types.value(result_type)?;
        let value_type = self.types.value(element)?;
        let with_value = self.register();
        self.line(format!(
            "  {with_value} = insertvalue {} poison, {} {}, 0",
            product_type.llvm, value_type.llvm, value.representation
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_value}, ptr {}, 1",
            product_type.llvm, next.representation
        ));
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: result,
            owned: false,
        })
    }

    fn emit_cursor_store(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(fields) = &argument.ty else {
            return None;
        };
        let [cursor_type, element] = fields.as_ref() else {
            return None;
        };
        let Type::Cursor(cursor_element) = cursor_type else {
            return None;
        };
        if cursor_element.as_ref() != element || *result_type != *cursor_type {
            return None;
        }
        let [cursor, value] = self.product_fields(argument, [cursor_type, element])?;
        self.emit_source_store(&cursor, &value)?;
        self.offset_cursor(&cursor, element)
    }

    fn offset_cursor(&mut self, cursor: &EmittedValue, element: &Type) -> Option<EmittedValue> {
        let stride = self.types.source_layout(element)?.stride;
        if stride == 0 {
            return Some(cursor.clone());
        }
        let next = self.register();
        self.line(format!(
            "  {next} = getelementptr i8, ptr {}, {} {stride}",
            cursor.representation,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: cursor.ty.clone(),
            representation: next,
            owned: false,
        })
    }

    fn emit_source_load(&mut self, cursor: &EmittedValue, element: &Type) -> Option<EmittedValue> {
        if *element == Type::Unit {
            return Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
            });
        }
        if !matches!(
            element,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Address
                | Type::ByteSize
                | Type::USize
        ) {
            return None;
        }
        let value_type = self.types.value(element)?;
        let value = self.register();
        self.line(format!(
            "  {value} = load {}, ptr {}, align 1",
            value_type.llvm, cursor.representation
        ));
        Some(EmittedValue {
            ty: element.clone(),
            representation: value,
            owned: false,
        })
    }

    fn emit_source_store(&mut self, cursor: &EmittedValue, value: &EmittedValue) -> Option<()> {
        if value.ty == Type::Unit {
            return Some(());
        }
        if !matches!(
            value.ty,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Address
                | Type::ByteSize
                | Type::USize
        ) {
            return None;
        }
        let value_type = self.types.value(&value.ty)?;
        self.line(format!(
            "  store {} {}, ptr {}, align 1",
            value_type.llvm, value.representation, cursor.representation
        ));
        Some(())
    }

    fn emit_load(&mut self, pointer: &EmittedValue, ty: Type) -> Option<EmittedValue> {
        if pointer.ty != Type::Ptr {
            return None;
        }
        let value_type = self.types.value(&ty)?;
        let result = self.register();
        self.line(format!(
            "  {result} = load {}, ptr {}, align 1",
            value_type.llvm, pointer.representation
        ));
        Some(EmittedValue {
            ty,
            representation: result,
            owned: false,
        })
    }

    fn emit_store(&mut self, argument: &EmittedValue, value_type: &Type) -> Option<EmittedValue> {
        let [pointer, value] = self.product_fields(argument, [&Type::Ptr, value_type])?;
        let representation = self.types.value(value_type)?;
        self.line(format!(
            "  store {} {}, ptr {}, align 1",
            representation.llvm, value.representation, pointer.representation
        ));
        Some(EmittedValue {
            ty: Type::Unit,
            representation: "0".into(),
            owned: false,
        })
    }

    pub(super) fn product_fields<const N: usize>(
        &mut self,
        product: &EmittedValue,
        expected: [&Type; N],
    ) -> Option<[EmittedValue; N]> {
        let Type::Product(elements) = &product.ty else {
            return None;
        };
        if elements.iter().collect::<Vec<_>>() != expected {
            return None;
        }
        let product_type = self.types.value(&product.ty)?;
        let mut fields = Vec::with_capacity(N);
        for (index, ty) in elements.iter().enumerate() {
            let field = self.register();
            self.line(format!(
                "  {field} = extractvalue {} {}, {index}",
                product_type.llvm, product.representation
            ));
            fields.push(EmittedValue {
                ty: ty.clone(),
                representation: field,
                owned: false,
            });
        }
        fields.try_into().ok()
    }
}
