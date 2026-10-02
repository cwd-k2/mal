use crate::closure::ast::Atom;
use crate::control::ast::StateId;

use super::{EmittedValue, FunctionEmitter};

mod layout;
mod native;
mod region;
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
        emit_instruction! {
            self;
            let #{ frame_pointer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: #{ reservation.storage },
                indices: [typed(#{ self.types.index_llvm_type() }, #{ reservation.top.clone() })],
            };
        };
        if tagged {
            let tag = self.frame_tags.get(&site)?;
            emit_instruction! {
                self;
                store {
                    value: typed((int(32_u16)), #{ tag.to_string() }),
                    pointer: #{ frame_pointer.as_str() },
                    alignment: 4,
                    metadata: [],
                };
            };
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
            emit_instruction! {
                self;
                let #{ pointer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: (int(8_u16)),
                    pointer: #{ frame_pointer.as_str() },
                    indices: [typed((int(64_u16)), #{ layout.offset.to_string() })],
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed(#{ layout.value_type.llvm.clone() }, #{ value.value.representation.as_str() }),
                    pointer: #{ pointer },
                    alignment: #{ layout.value_type.alignment },
                    metadata: [],
                };
            };
        }
        if let Some(offset) = layout.environment {
            let environment = self.active_environment();
            let pointer = self.register();
            emit_instruction! {
                self;
                let #{ pointer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: (int(8_u16)),
                    pointer: #{ frame_pointer.as_str() },
                    indices: [typed((int(64_u16)), #{ offset.to_string() })],
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed((ptr), #{ environment }),
                    pointer: #{ pointer },
                    alignment: #{ self.types.pointer_alignment() },
                    metadata: [],
                };
            };
        }
        if let Some(offset) = layout.footer {
            let footer = self.register();
            emit_instruction! {
                self;
                let #{ footer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: (int(8_u16)),
                    pointer: #{ frame_pointer },
                    indices: [typed((int(64_u16)), #{ offset.to_string() })],
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed(#{ self.types.index_llvm_type() }, #{ reservation.top.as_str() }),
                    pointer: #{ footer },
                    alignment: #{ self.types.index_alignment() },
                    metadata: [],
                };
            };
        }
        emit_instruction! {
            self;
            store {
                value: typed(#{ self.types.index_llvm_type() }, #{ reservation.next_top.as_str() }),
                pointer: #{ self.control_top_pointer() },
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
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
            emit_terminator! {
                self;
                branch {
                    target: #{ format!("mal_state_{}", self.function.entry.0) },
                };
            };
            Some(())
        }
    }

    fn physical_frame_pass_through(
        &self,
        frame: &crate::execution::ControlFrame,
    ) -> std::collections::HashSet<crate::anf::ast::ValueId> {
        physical_frame_pass_through(frame, self.ownership)
    }
}
