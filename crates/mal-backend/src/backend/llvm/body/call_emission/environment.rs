use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter, function_number};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_environment_destructor(&mut self) -> Option<()> {
        let lowered = *self.index.lowered_functions.get(&self.current_function)?;
        let captures = &lowered.captures;
        if captures.is_empty() {
            return Some(());
        }
        let environment_type =
            Type::Product(captures.iter().map(|field| field.ty.clone()).collect());
        let value_type = self.types.value(&environment_type)?;
        self.begin_function(
            crate::backend::llvm::syntax::FunctionSignature::new(
                crate::backend::llvm::syntax::Type::Void,
                format!(
                    "mal_destroy_environment_{}",
                    function_number(self.function.id)
                ),
                [crate::backend::llvm::syntax::Parameter::named(
                    crate::backend::llvm::syntax::Type::Pointer,
                    "%mal_environment",
                )],
            )
            .with_linkage(crate::backend::llvm::syntax::Linkage::Internal),
        );
        self.block("entry");
        let environment = self.register();
        self.load(
            environment.clone(),
            value_type.llvm,
            "%mal_environment",
            value_type.alignment,
            [],
        );
        self.release_value(&environment_type, &environment)?;
        self.return_void();
        self.finish_function()?;
        self.next_register = 0;
        Some(())
    }

    pub(in crate::backend::llvm::body) fn closure_environment(
        &mut self,
        closure: &EmittedValue,
    ) -> Option<String> {
        let closure_type = self.types.value(&closure.ty)?;
        let environment = self.register();
        self.extract_value(
            environment.clone(),
            closure_type.llvm,
            closure.representation.clone(),
            [1],
        );
        Some(environment)
    }

    pub(in crate::backend::llvm::body) fn active_environment(&mut self) -> String {
        if self.common_region.is_none() {
            return "%mal_environment".into();
        }
        let environment = self.register();
        self.load(
            environment.clone(),
            crate::backend::llvm::syntax::Type::Pointer,
            "%mal_active_environment",
            self.types.pointer_alignment(),
            [],
        );
        environment
    }
}
