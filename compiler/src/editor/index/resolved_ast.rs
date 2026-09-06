use crate::resolve::ast as resolved;

use super::Index;
use crate::editor::{OccurrenceRole, SymbolId};

impl Index<'_> {
    pub(super) fn collect_resolved_top(&mut self, item: &resolved::TopItem) {
        match item {
            resolved::TopItem::TypeAlias { binding, value } => {
                let id = SymbolId::Type(binding.id);
                self.top_level.push(id);
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
                self.collect_resolved_type(value);
            }
            resolved::TopItem::ExternalType { binding } => {
                let id = SymbolId::Type(binding.id);
                self.top_level.push(id);
                self.add_raw(id, &binding.name, OccurrenceRole::Declaration);
            }
            resolved::TopItem::ExternalOperation { id, name, ty } => {
                let id = SymbolId::ExternalOperation(*id);
                self.top_level.push(id);
                self.add_raw(id, name, OccurrenceRole::Declaration);
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
        }
        self.collect_resolved_expression(&binding.value);
        self.collect_resolved_pattern(&binding.pattern, top_level);
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
            Expression::StorageSize(ty) => self.collect_resolved_type(ty),
            Expression::Product(elements) => {
                for element in elements {
                    self.collect_resolved_expression(element);
                }
            }
            Expression::Lambda(lambda) => {
                for capture in &lambda.captures {
                    self.add_raw(
                        SymbolId::Value(self.canonical_value(capture.source.id)),
                        &capture.source.name,
                        OccurrenceRole::Reference,
                    );
                }
                for parameter in &lambda.parameters {
                    self.collect_resolved_type(&parameter.ty);
                    self.add_raw(
                        SymbolId::Value(parameter.binding.id),
                        &parameter.binding.name,
                        OccurrenceRole::Declaration,
                    );
                }
                self.collect_resolved_body(&lambda.body.items, &lambda.body.result);
            }
            Expression::Call { callee, arguments } => {
                self.collect_resolved_expression(callee);
                for argument in arguments {
                    self.collect_resolved_expression(argument);
                }
            }
            Expression::ExternalCall {
                operation,
                arguments,
            } => {
                self.add_raw(
                    SymbolId::ExternalOperation(operation.id),
                    &operation.name,
                    OccurrenceRole::Reference,
                );
                for argument in arguments {
                    self.collect_resolved_expression(argument);
                }
            }
            Expression::Conversion { type_ref, value }
            | Expression::SumInjection {
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
            Expression::Case { scrutinee, arms } => {
                self.collect_resolved_expression(scrutinee);
                for arm in arms {
                    self.collect_resolved_pattern(&arm.pattern, false);
                    self.collect_resolved_body(&arm.body.items, &arm.body.result);
                }
            }
            Expression::Unary { operand, .. } => self.collect_resolved_expression(operand),
            Expression::Binary { left, right, .. } => {
                self.collect_resolved_expression(left);
                self.collect_resolved_expression(right);
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
