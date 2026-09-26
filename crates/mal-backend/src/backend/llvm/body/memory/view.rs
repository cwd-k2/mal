use mal_frontend::check::ast::Type;

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
        self.direct_call(
            Some(count.clone()),
            false,
            self.types.index_llvm_type(),
            "mal_runtime_buffer_count",
            [(
                crate::backend::llvm::syntax::Type::Pointer,
                buffer.representation.clone(),
            )],
        );
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
        let data = self.active_buffer_data(buffer);
        let count = self.register();
        self.direct_call(
            Some(count.clone()),
            false,
            self.types.index_llvm_type(),
            "mal_runtime_buffer_count",
            [(
                crate::backend::llvm::syntax::Type::Pointer,
                buffer.representation.clone(),
            )],
        );
        let owner = self.register();
        self.direct_call(
            Some(owner.clone()),
            false,
            crate::backend::llvm::syntax::Type::Pointer,
            "mal_runtime_bytes_read",
            [
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    "%mal_context".into(),
                ),
                (crate::backend::llvm::syntax::Type::Pointer, data),
                (self.types.index_llvm_type(), count.clone()),
            ],
        );
        let copied_data = self.register();
        self.direct_call(
            Some(copied_data.clone()),
            false,
            crate::backend::llvm::syntax::Type::Pointer,
            "mal_runtime_bytes_data",
            [(crate::backend::llvm::syntax::Type::Pointer, owner.clone())],
        );
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
        self.direct_call(
            Some(buffer.clone()),
            false,
            crate::backend::llvm::syntax::Type::Pointer,
            "mal_runtime_buffer_from",
            [
                (
                    crate::backend::llvm::syntax::Type::Pointer,
                    "%mal_context".into(),
                ),
                (crate::backend::llvm::syntax::Type::Pointer, fields.data),
                (self.types.index_llvm_type(), "0".into()),
                (self.types.index_llvm_type(), fields.count),
                (self.types.index_llvm_type(), "1".into()),
            ],
        );
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
        let [owner, data, count] = std::array::from_fn(|index| {
            let field = self.register();
            self.line(format!(
                "  {field} = extractvalue {} {}, {index}",
                runtime.llvm, value.representation
            ));
            field
        });
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
            self.types.index_integer()
        ));
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }
}
