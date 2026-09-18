use super::*;
impl FunctionEmitter<'_> {
    pub(super) fn emit_call(
        &mut self,
        site: StateId,
        target: FunctionId,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let target = *self.index.control_functions.get(&target)?;
        let (callee_operand, argument_operand) = if tail {
            (
                crate::execution::ownership::TerminatorOperand::TailCallee,
                crate::execution::ownership::TerminatorOperand::TailArgument,
            )
        } else {
            (
                crate::execution::ownership::TerminatorOperand::CallCallee,
                crate::execution::ownership::TerminatorOperand::CallArgument,
            )
        };
        self.require_terminator_borrow(site, callee_operand, callee)?;
        self.require_terminator_borrow(site, argument_operand, argument)?;
        let callee = self.atom(callee)?;
        let environment = self.closure_environment(&callee)?;
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
        let lowered = *self.index.lowered_functions.get(&target.id)?;
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
        entry: crate::execution::ownership::ParameterEntry,
    ) -> Option<()> {
        let function = *self.index.control_functions.get(&target)?;
        if function.parameter.ty != value.ty {
            return None;
        }
        let destination = self.execution.parameters.destination(target)?;
        if crate::execution::ownership::is_managed(&value.ty) {
            let effect = self.ownership.parameter_effect(target, entry);
            match (entry, effect, destination) {
                (
                    crate::execution::ownership::ParameterEntry::BorrowedAbi,
                    Some(crate::execution::ownership::ParameterEffect::ShareInto(binding)),
                    _,
                ) if !value.owned => {
                    let mut value = value.clone();
                    self.retain_if_borrowed(&mut value)?;
                    return self.store_parameter_binding(binding, &value);
                }
                (crate::execution::ownership::ParameterEntry::BorrowedAbi, None, _)
                    if !value.owned =>
                {
                    return Some(());
                }
                (
                    crate::execution::ownership::ParameterEntry::OwnedHandoff,
                    Some(crate::execution::ownership::ParameterEffect::ConsumeInto(binding)),
                    _,
                ) if value.owned => return self.store_parameter_binding(binding, value),
                (
                    crate::execution::ownership::ParameterEntry::OwnedHandoff,
                    Some(crate::execution::ownership::ParameterEffect::Drop),
                    _,
                ) if value.owned => {
                    return self.release_value(&value.ty, &value.representation);
                }
                _ => return None,
            }
        }
        match destination {
            ParameterDestination::Bind(binding) => {
                self.store_parameter_binding(binding, value)?;
            }
            ParameterDestination::Discard => {}
        }
        Some(())
    }

    fn store_parameter_binding(
        &mut self,
        binding: crate::anf::ast::ValueId,
        value: &EmittedValue,
    ) -> Option<()> {
        let slot = self.slots.get(&binding)?.clone();
        if slot.ty != value.ty {
            return None;
        }
        let value_type = self.types.value(&slot.ty)?;
        self.line(format!(
            "  store {} {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, value.representation, slot.index, value_type.alignment
        ));
        Some(())
    }

    pub(super) fn emit_environment_destructor(&mut self) -> Option<()> {
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

    pub(super) fn emit_indirect_call(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let (callee_operand, argument_operand) = if tail {
            (
                crate::execution::ownership::TerminatorOperand::TailCallee,
                crate::execution::ownership::TerminatorOperand::TailArgument,
            )
        } else {
            (
                crate::execution::ownership::TerminatorOperand::CallCallee,
                crate::execution::ownership::TerminatorOperand::CallArgument,
            )
        };
        self.require_terminator_borrow(site, callee_operand, callee)?;
        self.require_terminator_borrow(site, argument_operand, argument)?;
        let callee = self.atom(callee)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        let (code, environment) = if let Some(target) = self.types.compact_function(&callee.ty) {
            (
                format!("@{}", super::function_name(target)?),
                callee.representation.clone(),
            )
        } else {
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
            (code, environment)
        };
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
        self.index
            .control_functions
            .get(&self.current_function)
            .copied()
    }

    pub(super) fn closure_environment(&mut self, closure: &EmittedValue) -> Option<String> {
        if self.types.function_is_compact(&closure.ty) {
            return Some(closure.representation.clone());
        }
        let closure_type = self.types.value(&closure.ty)?;
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, closure.representation
        ));
        Some(environment)
    }

    pub(super) fn current_result_type(&self) -> Option<Type> {
        self.index
            .lowered_functions
            .get(&self.current_function)
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
            self.types.pointer_alignment()
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
