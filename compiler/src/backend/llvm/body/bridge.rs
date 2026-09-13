use crate::check::ast::Type;
use crate::closure::ast::Atom;
use crate::resolve::ast::ExternalOperationId;

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
        if !crate::backend::llvm::bridge_type_supported(&argument.ty) {
            return None;
        }
        let argument_type = self.types.value(&argument.ty)?;
        let result_type = result_type.clone();
        if !crate::backend::llvm::bridge_type_supported(&result_type) {
            return None;
        }
        let result_value_type = self.types.value(&result_type)?;
        self.line(format!(
            "  store {} {}, ptr %mal_bridge_argument, align {}",
            argument_type.llvm, argument.representation, argument_type.alignment
        ));
        let bridge = crate::backend::abi::Function::external_bridge(id);
        let call = bridge
            .llvm_signature()
            .replace("%mal_argument", "%mal_bridge_argument")
            .replace("%mal_result", "%mal_bridge_result");
        self.line(format!("  call {call}"));
        let register = self.register();
        self.line(format!(
            "  {register} = load {}, ptr %mal_bridge_result, align {}",
            result_value_type.llvm, result_value_type.alignment
        ));
        Some(EmittedValue {
            owned: crate::execution::ownership::is_managed(&result_type),
            ty: result_type,
            representation: register,
        })
    }
}
