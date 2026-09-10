use crate::anf::ast::ValueId;
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
            self.types.pointer_size()
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
            let tag = self
                .frame_sites
                .iter()
                .position(|candidate| *candidate == site)
                .and_then(|tag| u32::try_from(tag).ok())?;
            self.line(format!("  store i32 {tag}, ptr {frame_pointer}, align 4"));
        }
        for (field, layout) in frame.fields.iter().zip(&layout.fields) {
            let mut value = self.load_binding(field.id)?;
            self.retain_if_borrowed(&mut value)?;
            let pointer = self.register();
            self.line(format!(
                "  {pointer} = getelementptr i8, ptr {frame_pointer}, i64 {}",
                layout.offset
            ));
            self.line(format!(
                "  store {} {}, ptr {pointer}, align {}",
                layout.value_type.llvm, value.representation, layout.value_type.alignment
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
                self.types.pointer_size()
            ));
        }
        if let Some(offset) = layout.footer {
            let footer = self.register();
            self.line(format!(
                "  {footer} = getelementptr i8, ptr {frame_pointer}, i64 {offset}"
            ));
            self.line(format!(
                "  store {index_type} {top}, ptr {footer}, align {}",
                self.types.pointer_size()
            ));
        }
        self.line(format!(
            "  store {index_type} {next_top}, ptr %mal_control_top, align {}",
            self.types.pointer_size()
        ));
        if self.common_region.is_some() {
            self.emit_region_transition(site, callee, argument, frame.carries_environment)
        } else {
            let mut argument = self.atom(argument)?;
            if argument.ty != self.function.parameter.ty {
                return None;
            }
            self.retain_if_borrowed(&mut argument)?;
            self.release_local_managed();
            if let Some(parameter) = self.function.parameter.binding {
                let slot = self.slots.get(&parameter)?.clone();
                let value_type = self.types.value(&slot.ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, argument.representation, slot.index, value_type.alignment
                ));
            } else if self.function.parameter.ty != Type::Unit {
                return None;
            }
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
    ) -> Option<()> {
        let callee = self.atom(callee)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        let closure_type = self.types.value(&callee.ty)?;
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
                closure_type.llvm, callee.representation
            ));
            Some(code)
        } else {
            None
        };
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.representation
        ));
        let retained_environment = self.register();
        self.line(format!(
            "  {retained_environment} = call ptr @mal_runtime_environment_retain(ptr %mal_context, ptr {environment})"
        ));
        let mut argument = self.atom(argument)?;
        if argument.ty != **parameter {
            return None;
        }
        self.retain_if_borrowed(&mut argument)?;
        self.release_local_managed();
        if !preserve_environment {
            let previous = self.active_environment();
            self.line(format!(
                "  call void @mal_runtime_environment_release(ptr {previous})"
            ));
        }
        self.line(format!(
            "  store ptr {retained_environment}, ptr %mal_active_environment, align {}",
            self.types.pointer_size()
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
            return self.emit_region_target(target, &argument);
        }
        self.emit_region_dispatch(
            site,
            &targets,
            code.as_deref()?,
            &retained_environment,
            &argument,
            result,
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
            self.emit_frame_return(
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
        let function = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        let entry = function.entry;
        let parameter = function.parameter.clone();
        if let Some(binding) = parameter.binding {
            let slot = self.slots.get(&binding)?.clone();
            if slot.ty != argument.ty {
                return None;
            }
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  store {} {}, ptr %mal_slot_{}, align {}",
                value_type.llvm, argument.representation, slot.index, value_type.alignment
            ));
        } else if parameter.ty != Type::Unit {
            return None;
        }
        self.line(format!("  br label %mal_state_{}", entry.0));
        Some(())
    }

    pub(super) fn emit_frame_return(&mut self, site: StateId, result: &EmittedValue) -> Option<()> {
        let frame_sites = self.frame_sites.clone();
        let index_type = self.types.pointer_integer()?;
        self.release_local_managed();
        let top = self.register();
        self.line(format!(
            "  {top} = load {index_type}, ptr %mal_control_top, align {}",
            self.types.pointer_size()
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
                self.types.pointer_size()
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
            self.types.pointer_size()
        ));
        let footer = self.register();
        self.line(format!(
            "  {footer} = getelementptr i8, ptr {storage}, {index_type} {footer_offset}"
        ));
        let previous_top = self.register();
        self.line(format!(
            "  {previous_top} = load {index_type}, ptr {footer}, align {}",
            self.types.pointer_size()
        ));
        self.line(format!(
            "  store {index_type} {previous_top}, ptr %mal_control_top, align {}",
            self.types.pointer_size()
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
                    self.types.pointer_size()
                ));
                environment
            } else {
                "null".into()
            };
            self.line(format!(
                "  store ptr {environment}, ptr %mal_active_environment, align {}",
                self.types.pointer_size()
            ));
        }
        let input = self.control.states[frame.resume.0].input.as_ref()?;
        self.store_pattern(
            input,
            Some(&EmittedValue {
                ty: result.ty.clone(),
                representation: result.representation.clone(),
                owned: true,
            }),
        )?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }

    fn load_binding(&mut self, id: ValueId) -> Option<EmittedValue> {
        let slot = self.slots.get(&id)?.clone();
        let value_type = self.types.value(&slot.ty)?;
        let register = self.register();
        self.line(format!(
            "  {register} = load {}, ptr %mal_slot_{}, align {}",
            value_type.llvm, slot.index, value_type.alignment
        ));
        Some(EmittedValue {
            ty: slot.ty,
            representation: register,
            owned: false,
        })
    }
}
