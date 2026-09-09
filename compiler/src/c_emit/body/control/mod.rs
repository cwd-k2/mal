mod call;
mod common;
mod local;
mod ownership;
mod support;

use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Parameter, TranslationUnit, TypeName,
};
use crate::control::ast::StateId;

use super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(super) fn emit_control_frames(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for index in 0..self.control.states.len() {
            let site = StateId(index);
            if self.control_regions.site_region(site).is_none() {
                continue;
            }
            let Some(frame) = self.control_frames.frame(site) else {
                continue;
            };
            let mut fields = Vec::new();
            if !self.control_frames.frame_is_homogeneous(site) {
                fields.push(AggregateField::variable("MalControlFrameHeader", "header"));
            }
            fields.extend(frame.fields.iter().enumerate().map(|(index, field)| {
                AggregateField::variable(
                    self.types.c_type(&field.value.ty),
                    frame_field_name(index),
                )
            }));
            if frame.carries_environment {
                fields.push(AggregateField::variable(
                    TypeName::const_named("void").pointer(),
                    "environment",
                ));
                fields.push(AggregateField::function_pointer(
                    "void",
                    "destroy_environment",
                    [
                        Parameter::unnamed(TypeName::named("MalContext").pointer()),
                        Parameter::unnamed(TypeName::const_named("void").pointer()),
                    ],
                ));
            }
            if fields.is_empty() {
                fields.push(AggregateField::variable("uint8_t", "unused"));
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
