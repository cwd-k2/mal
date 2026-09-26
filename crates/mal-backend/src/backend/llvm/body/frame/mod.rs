use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::StateId;
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter};

mod layout;
mod native;
mod resume;

pub(super) use layout::FrameLayout;

pub(super) fn physical_frame_pass_through(
    frame: &crate::execution::ControlFrame,
    ownership: &crate::execution::OwnershipPlan,
) -> std::collections::HashSet<crate::anf::ast::ValueId> {
    frame
        .fields
        .iter()
        .filter(|field| {
            frame.pass_through.contains(&field.id)
                && (!crate::execution::ownership::is_managed(&field.ty)
                    || ownership.binding_is_borrowed(field.id))
        })
        .map(|field| field.id)
        .collect()
}

impl FunctionEmitter<'_> {
    pub(super) fn emit_frame_call(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
    ) -> Option<()> {
        let frame = self.execution.control_frames.frame(site)?.clone();
        let tagged = self.frame_sites.len() != 1;
        let pass_through = self.physical_frame_pass_through(&frame);
        let layout = FrameLayout::new(&frame, self.types.clone(), tagged, &pass_through)?;
        let replacement = self
            .execution
            .control_frames
            .replacement(site)
            .and_then(|retired| {
                let retired = self.execution.control_frames.frame(retired)?;
                let pass_through = self.physical_frame_pass_through(retired);
                FrameLayout::new(retired, self.types.clone(), tagged, &pass_through)
            })
            .is_some_and(|retired| layout.size <= retired.size);
        let reservation = self.reserve_control_frame(layout.size, replacement)?;
        let frame_pointer = self.register();
        self.get_element_ptr(
            frame_pointer.clone(),
            false,
            crate::backend::llvm::syntax::llvm_type!(int(8_u16)),
            reservation.storage,
            [(self.types.index_llvm_type(), reservation.top.clone())],
        );
        if tagged {
            let tag = self.frame_tags.get(&site)?;
            self.store(
                crate::backend::llvm::syntax::llvm_type!(int(32_u16)),
                tag.to_string(),
                frame_pointer.as_str(),
                4,
                [],
            );
        }
        let prepared_fields = frame
            .fields
            .iter()
            .enumerate()
            .map(|(field_index, field)| {
                let effect = if self.ownership.binding_is_borrowed(field.id) {
                    crate::execution::ownership::UseEffect::Borrow
                } else if crate::execution::ownership::is_managed(&field.ty) {
                    self.ownership.frame_field_use(site, field_index)?
                } else {
                    crate::execution::ownership::UseEffect::Borrow
                };
                self.prepare_binding_for_use(field.id, effect)
            })
            .collect::<Option<Vec<_>>>()?;
        for layout in &layout.fields {
            let value = prepared_fields.get(layout.index)?;
            let pointer = self.register();
            self.get_element_ptr(
                pointer.clone(),
                false,
                crate::backend::llvm::syntax::llvm_type!(int(8_u16)),
                frame_pointer.as_str(),
                [(
                    crate::backend::llvm::syntax::llvm_type!(int(64_u16)),
                    layout.offset.to_string(),
                )],
            );
            self.store(
                layout.value_type.llvm.clone(),
                value.value.representation.as_str(),
                pointer,
                layout.value_type.alignment,
                [],
            );
        }
        if let Some(offset) = layout.environment {
            let environment = self.active_environment();
            let pointer = self.register();
            self.get_element_ptr(
                pointer.clone(),
                false,
                crate::backend::llvm::syntax::llvm_type!(int(8_u16)),
                frame_pointer.as_str(),
                [(
                    crate::backend::llvm::syntax::llvm_type!(int(64_u16)),
                    offset.to_string(),
                )],
            );
            self.store(
                crate::backend::llvm::syntax::llvm_type!(ptr),
                environment,
                pointer,
                self.types.pointer_alignment(),
                [],
            );
        }
        if let Some(offset) = layout.footer {
            let footer = self.register();
            self.get_element_ptr(
                footer.clone(),
                false,
                crate::backend::llvm::syntax::llvm_type!(int(8_u16)),
                frame_pointer,
                [(
                    crate::backend::llvm::syntax::llvm_type!(int(64_u16)),
                    offset.to_string(),
                )],
            );
            self.store(
                self.types.index_llvm_type(),
                reservation.top.as_str(),
                footer,
                self.types.index_alignment(),
                [],
            );
        }
        self.store(
            self.types.index_llvm_type(),
            reservation.next_top.as_str(),
            self.control_top_pointer(),
            self.types.index_alignment(),
            [],
        );
        if self.common_region.is_some() {
            self.emit_region_transition(
                site,
                callee,
                argument,
                frame.carries_environment,
                &prepared_fields,
            )
        } else {
            let effect = self.ownership.terminator_operand_use(
                site,
                crate::execution::ownership::TerminatorOperand::CallArgument,
                argument,
            )?;
            let argument = self.prepare_atom_for_use(argument, effect)?;
            if argument.value.ty != self.function.parameter.ty {
                return None;
            }
            for field in &prepared_fields {
                self.commit_consumes(field)?;
            }
            self.commit_consumes(&argument)?;
            self.emit_parameter_handoff(
                self.function.id,
                &argument.value,
                crate::execution::ownership::ParameterEntry::OwnedHandoff,
            )?;
            self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
            self.branch(format!("mal_state_{}", self.function.entry.0));
            Some(())
        }
    }

    fn physical_frame_pass_through(
        &self,
        frame: &crate::execution::ControlFrame,
    ) -> std::collections::HashSet<crate::anf::ast::ValueId> {
        physical_frame_pass_through(frame, self.ownership)
    }

    pub(super) fn emit_region_transition(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
        preserve_environment: bool,
        pending: &[super::PreparedValue],
    ) -> Option<()> {
        let callee_effect = self.ownership.terminator_operand_use(
            site,
            match &self.control.states[site.0].terminator {
                crate::control::ast::Terminator::Call { .. } => {
                    crate::execution::ownership::TerminatorOperand::CallCallee
                }
                crate::control::ast::Terminator::TailCall { .. } => {
                    crate::execution::ownership::TerminatorOperand::TailCallee
                }
                _ => return None,
            },
            callee,
        )?;
        let callee = self.prepare_atom_for_use(callee, callee_effect)?;
        let Type::Function { parameter, result } = &callee.value.ty else {
            return None;
        };
        let parameter = parameter.clone();
        let result = result.clone();
        let direct_target = match self.execution.control_calls.mode(site)? {
            crate::execution::ControlCallMode::DirectRegion(target) => Some(target),
            crate::execution::ControlCallMode::Dispatch => None,
            crate::execution::ControlCallMode::Direct(_)
            | crate::execution::ControlCallMode::DirectSelfTail => return None,
        };
        let code = if direct_target.is_none() {
            let closure_type = self.types.value(&callee.value.ty)?;
            let code = self.register();
            self.extract_value(
                code.clone(),
                closure_type.llvm,
                callee.value.representation.clone(),
                [0],
            );
            Some(code)
        } else {
            None
        };
        let environment = self.closure_environment(&callee.value)?;
        let argument_operand = match &self.control.states[site.0].terminator {
            crate::control::ast::Terminator::Call { .. } => {
                crate::execution::ownership::TerminatorOperand::CallArgument
            }
            crate::control::ast::Terminator::TailCall { .. } => {
                crate::execution::ownership::TerminatorOperand::TailArgument
            }
            _ => return None,
        };
        let argument_effect =
            self.ownership
                .terminator_operand_use(site, argument_operand, argument)?;
        let argument = self.prepare_atom_for_use(argument, argument_effect)?;
        if argument.value.ty != *parameter {
            return None;
        }
        for value in pending {
            self.commit_consumes(value)?;
        }
        self.commit_consumes(&callee)?;
        self.commit_consumes(&argument)?;
        self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
        if !preserve_environment {
            let previous = self.active_environment();
            self.direct_call(
                None,
                false,
                crate::backend::llvm::syntax::llvm_type!(void),
                "mal_runtime_environment_release",
                [(crate::backend::llvm::syntax::llvm_type!(ptr), previous)],
            );
        }
        self.store(
            crate::backend::llvm::syntax::llvm_type!(ptr),
            environment.as_str(),
            "%mal_active_environment",
            self.types.pointer_alignment(),
            [],
        );
        let targets = self
            .execution
            .control_regions
            .recursive_targets(site)?
            .to_vec();
        if let Some(target) = direct_target {
            if !targets.contains(&target) {
                return None;
            }
            return self.emit_region_target(target, &argument.value);
        }
        self.emit_region_dispatch(
            site,
            &targets,
            code.as_deref()?,
            &environment,
            &argument.value,
            &result,
        )
    }

    fn emit_region_dispatch(
        &mut self,
        site: StateId,
        targets: &[FunctionId],
        code: &str,
        environment: &str,
        argument: &EmittedValue,
        result: &Type,
    ) -> Option<()> {
        for (index, target) in targets.iter().enumerate() {
            let matched = self.register();
            self.compare(
                matched.clone(),
                crate::backend::llvm::syntax::ComparisonKind::Integer,
                crate::backend::llvm::syntax::ComparisonPredicate::Eq,
                crate::backend::llvm::syntax::llvm_type!(ptr),
                code,
                format!("@{}", super::function_name(*target)),
            );
            let next = format!("mal_region_dispatch_{}_{}", site.0, index);
            self.conditional_branch(
                matched,
                format!("mal_region_target_{}_{index}", site.0),
                next.clone(),
            );
            self.block(next);
        }
        let has_native_target = self
            .execution
            .applications
            .targets(site)?
            .iter()
            .any(|target| !targets.contains(target));
        if has_native_target {
            let result_type = self.types.value(result)?;
            let mut arguments = vec![
                (
                    crate::backend::llvm::syntax::llvm_type!(ptr),
                    "%mal_context".into(),
                ),
                (
                    crate::backend::llvm::syntax::llvm_type!(ptr),
                    "%mal_control_top".into(),
                ),
                (
                    crate::backend::llvm::syntax::llvm_type!(ptr),
                    environment.into(),
                ),
            ];
            if argument.ty != Type::Unit {
                let argument_type = self.types.value(&argument.ty)?;
                arguments.push((argument_type.llvm, argument.representation.clone()));
            }
            let returned = self.register();
            self.sync_control_top()?;
            self.indirect_call(
                Some(returned.clone()),
                false,
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
            if argument.owned {
                self.release_value(&argument.ty, &argument.representation)?;
            }
            self.emit_continuation_return(
                site,
                &EmittedValue {
                    ty: result.clone(),
                    representation: returned,
                    owned: crate::execution::ownership::is_managed(result),
                },
            )?;
        } else {
            self.unreachable();
        }
        for (index, target) in targets.iter().enumerate() {
            self.block(format!("mal_region_target_{}_{index}", site.0));
            self.emit_region_target(*target, argument)?;
        }
        Some(())
    }

    fn emit_region_target(&mut self, target: FunctionId, argument: &EmittedValue) -> Option<()> {
        let function = *self.index.control_functions.get(&target)?;
        let entry = function.entry;
        self.emit_parameter_handoff(
            target,
            argument,
            crate::execution::ownership::ParameterEntry::OwnedHandoff,
        )?;
        self.branch(format!("mal_state_{}", entry.0));
        Some(())
    }
}
