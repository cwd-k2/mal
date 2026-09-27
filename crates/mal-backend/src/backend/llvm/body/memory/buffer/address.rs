use mal_frontend::check::ast::Type;

use super::super::super::{EmittedValue, FunctionEmitter};
use super::emitted_buffer;

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_buffer_from_address(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Buffer(element) = result_type else {
            return None;
        };
        let argument_type = Type::Product(vec![Type::Address, Type::USize, Type::USize].into());
        if argument.ty != argument_type {
            return None;
        }
        let [address, offset, length] =
            self.product_fields(argument, [&Type::Address, &Type::USize, &Type::USize])?;
        let stride = self.source_layouts.layout(element)?.stride;
        let representation = self.register();
        emit_instruction!(
            self;
            call { Some(representation.clone()) },
            false,
            (ptr),
            direct "mal_runtime_buffer_from";
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => { address.representation }),
                (typed { self.types.index_llvm_type() } => { offset.representation }),
                (typed { self.types.index_llvm_type() } => { length.representation }),
                (typed { self.types.index_llvm_type() } => { stride.to_string() }),
            ]
        );
        Some(emitted_buffer(representation, result_type.clone()))
    }

    pub(in crate::backend::llvm::body) fn emit_buffer_into_address(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(elements) = &argument.ty else {
            return None;
        };
        let [buffer_type, address_type, offset_type, length_type] = elements.as_ref() else {
            return None;
        };
        let Type::Buffer(element) = buffer_type else {
            return None;
        };
        if address_type != &Type::Address
            || offset_type != &Type::USize
            || length_type != &Type::USize
            || result_type != &Type::Unit
        {
            return None;
        }
        let [buffer, address, offset, length] = self.product_fields(
            argument,
            [buffer_type, address_type, offset_type, length_type],
        )?;
        let stride = self.source_layouts.layout(element)?.stride;
        emit_instruction!(
            self;
            call None,
            false,
            (void),
            direct "mal_runtime_buffer_into";
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => { buffer.representation }),
                (typed (ptr) => { address.representation }),
                (typed { self.types.index_llvm_type() } => { offset.representation }),
                (typed { self.types.index_llvm_type() } => { length.representation }),
                (typed { self.types.index_llvm_type() } => { stride.to_string() }),
            ]
        );
        Some(EmittedValue {
            ty: Type::Unit,
            representation: "0".into(),
            owned: false,
        })
    }
}
