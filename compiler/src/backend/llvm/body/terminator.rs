use super::*;
impl FunctionEmitter<'_> {
    pub(super) fn emit_state(&mut self, site: StateId) -> Option<()> {
        self.current_function = self.function_for_state(site)?;
        let state = &self.control.states[site.0];
        self.line(format!("mal_state_{}:", site.0));
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let value = self.emit_operation(
                site,
                binding_index,
                &binding.operation,
                pattern_value_type(&binding.pattern),
                self.optimizations.symbol_concat_mode(site, binding_index),
            )?;
            self.store_binding_pattern(site, binding_index, &binding.pattern, value.as_ref())?;
            let mut drops = self
                .ownership
                .drops_after_binding(site, binding_index)
                .to_vec();
            drops.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
            for id in drops {
                self.release_dead_slot(id)?;
            }
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let effect = self
                    .ownership
                    .terminator_use(site, crate::execution::ownership::TerminatorOperand::Return)
                    .or_else(|| {
                        (!crate::execution::ownership::is_managed(&value.ty))
                            .then_some(crate::execution::ownership::UseEffect::Borrow)
                    })?;
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
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::Jump { target, value } => {
                let effect = self
                    .ownership
                    .terminator_use(
                        site,
                        crate::execution::ownership::TerminatorOperand::JumpValue,
                    )
                    .or_else(|| {
                        (!crate::execution::ownership::is_managed(&value.ty))
                            .then_some(crate::execution::ownership::UseEffect::Borrow)
                    })?;
                let value = self.prepare_atom_for_use(value, effect)?;
                self.commit_consumes(&value)?;
                self.store_input_pattern(*target, Some(&value.value))?;
                self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
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
                if left.ty == Type::Symbol {
                    let (left_owner, left_offset, left_length) = self.byte_view_fields(&left)?;
                    let (right_owner, right_offset, right_length) =
                        self.byte_view_fields(&right)?;
                    let equality = self.register();
                    self.line(format!(
                        "  {equality} = call i8 @mal_runtime_symbol_equal(ptr {left_owner}, {0} {left_offset}, {0} {left_length}, ptr {right_owner}, {0} {right_offset}, {0} {right_length})",
                        self.types.pointer_integer()?
                    ));
                    let predicate = match operator {
                        crate::core::ast::BinaryPrimitive::Equal => "ne",
                        crate::core::ast::BinaryPrimitive::NotEqual => "eq",
                        _ => return None,
                    };
                    self.line(format!("  {condition} = icmp {predicate} i8 {equality}, 0"));
                } else if is_bool(&left.ty) {
                    let predicate = match operator {
                        crate::core::ast::BinaryPrimitive::Equal => "eq",
                        crate::core::ast::BinaryPrimitive::NotEqual => "ne",
                        _ => return None,
                    };
                    self.line(format!(
                        "  {condition} = icmp {predicate} i1 {}, {}",
                        left.representation, right.representation
                    ));
                } else {
                    let predicate = comparison_predicate(*operator)?;
                    let scalar = scalar_type(&left.ty, self.types.index_size())?;
                    let predicate = predicate.for_scalar(scalar);
                    let instruction = if scalar.floating { "fcmp" } else { "icmp" };
                    self.line(format!(
                        "  {condition} = {instruction} {predicate} {} {}, {}",
                        scalar.llvm, left.representation, right.representation
                    ));
                }
                let then_drops = !self
                    .ownership
                    .drops_on_edge(site, crate::execution::ownership::ControlPath::BranchThen)
                    .is_empty();
                let otherwise_drops = !self
                    .ownership
                    .drops_on_edge(
                        site,
                        crate::execution::ownership::ControlPath::BranchOtherwise,
                    )
                    .is_empty();
                let then_label = if then_drops {
                    format!("mal_edge_{}_then", site.0)
                } else {
                    format!("mal_state_{}", then.0)
                };
                let otherwise_label = if otherwise_drops {
                    format!("mal_edge_{}_otherwise", site.0)
                } else {
                    format!("mal_state_{}", otherwise.0)
                };
                self.line(format!(
                    "  br i1 {condition}, label %{then_label}, label %{otherwise_label}"
                ));
                if then_drops {
                    self.line(format!("mal_edge_{}_then:", site.0));
                    self.emit_edge_drops(
                        site,
                        crate::execution::ownership::ControlPath::BranchThen,
                    )?;
                    self.line(format!("  br label %mal_state_{}", then.0));
                }
                if otherwise_drops {
                    self.line(format!("mal_edge_{}_otherwise:", site.0));
                    self.emit_edge_drops(
                        site,
                        crate::execution::ownership::ControlPath::BranchOtherwise,
                    )?;
                    self.line(format!("  br label %mal_state_{}", otherwise.0));
                }
            }
            Terminator::Call {
                callee,
                argument,
                resume,
            } => match self.execution.control_calls.mode(site)? {
                ControlCallMode::Direct(target) => {
                    let result = self.emit_call(site, target, callee, argument, false)?;
                    self.store_input_pattern(*resume, Some(&result))?;
                    self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                    self.line(format!("  br label %mal_state_{}", resume.0));
                }
                ControlCallMode::Dispatch
                    if self.execution.control_frames.frame(site).is_some() =>
                {
                    self.emit_frame_call(site, callee, argument)?;
                }
                ControlCallMode::DirectRegion(_)
                    if self.execution.control_frames.frame(site).is_some() =>
                {
                    self.emit_frame_call(site, callee, argument)?;
                }
                ControlCallMode::Dispatch => {
                    let Terminator::Call { callee, .. } = terminator else {
                        unreachable!()
                    };
                    let result = self.emit_indirect_call(site, callee, argument, false)?;
                    self.store_input_pattern(*resume, Some(&result))?;
                    self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
                    self.line(format!("  br label %mal_state_{}", resume.0));
                }
                ControlCallMode::DirectSelfTail => return None,
                ControlCallMode::DirectRegion(_) => return None,
            },
            Terminator::TailCall { callee, argument } => {
                match self.execution.control_calls.mode(site)? {
                    ControlCallMode::DirectSelfTail => {
                        let argument = self
                            .execution
                            .control_calls
                            .forwarded_self_argument(site)
                            .unwrap_or(argument);
                        let effect = self
                            .ownership
                            .terminator_use(
                                site,
                                crate::execution::ownership::TerminatorOperand::TailArgument,
                            )
                            .or_else(|| {
                                (!crate::execution::ownership::is_managed(&argument.ty))
                                    .then_some(crate::execution::ownership::UseEffect::Borrow)
                            })?;
                        let value = self.prepare_atom_for_use(argument, effect)?;
                        let function = self.current_function()?.clone();
                        if function.parameter.ty != value.value.ty {
                            return None;
                        }
                        self.commit_consumes(&value)?;
                        self.emit_parameter_handoff(
                            function.id,
                            &value.value,
                            crate::execution::ownership::ParameterEntry::OwnedHandoff,
                        )?;
                        self.emit_edge_drops(
                            site,
                            crate::execution::ownership::ControlPath::Single,
                        )?;
                        self.line(format!("  br label %mal_state_{}", function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(site, target, callee, argument, true)?;
                        self.emit_edge_drops(
                            site,
                            crate::execution::ownership::ControlPath::Single,
                        )?;
                        self.emit_continuation_return(site, &result)?;
                    }
                    ControlCallMode::DirectRegion(_) => {
                        self.emit_region_transition(site, callee, argument, false, &[])?;
                    }
                    ControlCallMode::Dispatch => {
                        if self.common_region.is_some()
                            && self.execution.control_regions.site_region(site)
                                == self.common_region
                        {
                            self.emit_region_transition(site, callee, argument, false, &[])?;
                        } else {
                            let result = self.emit_indirect_call(site, callee, argument, true)?;
                            self.emit_edge_drops(
                                site,
                                crate::execution::ownership::ControlPath::Single,
                            )?;
                            self.emit_continuation_return(site, &result)?;
                        }
                    }
                }
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
}
