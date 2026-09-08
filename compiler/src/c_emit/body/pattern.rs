use crate::c_emit::syntax::{
    Block, Declaration, Expr, Statement, TranslationUnit, VariableDeclaration,
};
use crate::check::ast::Type;
use crate::closure::ast::{Pattern, TopLevelPattern};

use super::{BodyEmitter, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_top_level_globals(
        &self,
        output: &mut TranslationUnit,
        pattern: &TopLevelPattern,
    ) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } => {
                output.push(Declaration::variable(VariableDeclaration::static_variable(
                    self.types.c_type(ty),
                    value_name(*id),
                )));
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
            TopLevelPattern::Binding { id, ty, .. } => {
                block.push(Statement::assignment(
                    Expr::identifier(value_name(*id)),
                    self.types.copy_value(ty, value),
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

    pub(super) fn destroy_top_level_pattern(&self, block: &mut Block, pattern: &TopLevelPattern) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } => {
                self.types
                    .destroy_value(block, ty, Expr::identifier(value_name(*id)))
            }
            TopLevelPattern::Wildcard { .. } => {}
            TopLevelPattern::Product { elements, .. } => {
                for element in elements.iter().rev() {
                    self.destroy_top_level_pattern(block, element);
                }
            }
        }
    }

    pub(super) fn emit_pattern_bindings(&self, block: &mut Block, pattern: &Pattern, value: Expr) {
        match pattern {
            Pattern::Binding { id, ty } => {
                let name = value_name(*id);
                block.push(Statement::variable(
                    self.types.c_type(ty),
                    &name,
                    Some(self.types.copy_value(ty, value)),
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

    pub(super) fn emit_borrowed_pattern_bindings(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        value: Expr,
    ) -> Vec<crate::anf::ast::ValueId> {
        let mut borrowed = Vec::new();
        self.emit_borrowed_pattern_bindings_into(block, pattern, value, &mut borrowed);
        borrowed
    }

    fn emit_borrowed_pattern_bindings_into(
        &mut self,
        block: &mut Block,
        pattern: &Pattern,
        value: Expr,
        borrowed: &mut Vec<crate::anf::ast::ValueId>,
    ) {
        match pattern {
            Pattern::Binding { id, ty } => {
                let name = value_name(*id);
                block.push(Statement::variable(
                    self.types.c_type(ty),
                    &name,
                    Some(value),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(name),
                )));
                self.borrowed_bindings.insert(*id);
                borrowed.push(*id);
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_borrowed_pattern_bindings_into(
                        block,
                        element,
                        value.clone().field(format!("field_{index}")),
                        borrowed,
                    );
                }
            }
        }
    }

    pub(super) fn end_borrowed_bindings(
        &mut self,
        borrowed: impl IntoIterator<Item = crate::anf::ast::ValueId>,
    ) {
        for id in borrowed {
            self.borrowed_bindings.remove(&id);
        }
    }

    pub(super) fn emit_owned_pattern_bindings(
        &self,
        block: &mut Block,
        pattern: &Pattern,
        value: Expr,
    ) {
        match pattern {
            Pattern::Binding { id, ty } => {
                let name = value_name(*id);
                block.push(Statement::variable(
                    self.types.c_type(ty),
                    &name,
                    Some(value),
                ));
                block.push(Statement::expression(Expr::cast(
                    "void",
                    Expr::identifier(name),
                )));
            }
            Pattern::Wildcard { ty, .. } => self.types.destroy_value(block, ty, value),
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_owned_pattern_bindings(
                        block,
                        element,
                        value.clone().field(format!("field_{index}")),
                    );
                }
            }
        }
    }

    pub(super) fn destroy_pattern_bindings(&self, block: &mut Block, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { id, ty } => {
                if self
                    .closure_uses
                    .direct_closure(*id)
                    .is_some_and(|target| target.creator != *id || self.uses_stack_environment(*id))
                    || self.ephemeral_bindings.contains(id)
                    || self.borrowed_bindings.contains(id)
                {
                    return;
                }
                self.types
                    .destroy_value(block, ty, Expr::identifier(value_name(*id)))
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { elements, .. } => {
                for element in elements.iter().rev() {
                    self.destroy_pattern_bindings(block, element);
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
