use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::StateId;
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter};

mod layout;
mod resume;

use layout::FrameLayout;

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
        let index_type = self.types.pointer_integer()?;
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {}, {index_type} {}",
            reservation.storage, reservation.top
        ));
        if tagged {
            let tag = self.frame_tags.get(&site)?;
            self.line(format!("  store i32 {tag}, ptr {frame_pointer}, align 4"));
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
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                layout.offset
            ));
            self.line(format!(
                "  store {} {}, ptr {pointer}, align {}",
                layout.value_type.llvm, value.value.representation, layout.value_type.alignment
            ));
        }
        if let Some(offset) = layout.environment {
            let environment = self.active_environment();
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {offset}"
            ));
            self.line(format!(
                "  store ptr {environment}, ptr {pointer}, align {}",
                self.types.pointer_alignment()
            ));
        }
        if let Some(offset) = layout.footer {
            let footer = self.register();
            self.line(format!(
                "  {footer} = getelementptr i8, ptr {frame_pointer}, i64 {offset}"
            ));
            self.line(format!(
                "  store {index_type} {}, ptr {footer}, align {}",
                reservation.top,
                self.types.index_alignment()
            ));
        }
        self.line(format!(
            "  store {index_type} {}, ptr {}, align {}",
            reservation.next_top,
            self.control_top_pointer(),
            self.types.index_alignment()
        ));
        if self.common_region.is_some() {
            self.emit_region_transition(
                site,
                callee,
                argument,
                frame.carries_environment,
                &prepared_fields,
            )
        } else {
            let effect = self
                .ownership
                .terminator_use(
                    site,
                    crate::execution::ownership::TerminatorOperand::CallArgument,
                )
                .or_else(|| {
                    (!crate::execution::ownership::is_managed(&argument.ty))
                        .then_some(crate::execution::ownership::UseEffect::Borrow)
                })?;
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
            self.line(format!("  br label %mal_state_{}", self.function.entry.0));
            Some(())
        }
    }

    fn physical_frame_pass_through(
        &self,
        frame: &crate::execution::ControlFrame,
    ) -> std::collections::HashSet<crate::anf::ast::ValueId> {
        frame
            .fields
            .iter()
            .filter(|field| {
                frame.pass_through.contains(&field.id)
                    && (!crate::execution::ownership::is_managed(&field.ty)
                        || self.ownership.binding_is_borrowed(field.id))
            })
            .map(|field| field.id)
            .collect()
    }

    pub(super) fn emit_region_transition(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
        preserve_environment: bool,
        pending: &[super::PreparedValue],
    ) -> Option<()> {
        let callee_effect = self
            .ownership
            .terminator_use(
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
            )
            .or_else(|| {
                (!crate::execution::ownership::is_managed(&callee.ty))
                    .then_some(crate::execution::ownership::UseEffect::Borrow)
            })?;
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
            self.line(format!(
                "  {code} = extractvalue {} {}, 0",
                closure_type.llvm, callee.value.representation
            ));
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
        let argument_effect = self
            .ownership
            .terminator_use(site, argument_operand)
            .or_else(|| {
                (!crate::execution::ownership::is_managed(&argument.ty))
                    .then_some(crate::execution::ownership::UseEffect::Borrow)
            })?;
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
            self.line(format!(
                "  call void @mal_runtime_environment_release(ptr {previous})"
            ));
        }
        self.line(format!(
            "  store ptr {environment}, ptr %mal_active_environment, align {}",
            self.types.pointer_alignment()
        ));
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
            self.line(format!(
                "  {matched} = icmp eq ptr {code}, @{}",
                super::function_name(*target)?
            ));
            let next = format!("mal_region_dispatch_{}_{}", site.0, index);
            self.line(format!(
                "  br i1 {matched}, label %mal_region_target_{}_{index}, label %{next}",
                site.0
            ));
            self.line(format!("{next}:"));
        }
        let has_native_target = self
            .execution
            .applications
            .targets(site)?
            .iter()
            .any(|target| !targets.contains(target));
        if has_native_target {
            let result_type = self.types.value(result)?;
            let arguments = if argument.ty == Type::Unit {
                format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
            } else {
                let argument_type = self.types.value(&argument.ty)?;
                format!(
                    "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
                    argument_type.llvm, argument.representation
                )
            };
            let returned = self.register();
            self.sync_control_top()?;
            self.line(format!(
                "  {returned} = call {} {code}({arguments})",
                result_type.llvm
            ));
            if self
                .optimizations
                .site_may_relocate_control_storage(&self.execution.applications, site)
            {
                self.refresh_control_storage()?;
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
            self.line("  unreachable");
        }
        for (index, target) in targets.iter().enumerate() {
            self.line(format!("mal_region_target_{}_{index}:", site.0));
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
        self.line(format!("  br label %mal_state_{}", entry.0));
        Some(())
    }
}
