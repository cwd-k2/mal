use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

pub(in crate::backend::llvm::body) struct ByteViewFields {
    pub(in crate::backend::llvm::body) owner: String,
    pub(in crate::backend::llvm::body) data: String,
    pub(in crate::backend::llvm::body) count: String,
}

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_buffer_length(
        &mut self,
        buffer: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if !matches!(buffer.ty, Type::Buffer(_)) || *result_type != Type::USize {
            return None;
        }
        let count = self.register();
        self.line(format!(
            "  {count} = call {} @mal_runtime_buffer_count(ptr {})",
            self.types.pointer_integer()?,
            buffer.representation
        ));
        Some(EmittedValue {
            ty: Type::USize,
            representation: count,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_buffer_to_symbol(
        &mut self,
        buffer: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::Symbol || buffer.ty != Type::Buffer(Type::UInt8.into()) {
            return None;
        }
        let data = self.active_buffer_data(buffer)?;
        let count = self.register();
        self.line(format!(
            "  {count} = call {} @mal_runtime_buffer_count(ptr {})",
            self.types.pointer_integer()?,
            buffer.representation
        ));
        let owner = self.register();
        self.line(format!(
            "  {owner} = call ptr @mal_runtime_bytes_read(ptr %mal_context, ptr {data}, {} {count})",
            self.types.pointer_integer()?
        ));
        let copied_data = self.register();
        self.line(format!(
            "  {copied_data} = call ptr @mal_runtime_bytes_data(ptr {owner})"
        ));
        self.make_byte_view(&Type::Symbol, &owner, &copied_data, &count, true)
    }

    pub(in crate::backend::llvm::body) fn emit_symbol_to_buffer(
        &mut self,
        symbol: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if symbol.ty != Type::Symbol || *result_type != Type::Buffer(Type::UInt8.into()) {
            return None;
        }
        let fields = self.byte_view_fields(symbol)?;
        let buffer = self.register();
        let index = self.types.pointer_integer()?;
        self.line(format!(
            "  {buffer} = call ptr @mal_runtime_buffer_from(ptr %mal_context, ptr {data}, {index} 0, {index} {count}, {index} 1)",
            data = fields.data,
            count = fields.count,
        ));
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: buffer,
            owned: true,
        })
    }

    pub(in crate::backend::llvm::body) fn byte_view_fields(
        &mut self,
        value: &EmittedValue,
    ) -> Option<ByteViewFields> {
        if value.ty != Type::Symbol {
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

    pub(in crate::backend::llvm::body) fn make_byte_view(
        &mut self,
        ty: &Type,
        owner: &str,
        data: &str,
        count: &str,
        owned: bool,
    ) -> Option<EmittedValue> {
        if *ty != Type::Symbol {
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
}
