use crate::check::ast::{MemoryPrimitive, Type};

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_region_admission(
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
                if prefix {
                    self.make_packed(result_type, &owner, &offset, &count.representation, true)
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
                    self.make_packed(result_type, &owner, &new_offset, &remainder, true)
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
            Type::Packed(_) => self.packed_fields(view)?.2,
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

    pub(in crate::backend::llvm::body) fn emit_region_index(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Cursor(element) = result_type else {
            return None;
        };
        let region_type = Type::Region(element.clone());
        let [region, index] = self.product_fields(argument, [&region_type, &Type::USize])?;
        let (address, _) = self.region_fields(&region)?;
        let stride = self.source_layouts.layout(element)?.stride;
        let byte_offset = self.multiply_by_stride(&index.representation, stride)?;
        let pointer = self.pointer_offset(&address, &byte_offset)?;
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: pointer,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_packed_to_symbol(
        &mut self,
        packed: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::Symbol || packed.ty != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let (owner, offset, count) = self.packed_fields(packed)?;
        self.make_byte_view(&Type::Symbol, &owner, &offset, &count, true)
    }

    pub(in crate::backend::llvm::body) fn emit_symbol_to_packed(
        &mut self,
        symbol: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if symbol.ty != Type::Symbol || *result_type != Type::Packed(Type::UInt8.into()) {
            return None;
        }
        let (owner, offset, count) = self.byte_view_fields(symbol)?;
        self.make_packed(result_type, &owner, &offset, &count, true)
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

    pub(in crate::backend::llvm::body) fn byte_view_fields(
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

    pub(in crate::backend::llvm::body) fn make_byte_view(
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
}
