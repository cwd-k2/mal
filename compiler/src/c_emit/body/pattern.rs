use crate::c_emit::syntax::{Block, Expr, Statement};
use crate::check::ast::Type;
use crate::closure::ast::{Pattern, TopLevelPattern};

use super::{BodyEmitter, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_top_level_globals(&self, output: &mut String, pattern: &TopLevelPattern) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } => {
                Statement::declaration(
                    format!("static {} {}", self.types.c_type(ty), value_name(*id)),
                    None,
                )
                .render(output, 0);
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
        block: &mut Block,
        pattern: &TopLevelPattern,
        value: Expr,
    ) {
        match pattern {
            TopLevelPattern::Binding { id, .. } => {
                block.push(Statement::assignment(
                    Expr::identifier(value_name(*id)),
                    value,
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(value_name(*id)),
                )));
            }
            TopLevelPattern::Wildcard { .. } => {}
            TopLevelPattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_top_level_pattern(
                        block,
                        element,
                        value.clone().field(format!("field_{index}")),
                    );
                }
            }
        }
    }

    pub(super) fn emit_pattern_bindings(&self, block: &mut Block, pattern: &Pattern, value: Expr) {
        match pattern {
            Pattern::Binding { id, ty } => {
                let name = value_name(*id);
                block.push(Statement::declaration(
                    format!("{} {name}", self.types.c_type(ty)),
                    Some(value),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(name),
                )));
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(
                        block,
                        element,
                        value.clone().field(format!("field_{index}")),
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
