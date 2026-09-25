use crate::resolve::ast as resolved;

use super::Index;

impl Index {
    pub(super) fn collect_aliases_top(&mut self, item: &resolved::TopItem) {
        match item {
            resolved::TopItem::Binding(binding) => {
                self.collect_aliases_expression(&binding.value.kind)
            }
            resolved::TopItem::GenericBinding { value, .. } => {
                self.collect_aliases_expression(&value.kind)
            }
            _ => {}
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
            Expression::Block(block) => {
                self.collect_aliases_body(&block.items, &block.result.kind);
            }
            Expression::ResultBlock { body, .. } => {
                self.collect_aliases_body(&body.items, &body.result.kind);
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
            Expression::ContinuationApplication {
                value,
                continuations,
            } => {
                self.collect_aliases_expression(&value.kind);
                for continuation in continuations {
                    match continuation {
                        resolved::Continuation::Function(expression) => {
                            self.collect_aliases_expression(&expression.kind);
                        }
                        resolved::Continuation::Branch(branch) => {
                            self.collect_aliases_body(&branch.body.items, &branch.body.result.kind);
                        }
                    }
                }
            }
            Expression::Conversion { value, .. } => {
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
            Expression::When { condition, body } => {
                self.collect_aliases_expression(&condition.kind);
                self.collect_aliases_body(&body.items, &body.result.kind);
            }
            Expression::Unary { operand, .. } => self.collect_aliases_expression(&operand.kind),
            Expression::Binary { left, right, .. } => {
                let mut pending = vec![&right.kind, &left.kind];
                while let Some(expression) = pending.pop() {
                    if let Expression::Binary { left, right, .. } = expression {
                        pending.push(&right.kind);
                        pending.push(&left.kind);
                    } else {
                        self.collect_aliases_expression(expression);
                    }
                }
            }
            Expression::Reference(_)
            | Expression::GenericReference { .. }
            | Expression::Integer(_)
            | Expression::Float(_)
            | Expression::Byte(_)
            | Expression::Symbol(_)
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
