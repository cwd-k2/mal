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
        emit_instruction!(
            self;
            call { Some(count.clone()) },
            false,
            { self.types.index_llvm_type() },
            direct "mal_runtime_buffer_count";
            [
                (typed (ptr) => { buffer.representation.clone() }),
            ]
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
        emit_instruction!(
            self;
            call { Some(count.clone()) },
            false,
            { self.types.index_llvm_type() },
            direct "mal_runtime_buffer_count";
            [
                (typed (ptr) => { buffer.representation.clone() }),
            ]
        );
        let owner = self.register();
        emit_instruction!(
            self;
            call { Some(owner.clone()) },
            false,
            (ptr),
            direct "mal_runtime_bytes_read";
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => { data }),
                (typed { self.types.index_llvm_type() } => { count.clone() }),
            ]
        );
        let copied_data = self.register();
        emit_instruction!(
            self;
            call { Some(copied_data.clone()) },
            false,
            (ptr),
            direct "mal_runtime_bytes_data";
            [
                (typed (ptr) => { owner.clone() }),
            ]
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
        emit_instruction!(
            self;
            call { Some(buffer.clone()) },
            false,
            (ptr),
            direct "mal_runtime_buffer_from";
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => { fields.data }),
                (typed { self.types.index_llvm_type() } => "0"),
                (typed { self.types.index_llvm_type() } => { fields.count }),
                (typed { self.types.index_llvm_type() } => "1"),
            ]
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
            emit_instruction!(
                self;
                extract_value { field.clone() };
                { runtime.llvm.clone() } => { value.representation.clone() },
                [{ index }]
            );
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
        emit_instruction!(
            self;
            insert_value { with_owner.clone() };
            { runtime.llvm.clone() } => "poison",
            (ptr) => { owner },
            [0]
        );
        let with_data = self.register();
        emit_instruction!(
            self;
            insert_value { with_data.clone() };
            { runtime.llvm.clone() } => { with_owner },
            (ptr) => { data },
            [1]
        );
        let result = self.register();
        emit_instruction!(
            self;
            insert_value { result.clone() };
            { runtime.llvm } => { with_data },
            { self.types.index_llvm_type() } => { count },
            [2]
        );
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }
}
