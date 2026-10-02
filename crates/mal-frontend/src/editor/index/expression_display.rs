//! Display types of expressions as the source spells them: declared, propagated from context, or instantiated
//! from a generic signature, so hover keeps aliases.

use super::*;

impl Index {
    pub(super) fn expanded_type(
        &self,
        ty: &mal_syntax::ast::Node<resolved::TypeExpression>,
    ) -> mal_syntax::ast::Node<resolved::TypeExpression> {
        let mut expanded = ty.clone();
        let mut seen = HashSet::new();
        loop {
            match &expanded.kind {
                resolved::TypeExpression::Named(reference) if seen.insert(reference.id) => {
                    let Some(alias) = self.type_aliases.get(&reference.id) else {
                        return expanded;
                    };
                    expanded = alias.clone();
                }
                resolved::TypeExpression::Application {
                    constructor,
                    arguments,
                } if seen.insert(constructor.id) => {
                    let Some(alias) = self.generic_type_aliases.get(&constructor.id) else {
                        return expanded;
                    };
                    let substitutions = alias
                        .parameters
                        .iter()
                        .copied()
                        .zip(arguments.iter().cloned())
                        .collect();
                    expanded = type_display::substitute(&alias.value, &substitutions);
                }
                resolved::TypeExpression::Parenthesized(inner) => expanded = (**inner).clone(),
                _ => return expanded,
            }
        }
    }

    pub(super) fn expression_display_type(
        &self,
        expression: &mal_syntax::ast::Node<resolved::Expression>,
    ) -> Option<mal_syntax::ast::Node<resolved::TypeExpression>> {
        use resolved::Expression;
        match &expression.kind {
            Expression::Reference(reference) => self
                .declared_value_types
                .get(&self.canonical_value(reference.id))
                .cloned(),
            Expression::GenericReference {
                reference,
                arguments,
            } => {
                let signature = self.generic_value_types.get(&reference.id)?;
                let substitutions = signature
                    .parameters
                    .iter()
                    .copied()
                    .zip(arguments.iter().cloned())
                    .collect();
                Some(type_display::substitute(&signature.ty, &substitutions))
            }
            Expression::Parenthesized(inner) => self.expression_display_type(inner),
            Expression::Call { callee, .. } => {
                let signature = self.expanded_type(&self.expression_display_type(callee)?);
                match signature.kind {
                    resolved::TypeExpression::Function { result, .. } => Some(*result),
                    _ => None,
                }
            }
            Expression::Block(block) => self.expression_display_type(&block.result),
            Expression::ResultBlock { body, .. } => self.expression_display_type(&body.result),
            Expression::If {
                then_branch,
                else_branch,
                ..
            } => self
                .expression_display_type(&then_branch.result)
                .or_else(|| self.expression_display_type(&else_branch.result)),
            _ => None,
        }
    }

    pub(super) fn inferred_expression_display_name(
        &self,
        expression: &mal_syntax::ast::Node<resolved::Expression>,
    ) -> Option<String> {
        use resolved::Expression;
        match &expression.kind {
            Expression::Reference(reference) => {
                self.instantiated_generic_type_name(reference.id, expression.span, false)
            }
            Expression::Parenthesized(inner) => self.inferred_expression_display_name(inner),
            Expression::Call { callee, .. } => {
                let Expression::Reference(reference) = &callee.kind else {
                    return None;
                };
                self.instantiated_generic_type_name(reference.id, callee.span, true)
            }
            Expression::Block(block) => self.inferred_expression_display_name(&block.result),
            Expression::ResultBlock { body, .. } => {
                self.inferred_expression_display_name(&body.result)
            }
            Expression::If {
                then_branch,
                else_branch,
                ..
            } => self
                .inferred_expression_display_name(&then_branch.result)
                .or_else(|| self.inferred_expression_display_name(&else_branch.result)),
            _ => None,
        }
    }

    fn instantiated_generic_type_name(
        &self,
        id: resolved::ValueId,
        span: Span,
        call_result: bool,
    ) -> Option<String> {
        let signature = self.generic_value_types.get(&id)?;
        let arguments = self.inferred_type_arguments.get(&span)?;
        let substitutions = signature
            .parameters
            .iter()
            .copied()
            .zip(arguments.iter().cloned())
            .collect();
        let ty = if call_result {
            let resolved::TypeExpression::Function { result, .. } = &signature.ty.kind else {
                return None;
            };
            result.as_ref()
        } else {
            &signature.ty
        };
        Some(type_display::type_name_with_substitutions(
            ty,
            &substitutions,
        ))
    }
}
