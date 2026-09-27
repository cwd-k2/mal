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
            let {{ count.clone() }} = call {
                tail: false,
                result_type: {{ self.types.index_llvm_type() }},
                callee: direct("mal_runtime_buffer_count"),
                arguments: [typed((ptr), {{ buffer.representation.clone() }})],
            };
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
            let {{ count.clone() }} = call {
                tail: false,
                result_type: {{ self.types.index_llvm_type() }},
                callee: direct("mal_runtime_buffer_count"),
                arguments: [typed((ptr), {{ buffer.representation.clone() }})],
            };
        );
        let owner = self.register();
        emit_instruction!(
            self;
            let {{ owner.clone() }} = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_bytes_read"),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), {{ data }}), typed({{ self.types.index_llvm_type() }}, {{ count.clone() }})],
            };
        );
        let copied_data = self.register();
        emit_instruction!(
            self;
            let {{ copied_data.clone() }} = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_bytes_data"),
                arguments: [typed((ptr), {{ owner.clone() }})],
            };
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
            let {{ buffer.clone() }} = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_buffer_from"),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), {{ fields.data }}), typed({{ self.types.index_llvm_type() }}, "0"), typed({{ self.types.index_llvm_type() }}, {{ fields.count }}), typed({{ self.types.index_llvm_type() }}, "1")],
            };
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
                let {{ field.clone() }} = extract_value {
                    aggregate: typed({{ runtime.llvm.clone() }}, {{ value.representation.clone() }}),
                    indices: [{{ index }}],
                };
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
            let {{ with_owner.clone() }} = insert_value {
                aggregate: typed({{ runtime.llvm.clone() }}, "poison"),
                element: typed((ptr), {{ owner }}),
                indices: [0],
            };
        );
        let with_data = self.register();
        emit_instruction!(
            self;
            let {{ with_data.clone() }} = insert_value {
                aggregate: typed({{ runtime.llvm.clone() }}, {{ with_owner }}),
                element: typed((ptr), {{ data }}),
                indices: [1],
            };
        );
        let result = self.register();
        emit_instruction!(
            self;
            let {{ result.clone() }} = insert_value {
                aggregate: typed({{ runtime.llvm }}, {{ with_data }}),
                element: typed({{ self.types.index_llvm_type() }}, {{ count }}),
                indices: [2],
            };
        );
        Some(EmittedValue {
            ty: ty.clone(),
            representation: result,
            owned,
        })
    }
}
