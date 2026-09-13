use crate::resolve::ast as resolved;

use super::Index;
use crate::editor::{OccurrenceRole, SymbolId};

impl Index {
    pub(super) fn collect_resolved_top(&mut self, item: &resolved::TopItem) {
        match item {
            resolved::TopItem::TypeAlias { binding, value } => {
                let id = SymbolId::Type(binding.id);
                self.type_details
                    .insert(binding.id, super::type_display::type_name(value));
                self.top_level.push(id);
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
                self.collect_resolved_type(value);
            }
            resolved::TopItem::ExternalType { binding } => {
                let id = SymbolId::Type(binding.id);
                self.top_level.push(id);
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
            }
            resolved::TopItem::ExternalOperation { binding, ty, .. } => {
                self.value_types
                    .insert(binding.id, super::type_display::type_name(ty));
                self.functions.insert(binding.id);
                let id = SymbolId::Value(binding.id);
                self.top_level.push(id);
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
                self.collect_resolved_type(ty);
            }
            resolved::TopItem::Binding(binding) => {
                self.collect_resolved_binding(binding, true);
            }
        }
    }

    fn collect_resolved_type(&mut self, ty: &crate::ast::Node<resolved::TypeExpression>) {
        match &ty.kind {
            resolved::TypeExpression::Named(reference) => self.add_raw(
                SymbolId::Type(reference.id),
                &reference.name,
                OccurrenceRole::Reference,
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

    fn collect_resolved_binding(&mut self, binding: &resolved::Binding, top_level: bool) {
        if let Some(annotation) = &binding.annotation {
            self.collect_resolved_type(annotation);
            self.apply_declared_pattern_type(&binding.pattern, annotation);
        }
        self.collect_resolved_expression_with_expected(&binding.value, binding.annotation.as_ref());
        self.collect_resolved_pattern(&binding.pattern, top_level);
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
            _ => self.collect_resolved_expression(expression),
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
            self.collect_resolved_pattern(parameter, false);
        }
        if let Some(return_binders) = &lambda.return_binders {
            match return_binders.as_slice() {
                [] => {}
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
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
            }
        }
        for item in &lambda.body.items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_resolved_binding(&binding.kind, false);
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
    ) {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                let id = SymbolId::Value(self.canonical_value(binding.id));
                if top_level {
                    self.top_level.push(id);
                }
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
            }
            resolved::Pattern::Product(elements) => {
                for element in elements {
                    self.collect_resolved_pattern(element, top_level);
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
            ),
            Expression::Parenthesized(inner) => self.collect_resolved_expression(inner),
            Expression::TypeQualifiedPrimitive { type_ref, .. } => self.add_raw(
                SymbolId::Type(type_ref.id),
                &type_ref.name,
                OccurrenceRole::Reference,
            ),
            Expression::Product(elements) => {
                for element in elements {
                    self.collect_resolved_expression(element);
                }
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
                    self.collect_resolved_binding(&binding.kind, false);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_resolved_expression(expression);
                }
            }
        }
        self.collect_resolved_expression(result);
    }
}
