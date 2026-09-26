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
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    "mal_runtime_environment_release",
                    [(crate::backend::llvm::syntax::Type::Pointer, environment)],
                );
            }
            if result.ty != self.result_type {
                return None;
            }
            let result_type = self.types.value(&self.result_type)?;
            self.return_value(result_type.llvm, result.representation.as_str());
            return Some(());
        }
        let top = self.register();
        self.load(
            top.clone(),
            self.types.index_llvm_type(),
            self.control_top_pointer(),
            self.types.index_alignment(),
            [],
        );
        let finished = self.register();
        self.compare(
            finished.clone(),
            crate::backend::llvm::syntax::ComparisonKind::Integer,
            crate::backend::llvm::syntax::ComparisonPredicate::Eq,
            self.types.index_llvm_type(),
            top.clone(),
            "%mal_control_base",
        );
        self.conditional_branch(
            finished,
            format!("mal_return_done_{}", site.0),
            format!("mal_return_pop_{}", site.0),
        );
        self.block(format!("mal_return_done_{}", site.0));
        self.sync_control_top()?;
        if self.common_region.is_some() {
            let environment = self.active_environment();
            self.direct_call(
                None,
                false,
                crate::backend::llvm::syntax::Type::Void,
                "mal_runtime_environment_release",
                [(crate::backend::llvm::syntax::Type::Pointer, environment)],
            );
        }
        if result.ty == self.result_type {
            let result_type = self.types.value(&self.result_type)?;
            self.return_value(result_type.llvm, result.representation.as_str());
        } else {
            self.unreachable();
        }
        self.block(format!("mal_return_pop_{}", site.0));
        let storage = self.current_control_storage();
        if let [frame_site] = frame_sites.as_slice() {
            let frame = self.execution.control_frames.frame(*frame_site)?.clone();
            let pass_through = self.physical_frame_pass_through(&frame);
            let layout = FrameLayout::new(&frame, self.types.clone(), false, &pass_through)?;
            let previous_top = self.register();
            self.binary(
                previous_top.clone(),
                crate::backend::llvm::syntax::BinaryOperator::Sub,
                self.types.index_llvm_type(),
                top.clone(),
                layout.size.to_string(),
            );
            self.store(
                self.types.index_llvm_type(),
                previous_top.as_str(),
                self.control_top_pointer(),
                self.types.index_alignment(),
                [],
            );
            let frame_pointer = self.register();
            self.get_element_ptr(
                frame_pointer.clone(),
                false,
                crate::backend::llvm::syntax::Type::integer(8_u16),
                storage,
                [(self.types.index_llvm_type(), previous_top)],
            );
            if self.common_region.is_some() {
                let active = self.active_environment();
                self.direct_call(
                    None,
                    false,
                    crate::backend::llvm::syntax::Type::Void,
                    "mal_runtime_environment_release",
                    [(crate::backend::llvm::syntax::Type::Pointer, active)],
                );
            }
            self.branch(format!("mal_frame_{}_from_{}", frame_site.0, site.0));
            return self.emit_frame_resume(site, *frame_site, result, &frame_pointer, false);
        }
        let footer_offset = self.register();
        self.binary(
            footer_offset.clone(),
            crate::backend::llvm::syntax::BinaryOperator::Sub,
            self.types.index_llvm_type(),
            top,
            self.types.index_size().to_string(),
        );
        let footer = self.register();
        self.get_element_ptr(
            footer.clone(),
            false,
            crate::backend::llvm::syntax::Type::integer(8_u16),
            storage.clone(),
            [(self.types.index_llvm_type(), footer_offset)],
        );
        let previous_top = self.register();
        self.load(
            previous_top.clone(),
            self.types.index_llvm_type(),
            footer,
            self.types.index_alignment(),
            [],
        );
        self.store(
            self.types.index_llvm_type(),
            previous_top.as_str(),
            self.control_top_pointer(),
            self.types.index_alignment(),
            [],
        );
        let frame_pointer = self.register();
        self.get_element_ptr(
            frame_pointer.clone(),
            false,
            crate::backend::llvm::syntax::Type::integer(8_u16),
            storage,
            [(self.types.index_llvm_type(), previous_top)],
        );
        if self.common_region.is_some() {
            let active = self.active_environment();
            self.direct_call(
                None,
                false,
                crate::backend::llvm::syntax::Type::Void,
                "mal_runtime_environment_release",
                [(crate::backend::llvm::syntax::Type::Pointer, active)],
            );
        }
        let tag = self.register();
        self.load(
            tag.clone(),
            crate::backend::llvm::syntax::Type::integer(32_u16),
            frame_pointer.as_str(),
            4,
            [],
        );
        let cases = frame_sites
            .iter()
            .enumerate()
            .map(|(tag, frame_site)| {
                u32::try_from(tag).ok().map(|tag| {
                    (
                        tag.to_string(),
                        format!("mal_frame_{}_from_{}", frame_site.0, site.0),
                    )
                })
            })
            .collect::<Option<Vec<_>>>()?;
        self.terminate(crate::backend::llvm::syntax::Terminator::switch(
            crate::backend::llvm::syntax::Type::integer(32_u16),
            tag,
            format!("mal_invalid_frame_{}", site.0),
            cases,
        ));
        self.block(format!("mal_invalid_frame_{}", site.0));
        self.unreachable();
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
        let pass_through = self.physical_frame_pass_through(&frame);
        let layout = FrameLayout::new(&frame, self.types.clone(), tagged, &pass_through)?;
        self.block(format!("mal_frame_{}_from_{}", frame_site.0, return_site.0));
        match self
            .execution
            .control_frames
            .resume(return_site, frame_site)?
        {
            crate::execution::FrameResume::Resume => {}
            crate::execution::FrameResume::Unreachable => {
                self.unreachable();
                return Some(());
            }
        }
        for layout in &layout.fields {
            let field = frame.fields.get(layout.index)?;
            let pointer = self.register();
            self.get_element_ptr(
                pointer.clone(),
                false,
                crate::backend::llvm::syntax::Type::integer(8_u16),
                frame_pointer,
                [(
                    crate::backend::llvm::syntax::Type::integer(64_u16),
                    layout.offset.to_string(),
                )],
            );
            let value = self.register();
            self.load(
                value.clone(),
                layout.value_type.llvm.clone(),
                pointer,
                layout.value_type.alignment,
                [],
            );
            let slot = self.slots.get(&field.id)?.clone();
            self.store(
                layout.value_type.llvm.clone(),
                value,
                format!("%mal_slot_{}", slot.index),
                layout.value_type.alignment,
                [],
            );
        }
        if self.common_region.is_some() {
            let environment = if let Some(offset) = layout.environment {
                let pointer = self.register();
                self.get_element_ptr(
                    pointer.clone(),
                    false,
                    crate::backend::llvm::syntax::Type::integer(8_u16),
                    frame_pointer,
                    [(
                        crate::backend::llvm::syntax::Type::integer(64_u16),
                        offset.to_string(),
                    )],
                );
                let environment = self.register();
                self.load(
                    environment.clone(),
                    crate::backend::llvm::syntax::Type::Pointer,
                    pointer,
                    self.types.pointer_alignment(),
                    [],
                );
                environment
            } else {
                "null".into()
            };
            self.store(
                crate::backend::llvm::syntax::Type::Pointer,
                environment,
                "%mal_active_environment",
                self.types.pointer_alignment(),
                [],
            );
        }
        self.store_input_pattern(
            frame.resume,
            Some(&EmittedValue {
                ty: result.ty.clone(),
                representation: result.representation.clone(),
                owned: true,
            }),
        )?;
        self.branch(format!("mal_state_{}", frame.resume.0));
        Some(())
    }
}
