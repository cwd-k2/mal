use crate::resolve::ast as resolved;

use super::Index;

impl Index<'_> {
    pub(super) fn collect_aliases_top(&mut self, item: &resolved::TopItem) {
        if let resolved::TopItem::Binding(binding) = item {
            self.collect_aliases_expression(&binding.value.kind);
        }
    }

    fn collect_aliases_expression(&mut self, expression: &resolved::Expression) {
        use resolved::Expression;
        match expression {
            Expression::Parenthesized(inner) => self.collect_aliases_expression(&inner.kind),
            Expression::Product(elements) => {
                for element in elements {
                    self.collect_aliases_expression(&element.kind);
                }
            }
            Expression::Lambda(lambda) => {
                for capture in &lambda.captures {
                    let source = self.canonical_value(capture.source.id);
                    self.aliases.insert(capture.binding.id, source);
                }
                self.collect_aliases_body(&lambda.body.items, &lambda.body.result.kind);
            }
            Expression::Call { callee, arguments } => {
                self.collect_aliases_expression(&callee.kind);
                for argument in arguments {
                    self.collect_aliases_expression(&argument.kind);
                }
            }
            Expression::ExternalCall { arguments, .. } => {
                for argument in arguments {
                    self.collect_aliases_expression(&argument.kind);
                }
            }
            Expression::Conversion { value, .. } | Expression::SumInjection { value, .. } => {
                self.collect_aliases_expression(&value.kind);
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_aliases_expression(&condition.kind);
                self.collect_aliases_body(&then_branch.items, &then_branch.result.kind);
                self.collect_aliases_body(&else_branch.items, &else_branch.result.kind);
            }
            Expression::Case { scrutinee, arms } => {
                self.collect_aliases_expression(&scrutinee.kind);
                for arm in arms {
                    self.collect_aliases_body(&arm.body.items, &arm.body.result.kind);
                }
            }
            Expression::Unary { operand, .. } => self.collect_aliases_expression(&operand.kind),
            Expression::Binary { left, right, .. } => {
                self.collect_aliases_expression(&left.kind);
                self.collect_aliases_expression(&right.kind);
            }
            Expression::Reference(_)
            | Expression::Integer(_)
            | Expression::Float(_)
            | Expression::Byte(_)
            | Expression::String(_)
            | Expression::Unit => {}
        }
    }

    fn collect_aliases_body(
        &mut self,
        items: &[resolved::BodyItem],
        result: &resolved::Expression,
    ) {
        for item in items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_aliases_expression(&binding.kind.value.kind);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_aliases_expression(&expression.kind);
                }
            }
        }
        self.collect_aliases_expression(result);
    }
}
