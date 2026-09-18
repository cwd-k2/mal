use crate::check::ast::{MemoryPrimitive, Type};

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_memory(
        &mut self,
        primitive: MemoryPrimitive,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
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
                    representation: argument.representation.clone(),
                    owned: false,
                })
            }
            MemoryPrimitive::Region => {
                let Type::Region(element) = result_type else {
                    return None;
                };
                let cursor = Type::Cursor(element.clone());
                let [address, count] = self.product_fields(argument, [&cursor, &Type::USize])?;
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
                    Type::Cursor(_) => argument.representation.clone(),
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
            MemoryPrimitive::Align => self.emit_align(argument.clone(), result_type),
            MemoryPrimitive::LoadValue => self.emit_cursor_load(argument, result_type),
            MemoryPrimitive::StoreValue => self.emit_cursor_store(argument, result_type),
            MemoryPrimitive::AdmitRegion => self.emit_region_admission(argument, result_type),
            MemoryPrimitive::StorePacked => self.emit_packed_store(argument, result_type),
            MemoryPrimitive::Prefix | MemoryPrimitive::RemainderView => {
                self.emit_view_slice(primitive, argument, result_type)
            }
            MemoryPrimitive::ViewLength => self.emit_view_length(argument, result_type),
            MemoryPrimitive::PackedIndex => self.emit_packed_index(argument, result_type),
            MemoryPrimitive::PackedToSymbol => self.emit_packed_to_symbol(argument, result_type),
            MemoryPrimitive::SymbolToPacked => self.emit_symbol_to_packed(argument, result_type),
        }
    }
}
