mod call;
mod local;
mod ownership;
mod support;

use crate::c_emit::syntax::{AggregateDefinition, AggregateField, TranslationUnit, TypeName};
use crate::control::ast::StateId;

use super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(super) fn emit_control_frames(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for index in 0..self.control.states.len() {
            let site = StateId(index);
            let Some(frame) = self.control_frames.frame(site) else {
                continue;
            };
            let mut fields = vec![AggregateField::variable("MalControlFrameHeader", "header")];
            fields.extend(frame.fields.iter().enumerate().map(|(index, field)| {
                AggregateField::variable(
                    self.types.c_type(&field.value.ty),
                    frame_field_name(index),
                )
            }));
            if frame.needs_environment {
                fields.push(AggregateField::variable(
                    TypeName::const_named("void").pointer(),
                    "environment",
                ));
            }
            output.push(AggregateDefinition::typedef_structure(
                None,
                fields,
                frame_name(site),
            ));
            output.blank_line();
        }
        output
    }
}

pub(super) fn frame_name(site: StateId) -> String {
    format!("MalControlFrame_{}", site.0)
}

pub(super) fn frame_field_name(index: usize) -> String {
    format!("field_{index}")
}
