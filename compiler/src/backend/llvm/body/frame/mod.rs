use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::StateId;

use super::{EmittedValue, FunctionEmitter};

mod layout;

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
        let layout = FrameLayout::new(&frame, self.types, tagged)?;
        let index_type = self.types.pointer_integer()?;
        let top = self.register();
        self.line(format!(
            "  {top} = load {index_type}, ptr %mal_control_top, align {}",
            self.types.index_alignment()
        ));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_reserve_frame(ptr %mal_context, {index_type} {top}, {index_type} {})",
            layout.size
        ));
        let next_top = self.register();
        self.line(format!(
            "  {next_top} = add {index_type} {top}, {}",
            layout.size
        ));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, {index_type} {top}"
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
                let effect = if crate::execution::ownership::is_managed(&field.ty) {
                    self.ownership.frame_field_use(site, field_index)?
                } else {
                    crate::execution::ownership::UseEffect::Borrow
                };
                self.prepare_binding_for_use(field.id, effect)
            })
            .collect::<Option<Vec<_>>>()?;
        for (value, layout) in prepared_fields.iter().zip(&layout.fields) {
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
                "  store {index_type} {top}, ptr {footer}, align {}",
                self.types.index_alignment()
            ));
        }
        self.line(format!(
            "  store {index_type} {next_top}, ptr %mal_control_top, align {}",
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
        let closure_type = self.types.value(&callee.value.ty)?;
        let direct_target = match self.execution.control_calls.mode(site)? {
            crate::execution::ControlCallMode::DirectRegion(target) => Some(target),
            crate::execution::ControlCallMode::Dispatch => None,
            crate::execution::ControlCallMode::Direct(_)
            | crate::execution::ControlCallMode::DirectSelfTail => return None,
        };
        let code = if direct_target.is_none() {
            let code = self.register();
            self.line(format!(
                "  {code} = extractvalue {} {}, 0",
                closure_type.llvm, callee.value.representation
            ));
            Some(code)
        } else {
            None
        };
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.value.representation
        ));
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
            self.line(format!(
                "  {returned} = call {} {code}({arguments})",
                result_type.llvm
            ));
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

    pub(super) fn emit_continuation_return(
        &mut self,
        site: StateId,
        result: &EmittedValue,
    ) -> Option<()> {
        let frame_sites = self.frame_sites.clone();
        if frame_sites.is_empty() {
            if self.common_region.is_some() {
                let environment = self.active_environment();
                self.line(format!(
                    "  call void @mal_runtime_environment_release(ptr {environment})"
                ));
            }
            if result.ty != self.result_type {
                return None;
            }
            let result_type = self.types.value(&self.result_type)?;
            self.line(format!(
                "  ret {} {}",
                result_type.llvm, result.representation
            ));
            return Some(());
        }
        let index_type = self.types.pointer_integer()?;
        let top = self.register();
        self.line(format!(
            "  {top} = load {index_type}, ptr %mal_control_top, align {}",
            self.types.index_alignment()
        ));
        let finished = self.register();
        self.line(format!(
            "  {finished} = icmp eq {index_type} {top}, %mal_control_base"
        ));
        self.line(format!(
            "  br i1 {finished}, label %mal_return_done_{}, label %mal_return_pop_{}",
            site.0, site.0
        ));
        self.line(format!("mal_return_done_{}:", site.0));
        if self.common_region.is_some() {
            let environment = self.active_environment();
            self.line(format!(
                "  call void @mal_runtime_environment_release(ptr {environment})"
            ));
        }
        if result.ty == self.result_type {
            let result_type = self.types.value(&self.result_type)?;
            self.line(format!(
                "  ret {} {}",
                result_type.llvm, result.representation
            ));
        } else {
            self.line("  unreachable");
        }
        self.line(format!("mal_return_pop_{}:", site.0));
        let storage = self.register();
        self.line(format!(
            "  {storage} = call ptr @mal_control_storage(ptr %mal_context)"
        ));
        if let [frame_site] = frame_sites.as_slice() {
            let frame = self.execution.control_frames.frame(*frame_site)?.clone();
            let layout = FrameLayout::new(&frame, self.types, false)?;
            let previous_top = self.register();
            self.line(format!(
                "  {previous_top} = sub {index_type} {top}, {}",
                layout.size
            ));
            self.line(format!(
                "  store {index_type} {previous_top}, ptr %mal_control_top, align {}",
                self.types.index_alignment()
            ));
            let frame_pointer = self.register();
            self.line(format!(
                "  {frame_pointer} = getelementptr i8, ptr {storage}, {index_type} {previous_top}"
            ));
            if self.common_region.is_some() {
                let active = self.active_environment();
                self.line(format!(
                    "  call void @mal_runtime_environment_release(ptr {active})"
                ));
            }
            self.line(format!(
                "  br label %mal_frame_{}_from_{}",
                frame_site.0, site.0
            ));
            return self.emit_frame_resume(site, *frame_site, result, &frame_pointer, false);
        }
        let footer_offset = self.register();
        self.line(format!(
            "  {footer_offset} = sub {index_type} {top}, {}",
            self.types.index_size()
        ));
        let footer = self.register();
        self.line(format!(
            "  {footer} = getelementptr i8, ptr {storage}, {index_type} {footer_offset}"
        ));
        let previous_top = self.register();
        self.line(format!(
            "  {previous_top} = load {index_type}, ptr {footer}, align {}",
            self.types.index_alignment()
        ));
        self.line(format!(
            "  store {index_type} {previous_top}, ptr %mal_control_top, align {}",
            self.types.index_alignment()
        ));
        let frame_pointer = self.register();
        self.line(format!(
            "  {frame_pointer} = getelementptr i8, ptr {storage}, {index_type} {previous_top}"
        ));
        if self.common_region.is_some() {
            let active = self.active_environment();
            self.line(format!(
                "  call void @mal_runtime_environment_release(ptr {active})"
            ));
        }
        let tag = self.register();
        self.line(format!("  {tag} = load i32, ptr {frame_pointer}, align 4"));
        let cases = frame_sites
            .iter()
            .enumerate()
            .map(|(tag, frame_site)| {
                u32::try_from(tag).ok().map(|tag| {
                    format!(
                        "    i32 {}, label %mal_frame_{}_from_{}",
                        tag, frame_site.0, site.0
                    )
                })
            })
            .collect::<Option<Vec<_>>>()?
            .join("\n");
        self.line(format!(
            "  switch i32 {tag}, label %mal_invalid_frame_{0} [\n{cases}\n  ]",
            site.0
        ));
        self.line(format!("mal_invalid_frame_{}:", site.0));
        self.line("  unreachable");
        for frame_site in frame_sites {
            self.emit_frame_resume(site, frame_site, result, &frame_pointer, true)?;
        }
        Some(())
    }

    fn emit_frame_resume(
        &mut self,
        return_site: StateId,
        frame_site: StateId,
        result: &EmittedValue,
        frame_pointer: &str,
        tagged: bool,
    ) -> Option<()> {
        let frame = self.execution.control_frames.frame(frame_site)?.clone();
        let layout = FrameLayout::new(&frame, self.types, tagged)?;
        self.line(format!(
            "mal_frame_{}_from_{}:",
            frame_site.0, return_site.0
        ));
        match self
            .execution
            .control_frames
            .resume(return_site, frame_site)?
        {
            crate::execution::FrameResume::Resume => {}
            crate::execution::FrameResume::Unreachable => {
                self.line("  unreachable");
                return Some(());
            }
        }
        let input = self.control.states[frame.resume.0].input.as_ref()?;
        for (field, layout) in frame.fields.iter().zip(&layout.fields) {
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                layout.offset
            ));
            let value = self.register();
            self.line(format!(
                "  {value} = load {}, ptr {pointer}, align {}",
                layout.value_type.llvm, layout.value_type.alignment
            ));
            let slot = self.slots.get(&field.id)?.clone();
            self.line(format!(
                "  store {} {value}, ptr %mal_slot_{}, align {}",
                layout.value_type.llvm, slot.index, layout.value_type.alignment
            ));
        }
        if self.common_region.is_some() {
            let environment = if let Some(offset) = layout.environment {
                let pointer = self.register();
                self.line(format!(
                    "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {offset}"
                ));
                let environment = self.register();
                self.line(format!(
                    "  {environment} = load ptr, ptr {pointer}, align {}",
                    self.types.pointer_alignment()
                ));
                environment
            } else {
                "null".into()
            };
            self.line(format!(
                "  store ptr {environment}, ptr %mal_active_environment, align {}",
                self.types.pointer_alignment()
            ));
        }
        self.store_pattern(
            input,
            Some(&EmittedValue {
                ty: result.ty.clone(),
                representation: result.representation.clone(),
                owned: true,
            }),
        )?;
        self.emit_input_drops(frame.resume)?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }
}
