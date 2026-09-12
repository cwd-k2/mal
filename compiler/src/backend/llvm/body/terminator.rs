use super::*;
impl FunctionEmitter<'_> {
    pub(super) fn emit_state(&mut self, site: StateId) -> Option<()> {
        self.current_function = self.function_for_state(site)?;
        let state = &self.control.states[site.0];
        self.line(format!("mal_state_{}:", site.0));
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let value = self.emit_operation(
                &binding.operation,
                pattern_value_type(&binding.pattern),
                self.optimizations.symbol_concat_mode(site, binding_index),
            )?;
            self.store_pattern(&binding.pattern, value.as_ref())?;
            let mut dead = self.ownership.dead_values(site, binding_index).to_vec();
            dead.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
            for id in dead {
                self.release_dead_slot(id)?;
            }
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let mut value = self.atom(value)?;
                self.retain_if_borrowed(&mut value)?;
                let result_type = self.current_result_type()?;
                if value.ty != result_type {
                    return None;
                }
                self.emit_continuation_return(site, &value)?;
            }
            Terminator::Goto(target) => {
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::Jump { target, value } => {
                let value = self.atom(value)?;
                let input = self.control.states[target.0].input.as_ref()?;
                self.store_pattern(input, Some(&value))?;
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty != right.ty {
                    return None;
                }
                let condition = self.register();
                if left.ty == Type::Symbol {
                    let equality = self.register();
                    self.line(format!(
                        "  {equality} = call i8 @mal_runtime_symbol_equal(ptr {}, ptr {})",
                        left.representation, right.representation
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
                    let scalar = scalar_type(&left.ty)?;
                    let predicate = predicate.for_scalar(scalar);
                    let instruction = if scalar.floating { "fcmp" } else { "icmp" };
                    self.line(format!(
                        "  {condition} = {instruction} {predicate} {} {}, {}",
                        scalar.llvm, left.representation, right.representation
                    ));
                }
                self.line(format!(
                    "  br i1 {condition}, label %mal_state_{}, label %mal_state_{}",
                    then.0, otherwise.0
                ));
            }
            Terminator::Call {
                callee,
                argument,
                resume,
            } => match self.execution.control_calls.mode(site)? {
                ControlCallMode::Direct(target) => {
                    let result = self.emit_call(target, callee, argument, false)?;
                    let input = self.control.states[resume.0].input.as_ref()?;
                    self.store_pattern(input, Some(&result))?;
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
                    let result = self.emit_indirect_call(callee, argument, false)?;
                    let input = self.control.states[resume.0].input.as_ref()?;
                    self.store_pattern(input, Some(&result))?;
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
                        let mut value = self.atom(argument)?;
                        let function = self.current_function()?.clone();
                        if function.parameter.ty != value.ty {
                            return None;
                        }
                        self.retain_if_borrowed(&mut value)?;
                        self.release_local_managed();
                        self.emit_parameter_handoff(function.id, &value)?;
                        self.line(format!("  br label %mal_state_{}", function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, callee, argument, true)?;
                        self.emit_continuation_return(site, &result)?;
                    }
                    ControlCallMode::DirectRegion(_) => {
                        self.emit_region_transition(site, callee, argument, false)?;
                    }
                    ControlCallMode::Dispatch => {
                        if self.common_region.is_some()
                            && self.execution.control_regions.site_region(site)
                                == self.common_region
                        {
                            self.emit_region_transition(site, callee, argument, false)?;
                        } else {
                            let result = self.emit_indirect_call(callee, argument, true)?;
                            self.emit_continuation_return(site, &result)?;
                        }
                    }
                }
            }
            Terminator::Case { scrutinee, arms } => self.emit_case(site, scrutinee, arms)?,
        }
        Some(())
    }
}
