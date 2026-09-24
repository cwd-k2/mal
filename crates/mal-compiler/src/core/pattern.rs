use mal_frontend::check::ast as checked;

use super::Lowerer;
use super::ast::{Pattern, TopLevelPattern, ValueId};

impl Lowerer {
    pub(super) fn lower_top_level_pattern(&self, pattern: &checked::Pattern) -> TopLevelPattern {
        match pattern {
            checked::Pattern::Binding { binding, ty } => TopLevelPattern::Binding {
                id: ValueId::Source(binding.id),
                name: binding.name.text.clone(),
                ty: ty.clone(),
            },
            checked::Pattern::Wildcard { ty, span } => TopLevelPattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            checked::Pattern::Product { elements, ty, span } => TopLevelPattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.lower_top_level_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    pub(super) fn lower_pattern(&self, pattern: &checked::Pattern) -> Pattern {
        match pattern {
            checked::Pattern::Binding { binding, ty } => Pattern::Binding {
                id: ValueId::Source(binding.id),
                ty: ty.clone(),
            },
            checked::Pattern::Wildcard { ty, span } => Pattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            checked::Pattern::Product { elements, ty, span } => Pattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.lower_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }
}
