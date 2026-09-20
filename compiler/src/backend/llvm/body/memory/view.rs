use crate::check::ast::{MemoryPrimitive, Type};

use super::super::{EmittedValue, FunctionEmitter};

pub(in crate::backend::llvm::body) struct ByteViewFields {
    pub(in crate::backend::llvm::body) owner: String,
    pub(in crate::backend::llvm::body) data: String,
    pub(in crate::backend::llvm::body) count: String,
}

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_address_pack(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Packed(element) = result_type else {
            return None;
        };
        let [address, start, end] =
            self.product_fields(argument, [&Type::Address, &Type::USize, &Type::USize])?;
        let stride = self.source_layouts.layout(element)?.stride;
        let count = self.register();
        self.line(format!(
            "  {count} = sub {} {}, {}",
            self.types.pointer_integer()?,
            end.representation,
            start.representation
        ));
        let owner = if stride == 0 {
            "null".to_string()
        } else {
            let start_bytes = self.multiply_by_stride(&start.representation, stride)?;
            let first = self.pointer_offset(&address.representation, &start_bytes)?;
            let bytes = self.multiply_by_stride(&count, stride)?;
            let owner = self.register();
            self.line(format!(
                "  {owner} = call ptr @mal_runtime_bytes_read(ptr %mal_context, ptr {first}, {} {bytes})",
                self.types.pointer_integer()?
            ));
            owner
        };
        let data = if stride == 0 {
            "null".to_string()
        } else {
            let data = self.register();
            self.line(format!(
                "  {data} = call ptr @mal_runtime_bytes_data(ptr {owner})"
            ));
            data
        };
        self.make_packed(result_type, &owner, &data, &count, true)
    }

    pub(in crate::backend::llvm::body) fn emit_packed_store(
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
        let ByteViewFields { data, count, .. } = self.packed_fields(&packed)?;
        let stride = self.source_layouts.layout(element)?.stride;
        let bytes = self.multiply_by_stride(&count, stride)?;
        if stride != 0 {
            self.line(format!(
                "  call void @llvm.memcpy.p0.p0.i{}(ptr {address}, ptr {data}, {} {bytes}, i1 false)",
                self.types.index_size() * 8,
                self.types.pointer_integer()?
            ));
        }
        let suffix_address = if stride == 0 {
            address
        } else {
            self.pointer_offset(&address, &bytes)?
        };
        let suffix_count = self.register();
        self.line(format!(
            "  {suffix_count} = sub {} {region_count}, {count}",
            self.types.pointer_integer()?
        ));
        self.make_region(result_type, &suffix_address, &suffix_count)
    }

    pub(in crate::backend::llvm::body) fn emit_packed_concat(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Packed(element) = result_type else {
            return None;
        };
        let [left, right] = self.product_fields(argument, [result_type, result_type])?;
        let left = self.packed_fields(&left)?;
        let right = self.packed_fields(&right)?;
        let count = self.register();
        self.line(format!(
            "  {count} = add {} {}, {}",
            self.types.pointer_integer()?,
            left.count,
            right.count
        ));
        let stride = self.source_layouts.layout(element)?.stride;
        if stride == 0 {
            return self.make_packed(result_type, "null", "null", &count, false);
        }
        let left_bytes = self.multiply_by_stride(&left.count, stride)?;
        let right_bytes = self.multiply_by_stride(&right.count, stride)?;
        let runtime = self.types.value(result_type)?;
        let storage = self.entry_alloca(&runtime.llvm, runtime.alignment);
        self.line(format!(
            "  call void @mal_runtime_symbol_concatenate(ptr %mal_context, ptr {storage}, ptr {}, ptr {}, {integer} {left_bytes}, ptr {}, ptr {}, {integer} {right_bytes})",
            left.owner, left.data, right.owner, right.data,
            integer = self.types.pointer_integer()?
        ));
        let bytes_view = self.register();
        self.line(format!(
            "  {bytes_view} = load {}, ptr {storage}, align {}",
            runtime.llvm, runtime.alignment
        ));
        let packed = self.register();
        self.line(format!(
            "  {packed} = insertvalue {} {bytes_view}, {} {count}, 2",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: packed,
            owned: true,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_view_slice(
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
                    let address = if stride == 0 {
                        address
                    } else {
                        self.pointer_offset(&address, &bytes)?
                    };
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
                let ByteViewFields {
                    owner,
                    data,
                    count: old_count,
                } = self.packed_fields(&view)?;
                if prefix {
                    self.make_packed(result_type, &owner, &data, &count.representation, true)
                } else {
                    let stride = self.source_layouts.layout(element)?.stride;
                    let bytes = self.multiply_by_stride(&count.representation, stride)?;
                    let new_data = if stride == 0 {
                        data
                    } else {
                        self.pointer_offset(&data, &bytes)?
                    };
                    let remainder = self.register();
                    self.line(format!(
                        "  {remainder} = sub {} {old_count}, {}",
                        self.types.pointer_integer()?,
                        count.representation
                    ));
                    self.make_packed(result_type, &owner, &new_data, &remainder, true)
                }
            }
            _ => None,
        }
    }

    pub(in crate::backend::llvm::body) fn emit_view_length(
        &mut self,
        view: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::USize {
            return None;
        }
        let count = match &view.ty {
            Type::Region(_) => self.region_fields(view)?.1,
            Type::Packed(_) => self.packed_fields(view)?.count,
            _ => return None,
        };
        Some(EmittedValue {
            ty: Type::USize,
            representation: count,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_packed_index(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let packed_type = Type::Packed(result_type.clone().into());
        let [packed, index] = self.product_fields(argument, [&packed_type, &Type::USize])?;
        let ByteViewFields { data, .. } = self.packed_fields(&packed)?;
        if *result_type == Type::Unit {
            return Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
            });
        }
        let stride = self.source_layouts.layout(result_type)?.stride;
        let element_offset = self.multiply_by_stride(&index.representation, stride)?;
        let pointer = self.pointer_offset(&data, &element_offset)?;
        self.emit_aligned_source_load_at(&pointer, result_type)
    }

    pub(in crate::backend::llvm::body) fn emit_packed_to_symbol(
        &mut self,
        packed: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::Symbol || packed.ty != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let fields = self.packed_fields(packed)?;
        self.make_byte_view(
            &Type::Symbol,
            &fields.owner,
            &fields.data,
            &fields.count,
            true,
        )
    }

    pub(in crate::backend::llvm::body) fn emit_symbol_to_packed(
        &mut self,
        symbol: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if symbol.ty != Type::Symbol || *result_type != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let fields = self.byte_view_fields(symbol)?;
        self.make_packed(
            result_type,
            &fields.owner,
            &fields.data,
            &fields.count,
            true,
        )
    }

    pub(in crate::backend::llvm::body) fn region_fields(
        &mut self,
        region: &EmittedValue,
    ) -> Option<(String, String)> {
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

    fn packed_fields(&mut self, packed: &EmittedValue) -> Option<ByteViewFields> {
        let Type::Packed(_) = &packed.ty else {
            return None;
        };
        self.byte_view_fields(packed)
    }

    pub(in crate::backend::llvm::body) fn byte_view_fields(
        &mut self,
        value: &EmittedValue,
    ) -> Option<ByteViewFields> {
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
        let [owner, data, count]: [String; 3] = fields.try_into().ok()?;
        Some(ByteViewFields { owner, data, count })
    }

    pub(in crate::backend::llvm::body) fn make_region(
        &mut self,
        ty: &Type,
        address: &str,
        count: &str,
    ) -> Option<EmittedValue> {
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
        data: &str,
        count: &str,
        owned: bool,
    ) -> Option<EmittedValue> {
        self.make_byte_view(ty, owner, data, count, owned)
    }

    pub(in crate::backend::llvm::body) fn make_byte_view(
        &mut self,
        ty: &Type,
        owner: &str,
        data: &str,
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
        let with_data = self.register();
        self.line(format!(
            "  {with_data} = insertvalue {} {with_owner}, ptr {data}, 1",
            runtime.llvm
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_data}, {} {count}, 2",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }

    pub(in crate::backend::llvm::body) fn multiply_by_stride(
        &mut self,
        value: &str,
        stride: usize,
    ) -> Option<String> {
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

    pub(in crate::backend::llvm::body) fn pointer_offset(
        &mut self,
        pointer: &str,
        offset: &str,
    ) -> Option<String> {
        let result = self.register();
        self.line(format!(
            "  {result} = getelementptr i8, ptr {pointer}, {} {offset}",
            self.types.pointer_integer()?
        ));
        Some(result)
    }
}
