use super::*;

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
        let callee = self.atom(callee)?;
        let environment = self.closure_environment(&callee)?;
        let mut arguments = vec![
            (super::super::syntax::llvm_type!(ptr), "%mal_context".into()),
            (
                super::super::syntax::llvm_type!(ptr),
                "%mal_control_top".into(),
            ),
            (super::super::syntax::llvm_type!(ptr), environment),
        ];
        if target.parameter.ty == Type::Unit {
            if argument.ty != Type::Unit {
                return None;
            }
        } else {
            let argument = self.call_argument(site, argument_operand, argument)?;
            if argument.ty != target.parameter.ty {
                return None;
            }
            let argument_type = self.types.value(&argument.ty)?;
            arguments.push((argument_type.llvm, argument.representation));
        }
        let lowered = *self.index.lowered_functions.get(&target.id)?;
        let result_type = lowered.body.result.ty.clone();
        let result_value_type = self.types.value(&result_type)?;
        let register = self.register();
        self.sync_control_top()?;
        self.direct_call(
            Some(register.clone()),
            tail,
            result_value_type.llvm,
            function_name(target.id),
            arguments,
        );
        if self.optimizations.localizes_control_storage(target.id) {
            self.refresh_control_storage();
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
        let callee = self.atom(callee)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        let closure_type = self.types.value(&callee.ty)?;
        let code = self.register();
        self.extract_value(
            code.clone(),
            closure_type.llvm.clone(),
            callee.representation.clone(),
            [0],
        );
        let environment = self.register();
        self.extract_value(
            environment.clone(),
            closure_type.llvm,
            callee.representation,
            [1],
        );
        let mut arguments = vec![
            (super::super::syntax::llvm_type!(ptr), "%mal_context".into()),
            (
                super::super::syntax::llvm_type!(ptr),
                "%mal_control_top".into(),
            ),
            (super::super::syntax::llvm_type!(ptr), environment.clone()),
        ];
        if **parameter == Type::Unit {
            if argument.ty != Type::Unit {
                return None;
            }
        } else {
            let argument = self.call_argument(site, argument_operand, argument)?;
            if argument.ty != **parameter {
                return None;
            }
            let argument_type = self.types.value(&argument.ty)?;
            arguments.push((argument_type.llvm, argument.representation));
        }
        let result_type = self.types.value(result)?;
        let register = self.register();
        self.sync_control_top()?;
        self.indirect_call(
            Some(register.clone()),
            tail,
            result_type.llvm,
            code,
            arguments,
        );
        if self
            .optimizations
            .site_may_relocate_control_storage(&self.execution.applications, site)
        {
            self.refresh_control_storage();
        }
        Some(EmittedValue {
            ty: (**result).clone(),
            representation: register,
            owned: crate::execution::ownership::is_managed(result),
        })
    }

    /// The argument of a native call: borrowed, or handed to the callee when the call site passes it owned.
    fn call_argument(
        &mut self,
        site: StateId,
        operand: crate::execution::ownership::TerminatorOperand,
        argument: &Atom,
    ) -> Option<EmittedValue> {
        if !self.ownership.passes_owned_argument(site)
            || !crate::execution::ownership::is_managed(&argument.ty)
        {
            self.require_terminator_borrow(site, operand, argument)?;
            return self.atom(argument);
        }
        let effect = self.ownership.terminator_use(site, operand)?;
        if effect == crate::execution::ownership::UseEffect::Borrow {
            return None;
        }
        let prepared = self.prepare_atom_for_use(argument, effect)?;
        self.commit_consumes(&prepared)?;
        Some(prepared.value)
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

    pub(super) fn entry_alloca(
        &mut self,
        llvm: &super::super::syntax::Type,
        alignment: usize,
    ) -> String {
        let storage = format!("%mal_alloca_{}", self.next_entry_alloca);
        self.next_entry_alloca += 1;
        let Some(function) = self.current_definition.as_mut() else {
            self.emission_failed = true;
            return storage;
        };
        let Some(instruction) = super::super::syntax::llvm_instruction!(alloca storage.clone(), llvm.clone(), alignment)
        else {
            self.emission_failed = true;
            return storage;
        };
        function.structured_entry_instruction(instruction);
        storage
    }

    pub(super) fn label_id(&mut self) -> usize {
        let id = self.next_register;
        self.next_register += 1;
        id
    }

    pub(super) fn block(&mut self, label: impl Into<String>) {
        let Some(function) = self.current_definition.as_mut() else {
            self.emission_failed = true;
            return;
        };
        self.emission_failed |= !function.start_block(label);
    }

    pub(super) fn structured_instruction(
        &mut self,
        instruction: Option<super::super::syntax::Instruction>,
    ) {
        let Some(instruction) = instruction else {
            self.emission_failed = true;
            return;
        };
        let Some(function) = self.current_definition.as_mut() else {
            self.emission_failed = true;
            return;
        };
        self.emission_failed |= !function.structured_instruction(instruction);
    }

    pub(super) fn load(
        &mut self,
        result: impl Into<String>,
        ty: super::super::syntax::Type,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = super::super::syntax::MetadataAttachment>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(load
            result, ty, pointer, alignment, metadata,
        ));
    }

    pub(super) fn store(
        &mut self,
        ty: super::super::syntax::Type,
        value: impl Into<String>,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = super::super::syntax::MetadataAttachment>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(store
            ty, value, pointer, alignment, metadata,
        ));
    }

    pub(super) fn direct_call(
        &mut self,
        result: Option<String>,
        tail: bool,
        result_type: super::super::syntax::Type,
        callee: impl Into<String>,
        arguments: impl IntoIterator<Item = (super::super::syntax::Type, String)>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            call result, tail, result_type, direct callee, arguments
        ));
    }

    pub(super) fn indirect_call(
        &mut self,
        result: Option<String>,
        tail: bool,
        result_type: super::super::syntax::Type,
        callee: impl Into<String>,
        arguments: impl IntoIterator<Item = (super::super::syntax::Type, String)>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            call result, tail, result_type, indirect callee, arguments
        ));
    }

    pub(super) fn unary(
        &mut self,
        result: impl Into<String>,
        operator: super::super::syntax::UnaryOperator,
        ty: super::super::syntax::Type,
        value: impl Into<String>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            unary result, operator; ty => value
        ));
    }

    pub(super) fn binary(
        &mut self,
        result: impl Into<String>,
        operator: super::super::syntax::BinaryOperator,
        ty: super::super::syntax::Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(binary
            result, operator, ty, left, right,
        ));
    }

    pub(super) fn compare(
        &mut self,
        result: impl Into<String>,
        kind: super::super::syntax::ComparisonKind,
        predicate: super::super::syntax::ComparisonPredicate,
        ty: super::super::syntax::Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(compare
            result, kind, predicate, ty, left, right,
        ));
    }

    pub(super) fn cast(
        &mut self,
        result: impl Into<String>,
        operator: super::super::syntax::CastOperator,
        source_type: super::super::syntax::Type,
        source: impl Into<String>,
        target: super::super::syntax::Type,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            cast result, operator; source_type => source, target
        ));
    }

    pub(super) fn get_element_ptr(
        &mut self,
        result: impl Into<String>,
        inbounds: bool,
        element_type: super::super::syntax::Type,
        pointer: impl Into<String>,
        indices: impl IntoIterator<Item = (super::super::syntax::Type, String)>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            get_element_ptr result, inbounds, element_type, pointer, indices
        ));
    }

    pub(super) fn extract_value(
        &mut self,
        result: impl Into<String>,
        aggregate_type: super::super::syntax::Type,
        aggregate: impl Into<String>,
        indices: impl IntoIterator<Item = usize>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(
            extract_value result; aggregate_type => aggregate, indices
        ));
    }

    pub(super) fn insert_value(
        &mut self,
        result: impl Into<String>,
        aggregate_type: super::super::syntax::Type,
        aggregate: impl Into<String>,
        element_type: super::super::syntax::Type,
        element: impl Into<String>,
        indices: impl IntoIterator<Item = usize>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(insert_value
            result; aggregate_type => aggregate, element_type => element, indices
        ));
    }

    pub(super) fn phi(
        &mut self,
        result: impl Into<String>,
        ty: super::super::syntax::Type,
        incoming: impl IntoIterator<Item = (String, String)>,
    ) {
        self.structured_instruction(super::super::syntax::llvm_instruction!(phi
            result, ty, incoming
        ));
    }

    pub(super) fn terminate(&mut self, terminator: Option<super::super::syntax::Terminator>) {
        let Some(terminator) = terminator else {
            self.emission_failed = true;
            return;
        };
        let Some(function) = self.current_definition.as_mut() else {
            self.emission_failed = true;
            return;
        };
        self.emission_failed |= !function.terminate(terminator);
    }

    pub(super) fn unreachable(&mut self) {
        self.terminate(super::super::syntax::llvm_terminator!(unreachable));
    }

    pub(super) fn return_void(&mut self) {
        self.terminate(super::super::syntax::llvm_terminator!(return_void));
    }

    pub(super) fn branch(&mut self, target: impl Into<String>) {
        self.terminate(super::super::syntax::llvm_terminator!(branch target));
    }

    pub(super) fn conditional_branch(
        &mut self,
        condition: impl Into<String>,
        then_target: impl Into<String>,
        else_target: impl Into<String>,
    ) {
        self.terminate(super::super::syntax::llvm_terminator!(conditional
            condition => then_target, else_target
        ));
    }

    pub(super) fn return_value(
        &mut self,
        ty: super::super::syntax::Type,
        value: impl Into<String>,
    ) {
        self.terminate(super::super::syntax::llvm_terminator!(return ty => value));
    }

    pub(super) fn begin_function(&mut self, signature: super::super::syntax::FunctionSignature) {
        if self.current_definition.is_some() {
            self.emission_failed = true;
            return;
        }
        self.current_definition = Some(super::super::syntax::FunctionBuilder::new(signature));
    }

    pub(super) fn finish_function(&mut self) -> Option<()> {
        let definition = self.current_definition.take()?.finish()?;
        self.definitions.push(definition);
        Some(())
    }
}
