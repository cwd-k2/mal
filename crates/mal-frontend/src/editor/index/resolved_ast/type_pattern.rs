use crate::editor::{OccurrenceRole, SymbolId};
use crate::resolve::ast as resolved;

use super::Index;

impl Index {
    pub(super) fn collect_resolved_type(
        &mut self,
        ty: &mal_syntax::ast::Node<resolved::TypeExpression>,
    ) {
        match &ty.kind {
            resolved::TypeExpression::Named(reference) => self.add_raw(
                SymbolId::Type(reference.id),
                &reference.name,
                OccurrenceRole::Reference,
                None,
            ),
            resolved::TypeExpression::Application {
                constructor,
                arguments,
            } => {
                self.add_raw(
                    SymbolId::Type(constructor.id),
                    &constructor.name,
                    OccurrenceRole::Reference,
                    None,
                );
                for argument in arguments {
                    self.collect_resolved_type(argument);
                }
            }
            resolved::TypeExpression::Parenthesized(inner) => self.collect_resolved_type(inner),
            resolved::TypeExpression::Product(elements)
            | resolved::TypeExpression::Sum(elements) => {
                for element in elements {
                    self.collect_resolved_type(element);
                }
            }
            resolved::TypeExpression::Function { parameter, result } => {
                self.collect_resolved_type(parameter);
                self.collect_resolved_type(result);
            }
            resolved::TypeExpression::Unit => {}
        }
    }

    pub(super) fn apply_declared_pattern_type(
        &mut self,
        pattern: &mal_syntax::ast::Node<resolved::Pattern>,
        ty: &mal_syntax::ast::Node<resolved::TypeExpression>,
    ) {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                let id = self.canonical_value(binding.id);
                self.value_types
                    .insert(id, super::super::type_display::type_name(ty));
                self.declared_value_types.insert(id, ty.clone());
            }
            resolved::Pattern::Product(patterns) => {
                let expanded = self.expanded_type(ty);
                if let resolved::TypeExpression::Product(types) = expanded.kind {
                    for (pattern, ty) in patterns.iter().zip(&types) {
                        self.apply_declared_pattern_type(pattern, ty);
                    }
                }
            }
            resolved::Pattern::Wildcard => {}
        }
    }

    pub(super) fn collect_resolved_pattern(
        &mut self,
        pattern: &mal_syntax::ast::Node<resolved::Pattern>,
        top_level: bool,
        declaration_span: mal_syntax::source::Span,
    ) {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                let id = SymbolId::Value(self.canonical_value(binding.id));
                if top_level {
                    self.top_level.push(id);
                }
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(declaration_span),
                );
            }
            resolved::Pattern::Product(elements) => {
                for element in elements {
                    self.collect_resolved_pattern(element, top_level, declaration_span);
                }
            }
            resolved::Pattern::Wildcard => {}
        }
    }
}
