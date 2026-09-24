use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter, function_number};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_environment_destructor(&mut self) -> Option<()> {
        let lowered = *self.index.lowered_functions.get(&self.current_function)?;
        let Some(captures) = lowered.kind.captures() else {
            return Some(());
        };
        if captures.is_empty() {
            return Some(());
        }
        let environment_type =
            Type::Product(captures.iter().map(|field| field.ty.clone()).collect());
        let value_type = self.types.value(&environment_type)?;
        self.line(format!(
            "define internal void @mal_destroy_environment_{}(ptr %mal_environment) {{",
            function_number(self.function.id)?
        ));
        self.line("entry:");
        let environment = self.register();
        self.line(format!(
            "  {environment} = load {}, ptr %mal_environment, align {}",
            value_type.llvm, value_type.alignment
        ));
        self.release_value(&environment_type, &environment)?;
        self.line("  ret void");
        self.line("}");
        self.line("");
        self.next_register = 0;
        Some(())
    }

    pub(in crate::backend::llvm::body) fn closure_environment(
        &mut self,
        closure: &EmittedValue,
    ) -> Option<String> {
        let closure_type = self.types.value(&closure.ty)?;
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, closure.representation
        ));
        Some(environment)
    }

    pub(in crate::backend::llvm::body) fn active_environment(&mut self) -> String {
        if self.common_region.is_none() {
            return "%mal_environment".into();
        }
        let environment = self.register();
        self.line(format!(
            "  {environment} = load ptr, ptr %mal_active_environment, align {}",
            self.types.pointer_alignment()
        ));
        environment
    }
}
