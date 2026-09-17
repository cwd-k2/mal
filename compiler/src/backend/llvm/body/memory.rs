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
        let argument = self.atom(argument)?;
        match primitive {
            MemoryPrimitive::Place => {
                let Type::Cursor(element) = result_type else {
                    return None;
                };
                if argument.ty != Type::Address || self.source_layouts.layout(element).is_none() {
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
            MemoryPrimitive::Align => self.emit_align(argument, result_type),
            MemoryPrimitive::LoadValue => self.emit_cursor_load(&argument, result_type),
            MemoryPrimitive::StoreValue => self.emit_cursor_store(&argument, result_type),
            MemoryPrimitive::AdmitRegion => self.emit_region_admission(&argument, result_type),
            MemoryPrimitive::StorePacked => self.emit_packed_store(&argument, result_type),
            MemoryPrimitive::Prefix | MemoryPrimitive::RemainderView => {
                self.emit_view_slice(primitive, &argument, result_type)
            }
            MemoryPrimitive::ViewLength => self.emit_view_length(&argument, result_type),
            MemoryPrimitive::PackedIndex => self.emit_packed_index(&argument, result_type),
            MemoryPrimitive::PackedToSymbol => self.emit_packed_to_symbol(&argument, result_type),
            MemoryPrimitive::SymbolToPacked => self.emit_symbol_to_packed(&argument, result_type),
        }
    }

    fn emit_region_admission(
        &mut self,
        region: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Region(element) = &region.ty else {
            return None;
        };
        if *result_type != Type::Packed(element.clone()) {
            return None;
        }
        let (address, count) = self.region_fields(region)?;
        let stride = self.source_layouts.layout(element)?.stride;
        let owner = if stride == 0 {
            "null".to_string()
        } else {
            let bytes = self.multiply_by_stride(&count, stride)?;
            let owner = self.register();
            self.line(format!(
                "  {owner} = call ptr @mal_runtime_bytes_read(ptr %mal_context, ptr {address}, {} {bytes})",
                self.types.pointer_integer()?
            ));
            owner
        };
        self.make_packed(result_type, &owner, "0", &count, true)
    }

    fn emit_packed_store(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Region(element) = result_type else {
            return None;
        };
        let packed_type = Type::Packed(element.clone());
        let [region, packed] = self.product_fields(argument, [result_type, &packed_type])?;
        let (address, region_count) = self.region_fields(&region)?;
        let (owner, offset, count) = self.packed_fields(&packed)?;
        let stride = self.source_layouts.layout(element)?.stride;
        let bytes = self.multiply_by_stride(&count, stride)?;
        if stride != 0 {
            let data = self.packed_data_pointer(&owner, &offset)?;
            self.line(format!(
                "  call void @llvm.memcpy.p0.p0.i{}(ptr {address}, ptr {data}, {} {bytes}, i1 false)",
                self.types.index_size() * 8,
                self.types.pointer_integer()?
            ));
        }
        let suffix_address = self.pointer_offset(&address, &bytes)?;
        let suffix_count = self.register();
        self.line(format!(
            "  {suffix_count} = sub {} {region_count}, {count}",
            self.types.pointer_integer()?
        ));
        self.make_region(result_type, &suffix_address, &suffix_count)
    }

    fn emit_view_slice(
        &mut self,
        primitive: MemoryPrimitive,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(fields) = &argument.ty else {
            return None;
        };
        let [view_type, count_type] = fields.as_ref() else {
            return None;
        };
        if count_type != &Type::USize || view_type != result_type {
            return None;
        }
        let [view, count] = self.product_fields(argument, [view_type, &Type::USize])?;
        let prefix = primitive == MemoryPrimitive::Prefix;
        match view_type {
            Type::Region(element) => {
                let (address, old_count) = self.region_fields(&view)?;
                if prefix {
                    self.make_region(result_type, &address, &count.representation)
                } else {
                    let stride = self.source_layouts.layout(element)?.stride;
                    let bytes = self.multiply_by_stride(&count.representation, stride)?;
                    let address = self.pointer_offset(&address, &bytes)?;
                    let remainder = self.register();
                    self.line(format!(
                        "  {remainder} = sub {} {old_count}, {}",
                        self.types.pointer_integer()?,
                        count.representation
                    ));
                    self.make_region(result_type, &address, &remainder)
                }
            }
            Type::Packed(element) => {
                let (owner, offset, old_count) = self.packed_fields(&view)?;
                let retained = self.register();
                self.line(format!(
                    "  {retained} = call ptr @mal_runtime_bytes_retain(ptr %mal_context, ptr {owner})"
                ));
                if prefix {
                    self.make_packed(result_type, &retained, &offset, &count.representation, true)
                } else {
                    let stride = self.source_layouts.layout(element)?.stride;
                    let bytes = self.multiply_by_stride(&count.representation, stride)?;
                    let new_offset = self.register();
                    self.line(format!(
                        "  {new_offset} = add {} {offset}, {bytes}",
                        self.types.pointer_integer()?
                    ));
                    let remainder = self.register();
                    self.line(format!(
                        "  {remainder} = sub {} {old_count}, {}",
                        self.types.pointer_integer()?,
                        count.representation
                    ));
                    self.make_packed(result_type, &retained, &new_offset, &remainder, true)
                }
            }
            _ => None,
        }
    }

    fn emit_view_length(
        &mut self,
        view: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::USize {
            return None;
        }
        let count = match &view.ty {
            Type::Region(_) => self.region_fields(view)?.1,
            Type::Packed(_) => self.packed_fields(view)?.2,
            _ => return None,
        };
        Some(EmittedValue {
            ty: Type::USize,
            representation: count,
            owned: false,
        })
    }

    fn emit_packed_index(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let packed_type = Type::Packed(result_type.clone().into());
        let [packed, index] = self.product_fields(argument, [&packed_type, &Type::USize])?;
        let (owner, offset, _) = self.packed_fields(&packed)?;
        if *result_type == Type::Unit {
            return Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
            });
        }
        let stride = self.source_layouts.layout(result_type)?.stride;
        let element_offset = self.multiply_by_stride(&index.representation, stride)?;
        let absolute = self.register();
        self.line(format!(
            "  {absolute} = add {} {offset}, {element_offset}",
            self.types.pointer_integer()?
        ));
        let pointer = self.packed_data_pointer(&owner, &absolute)?;
        self.emit_source_load_at(&pointer, result_type)
    }

    fn emit_packed_to_symbol(
        &mut self,
        packed: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::Symbol || packed.ty != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let (owner, offset, count) = self.packed_fields(packed)?;
        let retained = self.register();
        self.line(format!(
            "  {retained} = call ptr @mal_runtime_bytes_retain(ptr %mal_context, ptr {owner})"
        ));
        self.make_byte_view(&Type::Symbol, &retained, &offset, &count, true)
    }

    fn emit_symbol_to_packed(
        &mut self,
        symbol: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if symbol.ty != Type::Symbol || *result_type != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let (owner, offset, count) = self.byte_view_fields(symbol)?;
        let retained = self.register();
        self.line(format!(
            "  {retained} = call ptr @mal_runtime_bytes_retain(ptr %mal_context, ptr {owner})"
        ));
        self.make_packed(result_type, &retained, &offset, &count, true)
    }

    fn region_fields(&mut self, region: &EmittedValue) -> Option<(String, String)> {
        let Type::Region(_) = &region.ty else {
            return None;
        };
        let runtime = self.types.value(&region.ty)?;
        let address = self.register();
        self.line(format!(
            "  {address} = extractvalue {} {}, 0",
            runtime.llvm, region.representation
        ));
        let count = self.register();
        self.line(format!(
            "  {count} = extractvalue {} {}, 1",
            runtime.llvm, region.representation
        ));
        Some((address, count))
    }

    fn packed_fields(&mut self, packed: &EmittedValue) -> Option<(String, String, String)> {
        let Type::Packed(_) = &packed.ty else {
            return None;
        };
        self.byte_view_fields(packed)
    }

    pub(super) fn byte_view_fields(
        &mut self,
        value: &EmittedValue,
    ) -> Option<(String, String, String)> {
        if !matches!(value.ty, Type::Symbol | Type::Packed(_)) {
            return None;
        }
        let runtime = self.types.value(&value.ty)?;
        let mut fields = Vec::with_capacity(3);
        for index in 0..3 {
            let field = self.register();
            self.line(format!(
                "  {field} = extractvalue {} {}, {index}",
                runtime.llvm, value.representation
            ));
            fields.push(field);
        }
        let [owner, offset, count]: [String; 3] = fields.try_into().ok()?;
        Some((owner, offset, count))
    }

    fn make_region(&mut self, ty: &Type, address: &str, count: &str) -> Option<EmittedValue> {
        let runtime = self.types.value(ty)?;
        let with_address = self.register();
        self.line(format!(
            "  {with_address} = insertvalue {} poison, ptr {address}, 0",
            runtime.llvm
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_address}, {} {count}, 1",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned: false,
        })
    }

    fn make_packed(
        &mut self,
        ty: &Type,
        owner: &str,
        offset: &str,
        count: &str,
        owned: bool,
    ) -> Option<EmittedValue> {
        self.make_byte_view(ty, owner, offset, count, owned)
    }

    pub(super) fn make_byte_view(
        &mut self,
        ty: &Type,
        owner: &str,
        offset: &str,
        count: &str,
        owned: bool,
    ) -> Option<EmittedValue> {
        if !matches!(ty, Type::Symbol | Type::Packed(_)) {
            return None;
        }
        let runtime = self.types.value(ty)?;
        let with_owner = self.register();
        self.line(format!(
            "  {with_owner} = insertvalue {} poison, ptr {owner}, 0",
            runtime.llvm
        ));
        let with_offset = self.register();
        self.line(format!(
            "  {with_offset} = insertvalue {} {with_owner}, {} {offset}, 1",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_offset}, {} {count}, 2",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }

    fn multiply_by_stride(&mut self, value: &str, stride: usize) -> Option<String> {
        if stride == 0 {
            return Some("0".into());
        }
        if stride == 1 {
            return Some(value.into());
        }
        let result = self.register();
        self.line(format!(
            "  {result} = mul {} {value}, {stride}",
            self.types.pointer_integer()?
        ));
        Some(result)
    }

    fn pointer_offset(&mut self, pointer: &str, offset: &str) -> Option<String> {
        let result = self.register();
        self.line(format!(
            "  {result} = getelementptr i8, ptr {pointer}, {} {offset}",
            self.types.pointer_integer()?
        ));
        Some(result)
    }

    fn packed_data_pointer(&mut self, owner: &str, offset: &str) -> Option<String> {
        let data = self.register();
        self.line(format!(
            "  {data} = call ptr @mal_runtime_bytes_data(ptr {owner})"
        ));
        self.pointer_offset(&data, offset)
    }

    fn emit_align(&mut self, value: EmittedValue, result_type: &Type) -> Option<EmittedValue> {
        if value.ty != *result_type {
            return None;
        }
        let element = match &value.ty {
            Type::Cursor(element) | Type::Region(element) => element,
            _ => return None,
        };
        let alignment = self.source_layouts.layout(element)?.alignment;
        if alignment == 1 {
            return Some(value);
        }
        let (address, count) = match &value.ty {
            Type::Cursor(_) => (value.representation.clone(), None),
            Type::Region(_) => {
                let runtime = self.types.value(&value.ty)?;
                let address = self.register();
                self.line(format!(
                    "  {address} = extractvalue {} {}, 0",
                    runtime.llvm, value.representation
                ));
                let count = self.register();
                self.line(format!(
                    "  {count} = extractvalue {} {}, 1",
                    runtime.llvm, value.representation
                ));
                (address, Some(count))
            }
            _ => return None,
        };
        let advanced = self.register();
        self.line(format!(
            "  {advanced} = getelementptr i8, ptr {address}, {} {}",
            self.types.pointer_integer()?,
            alignment - 1
        ));
        let aligned = self.register();
        let bits = self.types.pointer_size().checked_mul(8)?;
        self.line(format!(
            "  {aligned} = call ptr @llvm.ptrmask.p0.i{bits}(ptr {advanced}, i{bits} -{alignment})"
        ));
        let Some(count) = count else {
            return Some(EmittedValue {
                ty: value.ty,
                representation: aligned,
                owned: false,
            });
        };
        let runtime = self.types.value(&value.ty)?;
        let with_address = self.register();
        self.line(format!(
            "  {with_address} = insertvalue {} poison, ptr {aligned}, 0",
            runtime.llvm
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_address}, {} {count}, 1",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: value.ty,
            representation: result,
            owned: false,
        })
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
        let stride = self.source_layouts.layout(element)?.stride;
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
        self.emit_source_load_at(&cursor.representation, element)
    }

    fn emit_source_load_at(&mut self, pointer: &str, element: &Type) -> Option<EmittedValue> {
        if *element == Type::Unit {
            return Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
            });
        }
        if let Type::Product(elements) = element {
            let fields = self.source_layouts.product_fields(element)?;
            let product_type = self.types.value(element)?;
            let mut product = "poison".to_string();
            for (index, (field, field_type)) in fields.iter().zip(elements.iter()).enumerate() {
                let field_pointer = self.source_pointer_offset(pointer, field.offset)?;
                let field_value = self.emit_source_load_at(&field_pointer, field_type)?;
                let llvm_type = self.types.value(field_type)?;
                let inserted = self.register();
                self.line(format!(
                    "  {inserted} = insertvalue {} {product}, {} {}, {index}",
                    product_type.llvm, llvm_type.llvm, field_value.representation
                ));
                product = inserted;
            }
            return Some(EmittedValue {
                ty: element.clone(),
                representation: product,
                owned: false,
            });
        }
        if let Type::Sum(variants) = element {
            let layout = self.source_layouts.sum(element)?;
            let source_tag = self.register();
            self.line(format!(
                "  {source_tag} = load i{}, ptr {pointer}, align 1",
                layout.tag_bits
            ));
            if super::types::is_bool(element) {
                let value = self.register();
                self.line(format!("  {value} = trunc i8 {source_tag} to i1"));
                return Some(EmittedValue {
                    ty: element.clone(),
                    representation: value,
                    owned: false,
                });
            }
            let tag = if layout.tag_bits == 32 {
                source_tag
            } else if layout.tag_bits < 32 {
                let extended = self.register();
                self.line(format!(
                    "  {extended} = zext i{} {source_tag} to i32",
                    layout.tag_bits
                ));
                extended
            } else {
                let narrowed = self.register();
                self.line(format!("  {narrowed} = trunc i64 {source_tag} to i32"));
                narrowed
            };
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let runtime = self.types.value(element)?;
            let storage = self.register();
            self.line(format!(
                "  {storage} = alloca {}, align {}",
                runtime.llvm, runtime.alignment
            ));
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset)?;
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| format!("    i32 {index}, label %{stem}_variant_{index}"))
                .collect::<Vec<_>>()
                .join("\n");
            self.line(format!(
                "  switch i32 {tag}, label %{stem}_invalid [\n{cases}\n  ]"
            ));
            self.line(format!("{stem}_invalid:"));
            self.line("  unreachable");
            for (index, variant) in variants.iter().enumerate() {
                self.line(format!("{stem}_variant_{index}:"));
                let payload = self.emit_source_load_at(&payload_pointer, variant)?;
                let sum = self.emit_sum_value(index, payload, element, false)?;
                self.line(format!(
                    "  store {} {}, ptr {storage}, align {}",
                    runtime.llvm, sum.representation, runtime.alignment
                ));
                self.line(format!("  br label %{stem}_loaded"));
            }
            self.line(format!("{stem}_loaded:"));
            let result = self.register();
            self.line(format!(
                "  {result} = load {}, ptr {storage}, align {}",
                runtime.llvm, runtime.alignment
            ));
            return Some(EmittedValue {
                ty: element.clone(),
                representation: result,
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
            "  {value} = load {}, ptr {pointer}, align 1",
            value_type.llvm
        ));
        Some(EmittedValue {
            ty: element.clone(),
            representation: value,
            owned: false,
        })
    }

    fn emit_source_store(&mut self, cursor: &EmittedValue, value: &EmittedValue) -> Option<()> {
        self.emit_source_store_at(&cursor.representation, value)
    }

    fn emit_source_store_at(&mut self, pointer: &str, value: &EmittedValue) -> Option<()> {
        if value.ty == Type::Unit {
            return Some(());
        }
        if let Type::Product(elements) = &value.ty {
            let fields = self.source_layouts.product_fields(&value.ty)?;
            let runtime = self.types.value(&value.ty)?;
            for (index, (field, field_type)) in fields.iter().zip(elements.iter()).enumerate() {
                let field_value = self.register();
                self.line(format!(
                    "  {field_value} = extractvalue {} {}, {index}",
                    runtime.llvm, value.representation
                ));
                let field_pointer = self.source_pointer_offset(pointer, field.offset)?;
                self.emit_source_store_at(
                    &field_pointer,
                    &EmittedValue {
                        ty: field_type.clone(),
                        representation: field_value,
                        owned: false,
                    },
                )?;
            }
            return Some(());
        }
        if let Type::Sum(variants) = &value.ty {
            let layout = self.source_layouts.sum(&value.ty)?;
            if super::types::is_bool(&value.ty) {
                let tag = self.register();
                self.line(format!("  {tag} = zext i1 {} to i8", value.representation));
                self.line(format!("  store i8 {tag}, ptr {pointer}, align 1"));
                return Some(());
            }
            let runtime = self.types.value(&value.ty)?;
            let tag = self.register();
            self.line(format!(
                "  {tag} = extractvalue {} {}, 0",
                runtime.llvm, value.representation
            ));
            let source_tag = if layout.tag_bits == 32 {
                tag.clone()
            } else if layout.tag_bits < 32 {
                let narrowed = self.register();
                self.line(format!(
                    "  {narrowed} = trunc i32 {tag} to i{}",
                    layout.tag_bits
                ));
                narrowed
            } else {
                let extended = self.register();
                self.line(format!("  {extended} = zext i32 {tag} to i64"));
                extended
            };
            self.line(format!(
                "  store i{} {source_tag}, ptr {pointer}, align 1",
                layout.tag_bits
            ));
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset)?;
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| format!("    i32 {index}, label %{stem}_variant_{index}"))
                .collect::<Vec<_>>()
                .join("\n");
            self.line(format!(
                "  switch i32 {tag}, label %{stem}_invalid [\n{cases}\n  ]"
            ));
            self.line(format!("{stem}_invalid:"));
            self.line("  unreachable");
            for (index, variant) in variants.iter().enumerate() {
                self.line(format!("{stem}_variant_{index}:"));
                let payload = self.emit_sum_payload(&value.ty, variant, &value.representation)?;
                self.emit_source_store_at(
                    &payload_pointer,
                    &EmittedValue {
                        ty: variant.clone(),
                        representation: payload,
                        owned: false,
                    },
                )?;
                self.line(format!("  br label %{stem}_stored"));
            }
            self.line(format!("{stem}_stored:"));
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
            "  store {} {}, ptr {pointer}, align 1",
            value_type.llvm, value.representation
        ));
        Some(())
    }

    fn source_pointer_offset(&mut self, pointer: &str, offset: usize) -> Option<String> {
        if offset == 0 {
            return Some(pointer.to_string());
        }
        let field = self.register();
        self.line(format!(
            "  {field} = getelementptr i8, ptr {pointer}, {} {offset}",
            self.types.pointer_integer()?
        ));
        Some(field)
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
