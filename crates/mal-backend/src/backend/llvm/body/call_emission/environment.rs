use crate::backend::llvm::syntax::llvm_signature;
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
        self.begin_function(llvm_signature!(
            #[linkage(internal)]
            fn {{ format!(
                "mal_destroy_environment_{}",
                function_number(self.function.id)
            ) }}(
                "%mal_environment" : ptr,
            ) -> void
        ));
        self.block("entry");
        let environment = self.register();
        emit_instruction!(
            self;
            let {{ environment.clone() }} = load {
                ty: {{ value_type.llvm }},
                pointer: "%mal_environment",
                alignment: {{ value_type.alignment }},
                metadata: [],
            };
        );
        self.release_value(&environment_type, &environment)?;
        emit_terminator!(
            self;
            return;
        );
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
        emit_instruction!(
            self;
            let {{ environment.clone() }} = extract_value {
                aggregate: typed({{ closure_type.llvm }}, {{ closure.representation.clone() }}),
                indices: [1],
            };
        );
        Some(environment)
    }

    pub(in crate::backend::llvm::body) fn active_environment(&mut self) -> String {
        if self.common_region.is_none() {
            return "%mal_environment".into();
        }
        let environment = self.register();
        emit_instruction!(
            self;
            let {{ environment.clone() }} = load {
                ty: (ptr),
                pointer: "%mal_active_environment",
                alignment: {{ self.types.pointer_alignment() }},
                metadata: [],
            };
        );
        environment
    }
}
