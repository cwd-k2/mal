use crate::closure::ast::Atom;
use mal_frontend::check::ast::Type;
use mal_frontend::resolve::ast::ExternalOperationId;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_external_call(
        &mut self,
        id: ExternalOperationId,
        argument: &Atom,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let external = *self.index.externals.get(&id)?;
        let argument = self.atom(argument)?;
        if argument.ty != external.parameter || *result_type != external.result {
            return None;
        }
        let argument_type = self.types.value(&argument.ty)?;
        let result_type = result_type.clone();
        let result_value_type = self.types.value(&result_type)?;
        emit_instruction!(
            self;
            store { argument_type.llvm },
            { argument.representation.as_str() },
            "%mal_bridge_argument",
            { argument_type.alignment },
            []
        );
        let bridge = crate::backend::abi::Function::external_bridge(id);
        emit_instruction!(
            self;
            call None,
            false,
            (void),
            direct { bridge.name() };
            [
                (typed (ptr) => "%mal_context"),
                (typed (ptr) => "%mal_bridge_argument"),
                (typed (ptr) => "%mal_bridge_result"),
            ]
        );
        let register = self.register();
        emit_instruction!(
            self;
            load { register.clone() },
            { result_value_type.llvm },
            "%mal_bridge_result",
            { result_value_type.alignment },
            []
        );
        Some(EmittedValue {
            owned: false,
            ty: result_type,
            representation: register,
        })
    }
}
