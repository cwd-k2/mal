use super::*;
use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
impl FunctionEmitter<'_> {
    pub(super) fn emit_state(&mut self, site: StateId) -> Option<()> {
        self.current_function = self.function_for_state(site)?;
        let state = &self.control.states[site.0];
        let self_tail_parameter = self
            .current_function()
            .filter(|function| function.entry == site)
            .filter(|function| self.optimizations.self_tail_parameter(function.id))
            .and_then(|function| self.execution.self_tail_parameters.get(function.id))
            .cloned();
        self.block(format!("mal_state_{}", site.0));
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            match self
                .optimizations
                .buffer_identity_binding(site, binding_index)
            {
                Some(super::super::optimization::BufferIdentityBinding::Value) => {}
                Some(super::super::optimization::BufferIdentityBinding::Put) => {
                    let unit = super::memory::emitted_unit();
                    self.store_binding_pattern(site, binding_index, &binding.pattern, Some(&unit))?;
                }
                None => {
                    let value = self.emit_operation(
                        site,
                        binding_index,
                        &binding.operation,
                        Some(binding.pattern.ty()),
                        self.optimizations.symbol_concat_mode(site, binding_index),
                    )?;
                    self.store_binding_pattern(
                        site,
                        binding_index,
                        &binding.pattern,
                        Some(&value),
                    )?;
                }
            }
            let mut drops = self
                .ownership
                .drops_after_binding(site, binding_index)
                .to_vec();
            drops.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
            for id in drops {
                self.release_dead_slot(id)?;
            }
            if self_tail_parameter
                .as_ref()
                .is_some_and(|parameter| parameter.binding_count == binding_index + 1)
            {
                let label = self_tail_entry_label(self.current_function);
                emit_terminator! {
                    self;
                    branch {
                        target: { label.clone() },
                    };
                };
                self.block(label);
            }
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let effect = self.ownership.terminator_operand_use(
                    site,
                    crate::execution::ownership::TerminatorOperand::Return,
                    value,
                )?;
                let value = self.prepare_atom_for_use(value, effect)?;
                let result_type = self.current_result_type()?;
                if value.value.ty != result_type {
                    return None;
                }
                self.commit_consumes(&value)?;
                self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                self.emit_continuation_return(site, &value.value)?;
            }
            Terminator::Goto(target) => {
                self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                emit_terminator! {
                    self;
                    branch {
                        target: { format!("mal_state_{}", target.0) },
                    };
                };
            }
            Terminator::Jump { target, value } => {
                let effect = self.ownership.terminator_operand_use(
                    site,
                    crate::execution::ownership::TerminatorOperand::JumpValue,
                    value,
                )?;
                let value = self.prepare_atom_for_use(value, effect)?;
                self.commit_consumes(&value)?;
                self.store_input_pattern(*target, Some(&value.value))?;
                self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                emit_terminator! {
                    self;
                    branch {
                        target: { format!("mal_state_{}", target.0) },
                    };
                };
            }
            Terminator::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => self.emit_primitive_branch(site, *operator, left, right, *otherwise, *then)?,
            Terminator::Call {
                callee,
                argument,
                resume,
            } => self.emit_call_terminator(site, callee, argument, *resume)?,
            Terminator::TailCall { callee, argument } => {
                self.emit_tail_call_terminator(site, callee, argument)?;
            }
            Terminator::Case { scrutinee, arms } => {
                self.require_terminator_borrow(
                    site,
                    crate::execution::ownership::TerminatorOperand::CaseScrutinee,
                    scrutinee,
                )?;
                self.emit_case(site, scrutinee, arms)?;
            }
        }
        Some(())
    }

    fn emit_primitive_branch(
        &mut self,
        site: StateId,
        operator: crate::core::ast::BinaryPrimitive,
        left: &crate::closure::ast::Atom,
        right: &crate::closure::ast::Atom,
        otherwise: StateId,
        then: StateId,
    ) -> Option<()> {
        self.require_terminator_borrow(
            site,
            crate::execution::ownership::TerminatorOperand::BranchLeft,
            left,
        )?;
        self.require_terminator_borrow(
            site,
            crate::execution::ownership::TerminatorOperand::BranchRight,
            right,
        )?;
        let left = self.atom(left)?;
        let right = self.atom(right)?;
        if left.ty != right.ty {
            return None;
        }
        let condition = self.register();
        let predicate = comparison_predicate(operator)?;
        let scalar = scalar_type(&left.ty, self.types.index_size())?;
        let (kind, predicate) = predicate.for_scalar(scalar);
        emit_instruction! {
            self;
            let { condition.clone() } = compare {
                kind: { kind },
                predicate: { predicate },
                ty: { scalar.llvm_type() },
                left: { left.representation },
                right: { right.representation },
            };
        };
        let then_label = self.branch_edge_label(
            site,
            crate::execution::ownership::ControlPath::BranchThen,
            then,
        );
        let otherwise_label = self.branch_edge_label(
            site,
            crate::execution::ownership::ControlPath::BranchOtherwise,
            otherwise,
        );
        emit_terminator! {
            self;
            branch {
                condition: { condition },
                then: { then_label },
                otherwise: { otherwise_label },
            };
        };
        self.emit_branch_edge_drops(
            site,
            crate::execution::ownership::ControlPath::BranchThen,
            then,
        )?;
        self.emit_branch_edge_drops(
            site,
            crate::execution::ownership::ControlPath::BranchOtherwise,
            otherwise,
        )
    }

    fn branch_edge_label(
        &self,
        site: StateId,
        path: crate::execution::ownership::ControlPath,
        target: StateId,
    ) -> String {
        if self.ownership.drops_on_edge(site, path).is_empty() {
            format!("mal_state_{}", target.0)
        } else {
            branch_edge_name(site, path)
        }
    }

    fn emit_branch_edge_drops(
        &mut self,
        site: StateId,
        path: crate::execution::ownership::ControlPath,
        target: StateId,
    ) -> Option<()> {
        if self.ownership.drops_on_edge(site, path).is_empty() {
            return Some(());
        }
        self.block(branch_edge_name(site, path));
        self.emit_edge_drops(site, path)?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_state_{}", target.0) },
            };
        };
        Some(())
    }

    fn emit_call_terminator(
        &mut self,
        site: StateId,
        callee: &crate::closure::ast::Atom,
        argument: &crate::closure::ast::Atom,
        resume: StateId,
    ) -> Option<()> {
        match self.execution.control_calls.mode(site)? {
            ControlCallMode::Direct(target) => {
                let result = self.emit_call(site, target, callee, argument, false)?;
                self.resume_after_call(site, resume, &result)
            }
            ControlCallMode::DirectRegion(_) if self.mode == EmissionMode::Native => {
                self.emit_native_self_call(site, callee, argument)
            }
            ControlCallMode::Dispatch | ControlCallMode::DirectRegion(_)
                if self.execution.control_frames.frame(site).is_some() =>
            {
                self.emit_frame_call(site, callee, argument)
            }
            ControlCallMode::Dispatch => {
                let result = self.emit_indirect_call(site, callee, argument, false)?;
                self.resume_after_call(site, resume, &result)
            }
            ControlCallMode::DirectSelfTail | ControlCallMode::DirectRegion(_) => None,
        }
    }

    fn resume_after_call(
        &mut self,
        site: StateId,
        resume: StateId,
        result: &EmittedValue,
    ) -> Option<()> {
        self.store_input_pattern(resume, Some(result))?;
        self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_state_{}", resume.0) },
            };
        };
        Some(())
    }

    fn emit_tail_call_terminator(
        &mut self,
        site: StateId,
        callee: &crate::closure::ast::Atom,
        argument: &crate::closure::ast::Atom,
    ) -> Option<()> {
        match self.execution.control_calls.mode(site)? {
            ControlCallMode::DirectSelfTail => self.emit_self_tail_call(site, argument),
            ControlCallMode::Direct(target) => {
                let result = self.emit_call(site, target, callee, argument, true)?;
                self.return_tail_call(site, &result)
            }
            ControlCallMode::DirectRegion(_) => {
                self.emit_region_transition(site, callee, argument, false, &[])
            }
            ControlCallMode::Dispatch
                if self.common_region.is_some()
                    && self.execution.control_regions.site_region(site) == self.common_region =>
            {
                self.emit_region_transition(site, callee, argument, false, &[])
            }
            ControlCallMode::Dispatch => {
                let result = self.emit_indirect_call(site, callee, argument, true)?;
                self.return_tail_call(site, &result)
            }
        }
    }

    fn return_tail_call(&mut self, site: StateId, result: &EmittedValue) -> Option<()> {
        self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
        self.emit_continuation_return(site, result)
    }

    fn emit_self_tail_call(
        &mut self,
        site: StateId,
        argument: &crate::closure::ast::Atom,
    ) -> Option<()> {
        let argument = self
            .execution
            .control_calls
            .forwarded_self_argument(site)
            .unwrap_or(argument);
        let effect = self.ownership.terminator_operand_use(
            site,
            crate::execution::ownership::TerminatorOperand::TailArgument,
            argument,
        )?;
        let value = self.prepare_atom_for_use(argument, effect)?;
        let function = self.current_function()?.clone();
        if function.parameter.ty != value.value.ty {
            return None;
        }
        self.commit_consumes(&value)?;
        let self_tail_parameter = self
            .optimizations
            .self_tail_parameter(function.id)
            .then(|| self.execution.self_tail_parameters.get(function.id))
            .flatten()
            .cloned();
        if let Some(parameter) = &self_tail_parameter {
            self.store_self_tail_pattern(&parameter.pattern, &value.value)?;
        } else {
            self.emit_parameter_handoff(
                function.id,
                &value.value,
                crate::execution::ownership::ParameterEntry::OwnedHandoff,
            )?;
        }
        self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
        let target = if self_tail_parameter.is_some() {
            self_tail_entry_label(function.id)
        } else {
            format!("mal_state_{}", function.entry.0)
        };
        emit_terminator! {
            self;
            branch {
                target: { target },
            };
        };
        Some(())
    }
}

fn branch_edge_name(site: StateId, path: crate::execution::ownership::ControlPath) -> String {
    let suffix = match path {
        crate::execution::ownership::ControlPath::BranchThen => "then",
        crate::execution::ownership::ControlPath::BranchOtherwise => "otherwise",
        crate::execution::ownership::ControlPath::Single
        | crate::execution::ownership::ControlPath::CaseArm(_) => {
            unreachable!("only primitive branch paths need edge blocks")
        }
    };
    format!("mal_edge_{}_{}", site.0, suffix)
}
