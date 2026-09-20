use super::*;

mod buffer;
mod environment;
mod parameter;

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
            let target_uses_direct_buffer = self.optimizations.uses_direct_buffer(target.id);
            let argument = self.adapt_buffer_argument(argument, target_uses_direct_buffer)?;
            self.call_arguments(&environment, &argument, target_uses_direct_buffer)?
        };
        let lowered = *self.index.lowered_functions.get(&target.id)?;
        let result_type = lowered.body.result.ty.clone();
        let result_value_type = self.types.value(&result_type)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.sync_control_top()?;
        self.line(format!(
            "  {register} = {tail}call {} @{}({arguments})",
            result_value_type.llvm,
            function_name(target.id)?
        ));
        if self.optimizations.localizes_control_storage(target.id) {
            self.refresh_control_storage()?;
        }
        Some(EmittedValue {
            ty: result_type,
            representation: register,
            owned: crate::execution::ownership::is_managed(&lowered.body.result.ty),
        })
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
            let target_uses_direct_buffer = self
                .optimizations
                .site_uses_direct_buffer(&self.execution.applications, site);
            let argument = self.adapt_buffer_argument(argument, target_uses_direct_buffer)?;
            self.call_arguments(&environment, &argument, target_uses_direct_buffer)?
        };
        let result_type = self.types.value(result)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.sync_control_top()?;
        self.line(format!(
            "  {register} = {tail}call {} {code}({arguments})",
            result_type.llvm
        ));
        if self
            .optimizations
            .site_may_relocate_control_storage(&self.execution.applications, site)
        {
            self.refresh_control_storage()?;
        }
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

    pub(super) fn current_result_type(&self) -> Option<Type> {
        self.index
            .lowered_functions
            .get(&self.current_function)
            .map(|function| function.body.result.ty.clone())
    }

    pub(super) fn function_for_state(&self, site: StateId) -> Option<FunctionId> {
        self.state_functions.get(&site).copied()
    }

    pub(super) fn register(&mut self) -> String {
        let register = format!("%mal_value_{}", self.next_register);
        self.next_register += 1;
        register
    }

    pub(super) fn entry_alloca(&mut self, llvm: &str, alignment: usize) -> String {
        let storage = format!("%mal_alloca_{}", self.next_entry_alloca);
        self.next_entry_alloca += 1;
        self.entry_allocas
            .push_str(&format!("  {storage} = alloca {llvm}, align {alignment}\n"));
        storage
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
