use crate::check::ast::Type;
use crate::closure::ast::{Pattern, TopLevelPattern};

use super::{BodyEmitter, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_top_level_globals(&self, output: &mut String, pattern: &TopLevelPattern) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } => {
                c_line!(
                    output,
                    0,
                    "static {} {};",
                    self.types.c_type(ty),
                    value_name(*id)
                );
            }
            TopLevelPattern::Wildcard { .. } => {}
            TopLevelPattern::Product { elements, .. } => {
                for element in elements {
                    self.emit_top_level_globals(output, element);
                }
            }
        }
    }

    pub(super) fn emit_top_level_pattern(
        &self,
        output: &mut String,
        pattern: &TopLevelPattern,
        value: &str,
        indent: usize,
    ) {
        match pattern {
            TopLevelPattern::Binding { id, .. } => {
                c_line!(output, indent, "{} = {value};", value_name(*id));
                c_line!(output, indent, "(void){};", value_name(*id));
            }
            TopLevelPattern::Wildcard { .. } => {}
            TopLevelPattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_top_level_pattern(
                        output,
                        element,
                        &format!("{value}.field_{index}"),
                        indent,
                    );
                }
            }
        }
    }

    pub(super) fn emit_pattern_bindings(
        &self,
        output: &mut String,
        pattern: &Pattern,
        value: &str,
        indent: usize,
    ) {
        match pattern {
            Pattern::Binding { id, ty } => {
                let name = value_name(*id);
                c_line!(
                    output,
                    indent,
                    "{} {name} = {value};",
                    self.types.c_type(ty)
                );
                c_line!(output, indent, "(void){name};");
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(
                        output,
                        element,
                        &format!("{value}.field_{index}"),
                        indent,
                    );
                }
            }
        }
    }
}

pub(super) fn pattern_type(pattern: &Pattern) -> &Type {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => ty,
    }
}
