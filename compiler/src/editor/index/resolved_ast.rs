use crate::resolve::ast as resolved;

use super::Index;
use crate::editor::{OccurrenceRole, SymbolId};

impl Index {
    pub(super) fn collect_resolved_top(&mut self, item: &crate::ast::Node<resolved::TopItem>) {
        match &item.kind {
            resolved::TopItem::TypeAlias { binding, value } => {
                let id = SymbolId::Type(binding.id);
                self.type_details
                    .insert(binding.id, super::type_display::type_name(value));
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                self.collect_resolved_type(value);
            }
            resolved::TopItem::ExternalType { binding } => {
                let id = SymbolId::Type(binding.id);
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
            }
            resolved::TopItem::ExternalOperation { binding, ty, .. } => {
                self.value_types
                    .insert(binding.id, super::type_display::type_name(ty));
                self.functions.insert(binding.id);
                let id = SymbolId::Value(binding.id);
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                self.collect_resolved_type(ty);
            }
            resolved::TopItem::Binding(binding) => {
                self.collect_resolved_binding(binding, true, item.span);
            }
        }
    }

    fn collect_resolved_type(&mut self, ty: &crate::ast::Node<resolved::TypeExpression>) {
        match &ty.kind {
            resolved::TypeExpression::Named(reference) => self.add_raw(
                SymbolId::Type(reference.id),
                &reference.name,
                OccurrenceRole::Reference,
                None,
            ),
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

    fn collect_resolved_binding(
        &mut self,
        binding: &resolved::Binding,
        top_level: bool,
        declaration_span: crate::source::Span,
    ) {
        if let Some(annotation) = &binding.annotation {
            self.collect_resolved_type(annotation);
            self.apply_declared_pattern_type(&binding.pattern, annotation);
        }
        self.collect_resolved_expression_with_expected(&binding.value, binding.annotation.as_ref());
        self.collect_resolved_pattern(&binding.pattern, top_level, declaration_span);
    }

    fn apply_declared_pattern_type(
        &mut self,
        pattern: &crate::ast::Node<resolved::Pattern>,
        ty: &crate::ast::Node<resolved::TypeExpression>,
    ) {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                let id = self.canonical_value(binding.id);
                self.value_types
                    .insert(id, super::type_display::type_name(ty));
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

    fn collect_resolved_expression_with_expected(
        &mut self,
        expression: &crate::ast::Node<resolved::Expression>,
        expected: Option<&crate::ast::Node<resolved::TypeExpression>>,
    ) {
        match (&expression.kind, expected) {
            (resolved::Expression::Parenthesized(inner), Some(expected)) => {
                self.collect_resolved_expression_with_expected(inner, Some(expected));
            }
            (resolved::Expression::Lambda(lambda), Some(expected)) => {
                let expanded = self.expanded_type(expected);
                let resolved::TypeExpression::Function { parameter, result } = expanded.kind else {
                    self.collect_resolved_expression(expression);
                    return;
                };
                self.collect_resolved_lambda(lambda, Some(&parameter), Some(&result));
            }
            (resolved::Expression::Block(block), Some(expected)) => {
                for item in &block.items {
                    match item {
                        resolved::BodyItem::Binding(binding) => {
                            self.collect_resolved_binding(&binding.kind, false, binding.span);
                        }
                        resolved::BodyItem::Expression(expression) => {
                            self.collect_resolved_expression(expression);
                        }
                    }
                }
                self.collect_resolved_expression_with_expected(&block.result, Some(expected));
            }
            (
                resolved::Expression::ResultBlock {
                    return_binders,
                    body,
                },
                Some(expected),
            ) => {
                self.collect_resolved_return_binders(return_binders, Some(expected));
                for item in &body.items {
                    match item {
                        resolved::BodyItem::Binding(binding) => {
                            self.collect_resolved_binding(&binding.kind, false, binding.span);
                        }
                        resolved::BodyItem::Expression(expression) => {
                            self.collect_resolved_expression(expression);
                        }
                    }
                }
                self.collect_resolved_expression_with_expected(&body.result, Some(expected));
            }
            _ => self.collect_resolved_expression(expression),
        }
    }

    fn collect_resolved_return_binders(
        &mut self,
        return_binders: &[resolved::ValueBinding],
        result_type: Option<&crate::ast::Node<resolved::TypeExpression>>,
    ) {
        match return_binders {
            [binding] => {
                if let Some(result_type) = result_type {
                    self.value_types.insert(
                        self.canonical_value(binding.id),
                        super::type_display::type_name(result_type),
                    );
                }
            }
            bindings => {
                if let Some(result_type) = result_type
                    && let resolved::TypeExpression::Sum(members) =
                        self.expanded_type(result_type).kind
                {
                    for (binding, member) in bindings.iter().zip(&members) {
                        self.value_types.insert(
                            self.canonical_value(binding.id),
                            super::type_display::type_name(member),
                        );
                    }
                }
            }
        }
        for binding in return_binders {
            let id = SymbolId::Value(self.canonical_value(binding.id));
            self.add_raw(
                id,
                &binding.name,
                OccurrenceRole::Declaration,
                Some(binding.name.span),
            );
        }
    }

    fn collect_resolved_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        parameter_type: Option<&crate::ast::Node<resolved::TypeExpression>>,
        result_type: Option<&crate::ast::Node<resolved::TypeExpression>>,
    ) {
        if let Some(parameter) = &lambda.parameter {
            if let Some(parameter_type) = parameter_type {
                self.apply_declared_pattern_type(parameter, parameter_type);
            }
            self.collect_resolved_pattern(parameter, false, parameter.span);
        }
        if let Some(return_binders) = &lambda.return_binders {
            self.collect_resolved_return_binders(return_binders, result_type);
        }
        for item in &lambda.body.items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_resolved_binding(&binding.kind, false, binding.span);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_resolved_expression(expression);
                }
            }
        }
        self.collect_resolved_expression_with_expected(&lambda.body.result, result_type);
    }

    fn collect_resolved_pattern(
        &mut self,
        pattern: &crate::ast::Node<resolved::Pattern>,
        top_level: bool,
        declaration_span: crate::source::Span,
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

    fn collect_resolved_expression(&mut self, expression: &crate::ast::Node<resolved::Expression>) {
        use resolved::Expression;
        match &expression.kind {
            Expression::Reference(reference) => self.add_raw(
                SymbolId::Value(self.canonical_value(reference.id)),
                &reference.name,
                OccurrenceRole::Reference,
                None,
            ),
            Expression::Parenthesized(inner) => self.collect_resolved_expression(inner),
            Expression::TypeQualifiedPrimitive { type_ref, .. } => self.add_raw(
                SymbolId::Type(type_ref.id),
                &type_ref.name,
                OccurrenceRole::Reference,
                None,
            ),
            Expression::Product(elements) => {
                for element in elements {
                    self.collect_resolved_expression(element);
                }
            }
            Expression::Block(block) => {
                self.collect_resolved_body(&block.items, &block.result);
            }
            Expression::ResultBlock {
                return_binders,
                body,
            } => {
                self.collect_resolved_return_binders(return_binders, None);
                self.collect_resolved_body(&body.items, &body.result);
            }
            Expression::Lambda(lambda) => {
                self.collect_resolved_lambda(lambda, None, None);
            }
            Expression::Call { callee, arguments } => {
                self.collect_resolved_expression(callee);
                for argument in arguments {
                    self.collect_resolved_expression(argument);
                }
            }
            Expression::ContinuationApplication {
                value,
                continuations,
            } => {
                self.collect_resolved_expression(value);
                for continuation in continuations {
                    self.collect_resolved_expression(continuation);
                }
            }
            Expression::Conversion {
                type_ref, value, ..
            } => {
                self.add_raw(
                    SymbolId::Type(type_ref.id),
                    &type_ref.name,
                    OccurrenceRole::Reference,
                    None,
                );
                self.collect_resolved_expression(value);
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_resolved_expression(condition);
                self.collect_resolved_body(&then_branch.items, &then_branch.result);
                self.collect_resolved_body(&else_branch.items, &else_branch.result);
            }
            Expression::When { condition, body } => {
                self.collect_resolved_expression(condition);
                self.collect_resolved_body(&body.items, &body.result);
            }
            Expression::Unary { operand, .. } => self.collect_resolved_expression(operand),
            Expression::Binary { left, right, .. } => {
                let mut pending = vec![right.as_ref(), left.as_ref()];
                while let Some(expression) = pending.pop() {
                    if let Expression::Binary { left, right, .. } = &expression.kind {
                        pending.push(right);
                        pending.push(left);
                    } else {
                        self.collect_resolved_expression(expression);
                    }
                }
            }
            Expression::Integer(_)
            | Expression::Float(_)
            | Expression::Byte(_)
            | Expression::Symbol(_)
            | Expression::Unit => {}
        }
    }

    fn collect_resolved_body(
        &mut self,
        items: &[resolved::BodyItem],
        result: &crate::ast::Node<resolved::Expression>,
    ) {
        for item in items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_resolved_binding(&binding.kind, false, binding.span);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_resolved_expression(expression);
                }
            }
        }
        self.collect_resolved_expression(result);
    }
}
