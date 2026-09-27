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
            store {
                value: typed({{ argument_type.llvm }}, {{ argument.representation.as_str() }}),
                pointer: "%mal_bridge_argument",
                alignment: {{ argument_type.alignment }},
                metadata: [],
            };
        );
        let bridge = crate::backend::abi::Function::external_bridge(id);
        emit_instruction!(
            self;
            call {
                tail: false,
                result_type: (void),
                callee: direct({{ bridge.name() }}),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), "%mal_bridge_argument"), typed((ptr), "%mal_bridge_result")],
            };
        );
        let register = self.register();
        emit_instruction!(
            self;
            let {{ register.clone() }} = load {
                ty: {{ result_value_type.llvm }},
                pointer: "%mal_bridge_result",
                alignment: {{ result_value_type.alignment }},
                metadata: [],
            };
        );
        Some(EmittedValue {
            owned: false,
            ty: result_type,
            representation: register,
        })
    }
}
