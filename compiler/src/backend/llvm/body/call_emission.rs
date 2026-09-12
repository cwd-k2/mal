use super::*;
impl FunctionEmitter<'_> {
    pub(super) fn emit_call(
        &mut self,
        target: FunctionId,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let target = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        let callee = self.atom(callee)?;
        let closure_type = self.types.value(&callee.ty)?;
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.representation
        ));
        let arguments = if target.parameter.ty == Type::Unit {
            format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != target.parameter.ty {
                return None;
            }
            let value_type = self.types.value(&argument.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
                value_type.llvm, argument.representation
            )
        };
        let lowered = self
            .execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == target.id)?;
        let result_type = lowered.body.result.ty.clone();
        let result_value_type = self.types.value(&result_type)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call {} @{}({arguments})",
            result_value_type.llvm,
            function_name(target.id)?
        ));
        Some(EmittedValue {
            ty: result_type,
            representation: register,
            owned: crate::execution::ownership::is_managed(&lowered.body.result.ty),
        })
    }

    pub(super) fn emit_parameter_handoff(
        &mut self,
        target: FunctionId,
        value: &EmittedValue,
    ) -> Option<()> {
        let function = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        if function.parameter.ty != value.ty
            || (crate::execution::ownership::is_managed(&value.ty) && !value.owned)
        {
            return None;
        }
        match self.execution.parameters.destination(target)? {
            ParameterDestination::Bind(binding) => {
                let slot = self.slots.get(&binding)?.clone();
                if slot.ty != value.ty {
                    return None;
                }
                let value_type = self.types.value(&slot.ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
                ));
            }
            ParameterDestination::Discard if value.owned => {
                self.release_value(&value.ty, &value.representation)?;
            }
            ParameterDestination::Discard => {}
        }
        Some(())
    }

    pub(super) fn emit_environment_destructor(&mut self) -> Option<()> {
        if self.function.environment.is_empty() {
            return Some(());
        }
        let environment_type = Type::Product(
            self.function
                .environment
                .iter()
                .map(|field| field.ty.clone())
                .collect(),
        );
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

    pub(super) fn emit_indirect_call(
        &mut self,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let callee = self.atom(callee)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        let closure_type = self.types.value(&callee.ty)?;
        let code = self.register();
        self.line(format!(
            "  {code} = extractvalue {} {}, 0",
            closure_type.llvm, callee.representation
        ));
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.representation
        ));
        let arguments = if **parameter == Type::Unit {
            if argument.ty != Type::Unit {
                return None;
            }
            format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != **parameter {
                return None;
            }
            let value_type = self.types.value(parameter)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
                value_type.llvm, argument.representation
            )
        };
        let result_type = self.types.value(result)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call {} {code}({arguments})",
            result_type.llvm
        ));
        Some(EmittedValue {
            ty: (**result).clone(),
            representation: register,
            owned: crate::execution::ownership::is_managed(result),
        })
    }

    pub(super) fn current_function(&self) -> Option<&crate::control::ast::Function> {
        self.control
            .functions
            .iter()
            .find(|function| function.id == self.current_function)
    }

    pub(super) fn current_result_type(&self) -> Option<Type> {
        self.execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == self.current_function)
            .map(|function| function.body.result.ty.clone())
    }

    pub(super) fn function_for_state(&self, site: StateId) -> Option<FunctionId> {
        self.state_functions.get(&site).copied()
    }

    pub(super) fn active_environment(&mut self) -> String {
        if self.common_region.is_none() {
            return "%mal_environment".into();
        }
        let environment = self.register();
        self.line(format!(
            "  {environment} = load ptr, ptr %mal_active_environment, align {}",
            self.types.pointer_size()
        ));
        environment
    }

    pub(super) fn register(&mut self) -> String {
        let register = format!("%mal_value_{}", self.next_register);
        self.next_register += 1;
        register
    }

    pub(super) fn label_id(&mut self) -> usize {
        let id = self.next_register;
        self.next_register += 1;
        id
    }

    pub(super) fn line(&mut self, line: impl AsRef<str>) {
        self.output.push_str(line.as_ref());
        self.output.push('\n');
    }
}
