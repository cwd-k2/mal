use crate::backend::llvm::syntax::{BinaryOperator, ComparisonKind, ComparisonPredicate};
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
                emit_instruction!(
                    self;
                    call None,
                    false,
                    (void),
                    direct "mal_runtime_environment_release";
                    [
                        (typed (ptr) => { environment }),
                    ]
                );
            }
            if result.ty != self.result_type {
                return None;
            }
            let result_type = self.types.value(&self.result_type)?;
            emit_terminator!(self; return { result_type.llvm } => { result.representation.as_str() });
            return Some(());
        }
        let top = self.register();
        emit_instruction!(
            self;
            load { top.clone() },
            { self.types.index_llvm_type() },
            { self.control_top_pointer() },
            { self.types.index_alignment() },
            []
        );
        let finished = self.register();
        emit_instruction!(
            self;
            compare { finished.clone() },
            { ComparisonKind::Integer },
            { ComparisonPredicate::Eq },
            { self.types.index_llvm_type() },
            { top.clone() },
            "%mal_control_base"
        );
        emit_terminator!(self; conditional
            { finished } =>
            { format!("mal_return_done_{}", site.0) },
            { format!("mal_return_pop_{}", site.0) }
        );
        self.block(format!("mal_return_done_{}", site.0));
        self.sync_control_top()?;
        if self.common_region.is_some() {
            let environment = self.active_environment();
            emit_instruction!(
                self;
                call None,
                false,
                (void),
                direct "mal_runtime_environment_release";
                [
                    (typed (ptr) => { environment }),
                ]
            );
        }
        if result.ty == self.result_type {
            let result_type = self.types.value(&self.result_type)?;
            emit_terminator!(self; return { result_type.llvm } => { result.representation.as_str() });
        } else {
            emit_terminator!(self; unreachable);
        }
        self.block(format!("mal_return_pop_{}", site.0));
        let storage = self.current_control_storage();
        if let [frame_site] = frame_sites.as_slice() {
            let frame = self.execution.control_frames.frame(*frame_site)?.clone();
            let pass_through = self.physical_frame_pass_through(&frame);
            let layout = FrameLayout::new(&frame, self.types.clone(), false, &pass_through)?;
            let previous_top = self.register();
            emit_instruction!(
                self;
                binary { previous_top.clone() },
                { BinaryOperator::Sub },
                { self.types.index_llvm_type() },
                { top.clone() },
                { layout.size.to_string() }
            );
            emit_instruction!(
                self;
                store { self.types.index_llvm_type() },
                { previous_top.as_str() },
                { self.control_top_pointer() },
                { self.types.index_alignment() },
                []
            );
            let frame_pointer = self.register();
            emit_instruction!(
                self;
                get_element_ptr { frame_pointer.clone() },
                false,
                (int(8_u16)),
                { storage };
                [
                    (typed { self.types.index_llvm_type() } => { previous_top }),
                ]
            );
            if self.common_region.is_some() {
                let active = self.active_environment();
                emit_instruction!(
                    self;
                    call None,
                    false,
                    (void),
                    direct "mal_runtime_environment_release";
                    [
                        (typed (ptr) => { active }),
                    ]
                );
            }
            emit_terminator!(self; branch { format!("mal_frame_{}_from_{}", frame_site.0, site.0) });
            return self.emit_frame_resume(site, *frame_site, result, &frame_pointer, false);
        }
        let footer_offset = self.register();
        emit_instruction!(
            self;
            binary { footer_offset.clone() },
            { BinaryOperator::Sub },
            { self.types.index_llvm_type() },
            { top },
            { self.types.index_size().to_string() }
        );
        let footer = self.register();
        emit_instruction!(
            self;
            get_element_ptr { footer.clone() },
            false,
            (int(8_u16)),
            { storage.clone() };
            [
                (typed { self.types.index_llvm_type() } => { footer_offset }),
            ]
        );
        let previous_top = self.register();
        emit_instruction!(
            self;
            load { previous_top.clone() },
            { self.types.index_llvm_type() },
            { footer },
            { self.types.index_alignment() },
            []
        );
        emit_instruction!(
            self;
            store { self.types.index_llvm_type() },
            { previous_top.as_str() },
            { self.control_top_pointer() },
            { self.types.index_alignment() },
            []
        );
        let frame_pointer = self.register();
        emit_instruction!(
            self;
            get_element_ptr { frame_pointer.clone() },
            false,
            (int(8_u16)),
            { storage };
            [
                (typed { self.types.index_llvm_type() } => { previous_top }),
            ]
        );
        if self.common_region.is_some() {
            let active = self.active_environment();
            emit_instruction!(
                self;
                call None,
                false,
                (void),
                direct "mal_runtime_environment_release";
                [
                    (typed (ptr) => { active }),
                ]
            );
        }
        let tag = self.register();
        emit_instruction!(
            self;
            load { tag.clone() },
            (int(32_u16)),
            { frame_pointer.as_str() },
            4,
            []
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
        emit_terminator!(self; switch
            (int(32_u16)) => { tag };
            default { format!("mal_invalid_frame_{}", site.0) };
            [{{ cases }}]
        );
        self.block(format!("mal_invalid_frame_{}", site.0));
        emit_terminator!(self; unreachable);
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
                emit_terminator!(self; unreachable);
                return Some(());
            }
        }
        for layout in &layout.fields {
            let field = frame.fields.get(layout.index)?;
            let pointer = self.register();
            emit_instruction!(
                self;
                get_element_ptr { pointer.clone() },
                false,
                (int(8_u16)),
                { frame_pointer };
                [
                    (typed (int(64_u16)) => { layout.offset.to_string() }),
                ]
            );
            let value = self.register();
            emit_instruction!(
                self;
                load { value.clone() },
                { layout.value_type.llvm.clone() },
                { pointer },
                { layout.value_type.alignment },
                []
            );
            let slot = self.slots.get(&field.id)?.clone();
            emit_instruction!(
                self;
                store { layout.value_type.llvm.clone() },
                { value },
                { format!("%mal_slot_{}", slot.index) },
                { layout.value_type.alignment },
                []
            );
        }
        if self.common_region.is_some() {
            let environment = if let Some(offset) = layout.environment {
                let pointer = self.register();
                emit_instruction!(
                    self;
                    get_element_ptr { pointer.clone() },
                    false,
                    (int(8_u16)),
                    { frame_pointer };
                    [
                        (typed (int(64_u16)) => { offset.to_string() }),
                    ]
                );
                let environment = self.register();
                emit_instruction!(
                    self;
                    load { environment.clone() },
                    (ptr),
                    { pointer },
                    { self.types.pointer_alignment() },
                    []
                );
                environment
            } else {
                "null".into()
            };
            emit_instruction!(
                self;
                store (ptr),
                { environment },
                "%mal_active_environment",
                { self.types.pointer_alignment() },
                []
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
        emit_terminator!(self; branch { format!("mal_state_{}", frame.resume.0) });
        Some(())
    }
}
