use crate::backend::llvm::syntax::llvm_type;
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
            [(llvm_type!(ptr), buffer.representation.clone())],
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
            [(llvm_type!(ptr), buffer.representation.clone())],
        );
        let owner = self.register();
        self.direct_call(
            Some(owner.clone()),
            false,
            llvm_type!(ptr),
            "mal_runtime_bytes_read",
            [
                (llvm_type!(ptr), "%mal_context".into()),
                (llvm_type!(ptr), data),
                (self.types.index_llvm_type(), count.clone()),
            ],
        );
        let copied_data = self.register();
        self.direct_call(
            Some(copied_data.clone()),
            false,
            llvm_type!(ptr),
            "mal_runtime_bytes_data",
            [(llvm_type!(ptr), owner.clone())],
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
            llvm_type!(ptr),
            "mal_runtime_buffer_from",
            [
                (llvm_type!(ptr), "%mal_context".into()),
                (llvm_type!(ptr), fields.data),
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
            self.extract_value(
                field.clone(),
                runtime.llvm.clone(),
                value.representation.clone(),
                [index],
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
        self.insert_value(
            with_owner.clone(),
            runtime.llvm.clone(),
            "poison",
            llvm_type!(ptr),
            owner,
            [0],
        );
        let with_data = self.register();
        self.insert_value(
            with_data.clone(),
            runtime.llvm.clone(),
            with_owner,
            llvm_type!(ptr),
            data,
            [1],
        );
        let result = self.register();
        self.insert_value(
            result.clone(),
            runtime.llvm,
            with_data,
            self.types.index_llvm_type(),
            count,
            [2],
        );
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }
}
