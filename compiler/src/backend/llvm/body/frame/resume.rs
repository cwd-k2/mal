use crate::control::ast::StateId;

use super::layout::FrameLayout;
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_continuation_return(
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
        self.store_input_pattern(
            frame.resume,
            Some(&EmittedValue {
                ty: result.ty.clone(),
                representation: result.representation.clone(),
                owned: true,
            }),
        )?;
        self.line(format!("  br label %mal_state_{}", frame.resume.0));
        Some(())
    }
}
