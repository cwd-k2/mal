//! Resumption of a suspended frame with the value returned to it.

use crate::control::ast::StateId;

use super::layout::FrameLayout;
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_frame_resume(
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
            self.initialize_slot(
                &slot,
                &super::super::EmittedValue {
                    ty: field.ty.clone(),
                    representation: value,
                    owned: crate::execution::ownership::is_managed(&field.ty),
                },
            )?;
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
