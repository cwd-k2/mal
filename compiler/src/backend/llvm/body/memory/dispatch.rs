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
            MemoryPrimitive::FormRegion => self.emit_form_region(argument, result_type),
            MemoryPrimitive::RegionGet => self.emit_region_get(argument, result_type),
            MemoryPrimitive::RegionPut => self.emit_region_put(argument, result_type),
            MemoryPrimitive::PackAddress => self.emit_address_pack(argument, result_type),
            MemoryPrimitive::RegionSet => self.emit_packed_store(argument, result_type),
            MemoryPrimitive::PackedConcat => self.emit_packed_concat(argument, result_type),
            MemoryPrimitive::Prefix | MemoryPrimitive::RemainderView => {
                self.emit_view_slice(primitive, argument, result_type)
            }
            MemoryPrimitive::ViewLength => self.emit_view_length(argument, result_type),
            MemoryPrimitive::PackedIndex => self.emit_packed_index(argument, result_type),
            MemoryPrimitive::PackedToSymbol => self.emit_packed_to_symbol(argument, result_type),
            MemoryPrimitive::SymbolToPacked => self.emit_symbol_to_packed(argument, result_type),
            MemoryPrimitive::BufferNew
            | MemoryPrimitive::BufferGet
            | MemoryPrimitive::BufferPut => None,
        }
    }
}
