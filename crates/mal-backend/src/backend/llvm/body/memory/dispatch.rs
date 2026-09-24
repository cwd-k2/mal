use mal_frontend::check::ast::{MemoryPrimitive, Type};

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_memory(
        &mut self,
        primitive: MemoryPrimitive,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        match primitive {
            MemoryPrimitive::BufferFromAddress => {
                self.emit_buffer_from_address(argument, result_type)
            }
            MemoryPrimitive::BufferIntoAddress => {
                self.emit_buffer_into_address(argument, result_type)
            }
            MemoryPrimitive::ViewLength => self.emit_buffer_length(argument, result_type),
            MemoryPrimitive::BufferToSymbol => self.emit_buffer_to_symbol(argument, result_type),
            MemoryPrimitive::SymbolToBuffer => self.emit_symbol_to_buffer(argument, result_type),
            MemoryPrimitive::BufferMake
            | MemoryPrimitive::BufferNew
            | MemoryPrimitive::BufferGet
            | MemoryPrimitive::BufferPut
            | MemoryPrimitive::BufferFill
            | MemoryPrimitive::BufferCopy => None,
        }
    }
}
