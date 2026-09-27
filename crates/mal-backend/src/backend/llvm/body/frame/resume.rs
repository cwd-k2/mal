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
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (void),
                        callee: direct("mal_runtime_environment_release"),
                        arguments: [typed((ptr), #{ environment })],
                    };
                };
            }
            if result.ty != self.result_type {
                return None;
            }
            let result_type = self.types.value(&self.result_type)?;
            emit_terminator! {
                self;
                return typed(#{ result_type.llvm }, #{ result.representation.as_str() });
            };
            return Some(());
        }
        let top = self.register();
        emit_instruction! {
            self;
            let #{ top.clone() } = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: #{ self.control_top_pointer() },
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        let finished = self.register();
        emit_instruction! {
            self;
            let #{ finished.clone() } = compare {
                kind: #{ ComparisonKind::Integer },
                predicate: #{ ComparisonPredicate::Eq },
                ty: #{ self.types.index_llvm_type() },
                left: #{ top.clone() },
                right: "%mal_control_base",
            };
        };
        emit_terminator! {
            self;
            branch {
                condition: #{ finished },
                then: #{ format!("mal_return_done_{}", site.0) },
                otherwise: #{ format!("mal_return_pop_{}", site.0) },
            };
        };
        self.block(format!("mal_return_done_{}", site.0));
        self.sync_control_top()?;
        if self.common_region.is_some() {
            let environment = self.active_environment();
            emit_instruction! {
                self;
                call {
                    tail: false,
                    result_type: (void),
                    callee: direct("mal_runtime_environment_release"),
                    arguments: [typed((ptr), #{ environment })],
                };
            };
        }
        if result.ty == self.result_type {
            let result_type = self.types.value(&self.result_type)?;
            emit_terminator! {
                self;
                return typed(#{ result_type.llvm }, #{ result.representation.as_str() });
            };
        } else {
            emit_terminator! {
                self;
                unreachable;
            };
        }
        self.block(format!("mal_return_pop_{}", site.0));
        let storage = self.current_control_storage();
        if let [frame_site] = frame_sites.as_slice() {
            let frame = self.execution.control_frames.frame(*frame_site)?.clone();
            let pass_through = self.physical_frame_pass_through(&frame);
            let layout = FrameLayout::new(&frame, self.types.clone(), false, &pass_through)?;
            let previous_top = self.register();
            emit_instruction! {
                self;
                let #{ previous_top.clone() } = binary {
                    operator: #{ BinaryOperator::Sub },
                    ty: #{ self.types.index_llvm_type() },
                    left: #{ top.clone() },
                    right: #{ layout.size.to_string() },
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed(#{ self.types.index_llvm_type() }, #{ previous_top.as_str() }),
                    pointer: #{ self.control_top_pointer() },
                    alignment: #{ self.types.index_alignment() },
                    metadata: [],
                };
            };
            let frame_pointer = self.register();
            emit_instruction! {
                self;
                let #{ frame_pointer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: (int(8_u16)),
                    pointer: #{ storage },
                    indices: [typed(#{ self.types.index_llvm_type() }, #{ previous_top })],
                };
            };
            if self.common_region.is_some() {
                let active = self.active_environment();
                emit_instruction! {
                    self;
                    call {
                        tail: false,
                        result_type: (void),
                        callee: direct("mal_runtime_environment_release"),
                        arguments: [typed((ptr), #{ active })],
                    };
                };
            }
            emit_terminator! {
                self;
                branch {
                    target: #{ format!("mal_frame_{}_from_{}", frame_site.0, site.0) },
                };
            };
            return self.emit_frame_resume(site, *frame_site, result, &frame_pointer, false);
        }
        let footer_offset = self.register();
        emit_instruction! {
            self;
            let #{ footer_offset.clone() } = binary {
                operator: #{ BinaryOperator::Sub },
                ty: #{ self.types.index_llvm_type() },
                left: #{ top },
                right: #{ self.types.index_size().to_string() },
            };
        };
        let footer = self.register();
        emit_instruction! {
            self;
            let #{ footer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: #{ storage.clone() },
                indices: [typed(#{ self.types.index_llvm_type() }, #{ footer_offset })],
            };
        };
        let previous_top = self.register();
        emit_instruction! {
            self;
            let #{ previous_top.clone() } = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: #{ footer },
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed(#{ self.types.index_llvm_type() }, #{ previous_top.as_str() }),
                pointer: #{ self.control_top_pointer() },
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        let frame_pointer = self.register();
        emit_instruction! {
            self;
            let #{ frame_pointer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: #{ storage },
                indices: [typed(#{ self.types.index_llvm_type() }, #{ previous_top })],
            };
        };
        if self.common_region.is_some() {
            let active = self.active_environment();
            emit_instruction! {
                self;
                call {
                    tail: false,
                    result_type: (void),
                    callee: direct("mal_runtime_environment_release"),
                    arguments: [typed((ptr), #{ active })],
                };
            };
        }
        let tag = self.register();
        emit_instruction! {
            self;
            let #{ tag.clone() } = load {
                ty: (int(32_u16)),
                pointer: #{ frame_pointer.as_str() },
                alignment: 4,
                metadata: [],
            };
        };
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
        emit_terminator! {
            self;
            switch typed((int(32_u16)), #{ tag })  {
                cases: [...#{ cases }],
                default: #{ format!("mal_invalid_frame_{}", site.0) },
            };
        };
        self.block(format!("mal_invalid_frame_{}", site.0));
        emit_terminator! {
            self;
            unreachable;
        };
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
                emit_terminator! {
                    self;
                    unreachable;
                };
                return Some(());
            }
        }
        for layout in &layout.fields {
            let field = frame.fields.get(layout.index)?;
            let pointer = self.register();
            emit_instruction! {
                self;
                let #{ pointer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: (int(8_u16)),
                    pointer: #{ frame_pointer },
                    indices: [typed((int(64_u16)), #{ layout.offset.to_string() })],
                };
            };
            let value = self.register();
            emit_instruction! {
                self;
                let #{ value.clone() } = load {
                    ty: #{ layout.value_type.llvm.clone() },
                    pointer: #{ pointer },
                    alignment: #{ layout.value_type.alignment },
                    metadata: [],
                };
            };
            let slot = self.slots.get(&field.id)?.clone();
            emit_instruction! {
                self;
                store {
                    value: typed(#{ layout.value_type.llvm.clone() }, #{ value }),
                    pointer: #{ format!("%mal_slot_{}", slot.index) },
                    alignment: #{ layout.value_type.alignment },
                    metadata: [],
                };
            };
        }
        if self.common_region.is_some() {
            let environment = if let Some(offset) = layout.environment {
                let pointer = self.register();
                emit_instruction! {
                    self;
                    let #{ pointer.clone() } = get_element_ptr {
                        inbounds: false,
                        element_type: (int(8_u16)),
                        pointer: #{ frame_pointer },
                        indices: [typed((int(64_u16)), #{ offset.to_string() })],
                    };
                };
                let environment = self.register();
                emit_instruction! {
                    self;
                    let #{ environment.clone() } = load {
                        ty: (ptr),
                        pointer: #{ pointer },
                        alignment: #{ self.types.pointer_alignment() },
                        metadata: [],
                    };
                };
                environment
            } else {
                "null".into()
            };
            emit_instruction! {
                self;
                store {
                    value: typed((ptr), #{ environment }),
                    pointer: "%mal_active_environment",
                    alignment: #{ self.types.pointer_alignment() },
                    metadata: [],
                };
            };
        }
        self.store_input_pattern(
            frame.resume,
            Some(&EmittedValue {
                ty: result.ty.clone(),
                representation: result.representation.clone(),
                owned: true,
            }),
        )?;
        emit_terminator! {
            self;
            branch {
                target: #{ format!("mal_state_{}", frame.resume.0) },
            };
        };
        Some(())
    }
}
