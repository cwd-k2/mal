use crate::ast::BinaryOperator;
use crate::check::ast as checked;

use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern};

type Continuation<'a> = dyn FnMut(&mut Lowerer, Expression) -> Expression + 'a;

mod abrupt;
mod branch;
mod presence;
mod result_block;
mod value;

use presence::contains_control;

impl Lowerer {
    fn lower_with_join(
        &mut self,
        parameter_type: &checked::Type,
        result_type: &checked::Type,
        span: crate::source::Span,
        continuation: &mut Continuation<'_>,
        build: impl FnOnce(&mut Lowerer, &mut Continuation<'_>) -> Expression,
    ) -> Expression {
        let parameter = self.temporary();
        let argument = self.reference(parameter, parameter_type.clone(), span);
        let body = continuation(self, argument);
        let target = super::ast::JoinId(self.joins.len());
        self.joins.push(super::ast::Join {
            parameter: Pattern::Binding {
                id: parameter,
                ty: parameter_type.clone(),
            },
            body,
            span,
        });
        let mut jump = |_: &mut Lowerer, value: Expression| Expression {
            kind: ExpressionKind::Goto {
                target,
                value: Box::new(value),
            },
            ty: result_type.clone(),
            span,
        };
        build(self, &mut jump)
    }

    pub(super) fn lower_lambda_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        result_type: &checked::Type,
    ) -> Expression {
        if let checked::Completion::Value(value) = result
            && !contains_control(value)
            && !items.iter().any(|item| match item {
                checked::BodyItem::Binding(binding) => contains_control(&binding.value),
                checked::BodyItem::Expression(value) => contains_control(value),
            })
        {
            return self.lower_body(items, result);
        }
        let mut identity = |_: &mut Lowerer, value: Expression| value;
        self.lower_items_with(items, result, result_type, &mut identity)
    }

    fn lower_items_with(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
    ) -> Expression {
        let mut body = self.lower_completion_with(result, result_type, continuation);
        for item in items.iter().rev() {
            if !contains_control(body_item_value(item)) {
                body = self.prepend_body_item(item, body);
                continue;
            }
            let mut rest = Some(body);
            let mut next = |lowerer: &mut Lowerer, value: Expression| {
                lowerer.prepend_lowered_body_item(
                    item,
                    value,
                    rest.take().expect("a join continuation is lowered once"),
                )
            };
            body = self.lower_value_with(body_item_value(item), result_type, &mut next);
        }
        body
    }

    fn prepend_body_item(&mut self, item: &checked::BodyItem, body: Expression) -> Expression {
        let (pattern, value, span) = match item {
            checked::BodyItem::Binding(binding) => (
                self.lower_pattern(&binding.pattern),
                self.lower_expression(&binding.value),
                binding.span,
            ),
            checked::BodyItem::Expression(value) => {
                let lowered = self.lower_expression(value);
                (
                    Pattern::Wildcard {
                        ty: lowered.ty.clone(),
                        span: lowered.span,
                    },
                    lowered,
                    value.span,
                )
            }
        };
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern,
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty,
            span,
        }
    }

    fn prepend_lowered_body_item(
        &mut self,
        item: &checked::BodyItem,
        value: Expression,
        body: Expression,
    ) -> Expression {
        let (pattern, span) = match item {
            checked::BodyItem::Binding(binding) => {
                (self.lower_pattern(&binding.pattern), binding.span)
            }
            checked::BodyItem::Expression(source) => (
                Pattern::Wildcard {
                    ty: value.ty.clone(),
                    span: value.span,
                },
                source.span,
            ),
        };
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern,
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty,
            span,
        }
    }

    fn lower_completion_with(
        &mut self,
        completion: &checked::Completion,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
    ) -> Expression {
        match completion {
            checked::Completion::Value(value) => {
                self.lower_value_with(value, result_type, continuation)
            }
            checked::Completion::Abrupt(abrupt) => self.lower_abrupt(abrupt, result_type),
        }
    }

    fn lower_value_with(
        &mut self,
        value: &checked::Expression,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
    ) -> Expression {
        if !contains_control(value) {
            let lowered = self.lower_expression(value);
            return continuation(self, lowered);
        }
        match &value.kind {
            checked::ExpressionKind::Parenthesized(inner) => {
                self.lower_value_with(inner, result_type, continuation)
            }
            checked::ExpressionKind::Block(block) => {
                self.lower_items_with(&block.items, &block.result, result_type, continuation)
            }
            checked::ExpressionKind::ResultBlock { target, body, .. } => self
                .lower_result_block_with(
                    *target,
                    body,
                    &value.ty,
                    result_type,
                    continuation,
                    value.span,
                ),
            checked::ExpressionKind::Product(elements) => self.lower_values_with(
                elements,
                result_type,
                continuation,
                ExpressionKind::Product,
                value,
            ),
            checked::ExpressionKind::Call { callee, argument } => {
                let mut argument_next = |lowerer: &mut Lowerer, argument: Expression| {
                    let mut callee_next = |lowerer: &mut Lowerer, callee: Expression| {
                        let expression = Expression {
                            kind: ExpressionKind::Call {
                                callee: Box::new(callee),
                                argument: Box::new(argument.clone()),
                            },
                            ty: value.ty.clone(),
                            span: value.span,
                        };
                        continuation(lowerer, expression)
                    };
                    lowerer.lower_value_with(callee, result_type, &mut callee_next)
                };
                self.lower_value_with(argument, result_type, &mut argument_next)
            }
            checked::ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => self.lower_with_join(
                &value.ty,
                result_type,
                value.span,
                continuation,
                |lowerer, continuation| {
                    lowerer.lower_sum_elimination_body(
                        scrutinee,
                        continuations,
                        &value.ty,
                        result_type,
                        continuation,
                        value.span,
                    )
                },
            ),
            checked::ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_with_join(
                &value.ty,
                result_type,
                value.span,
                continuation,
                |lowerer, continuation| {
                    lowerer.lower_control_if_body(
                        condition,
                        then_branch,
                        else_branch,
                        result_type,
                        continuation,
                        value.span,
                    )
                },
            ),
            checked::ExpressionKind::Unary { operator, operand } => {
                let mut next = |lowerer: &mut Lowerer, operand: Expression| {
                    let expression = lowerer.lower_unary_value(operator.kind, operand, value);
                    continuation(lowerer, expression)
                };
                self.lower_value_with(operand, result_type, &mut next)
            }
            checked::ExpressionKind::Binary {
                operator,
                left,
                right,
            } if matches!(
                operator.kind,
                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
            ) =>
            {
                self.lower_logical_with(
                    operator.kind,
                    left,
                    right,
                    result_type,
                    continuation,
                    value.span,
                )
            }
            checked::ExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                let mut left_next = |lowerer: &mut Lowerer, left: Expression| {
                    let mut right_next = |lowerer: &mut Lowerer, right: Expression| {
                        let expression =
                            lowerer.lower_binary_value(operator.kind, left.clone(), right, value);
                        continuation(lowerer, expression)
                    };
                    lowerer.lower_value_with(right, result_type, &mut right_next)
                };
                self.lower_value_with(left, result_type, &mut left_next)
            }
            checked::ExpressionKind::SymbolLength { value: operand }
            | checked::ExpressionKind::NumericConversion { value: operand }
            | checked::ExpressionKind::SumInjection { value: operand, .. } => {
                let mut next = |lowerer: &mut Lowerer, operand: Expression| {
                    let kind = match &value.kind {
                        checked::ExpressionKind::SymbolLength { .. } => {
                            ExpressionKind::SymbolLength {
                                value: Box::new(operand),
                            }
                        }
                        checked::ExpressionKind::NumericConversion { .. } => {
                            ExpressionKind::NumericConversion {
                                value: Box::new(operand),
                            }
                        }
                        checked::ExpressionKind::SumInjection { index, .. } => {
                            ExpressionKind::SumInjection {
                                index: *index,
                                value: Box::new(operand),
                            }
                        }
                        _ => unreachable!(),
                    };
                    continuation(
                        lowerer,
                        Expression {
                            kind,
                            ty: value.ty.clone(),
                            span: value.span,
                        },
                    )
                };
                self.lower_value_with(operand, result_type, &mut next)
            }
            checked::ExpressionKind::SymbolAt { argument } => {
                let mut next = |lowerer: &mut Lowerer, argument: Expression| {
                    continuation(
                        lowerer,
                        Expression {
                            kind: ExpressionKind::SymbolAt {
                                argument: Box::new(argument),
                            },
                            ty: value.ty.clone(),
                            span: value.span,
                        },
                    )
                };
                self.lower_value_with(argument, result_type, &mut next)
            }
            checked::ExpressionKind::Memory {
                primitive,
                operands,
            } => self.lower_values_with(
                operands,
                result_type,
                continuation,
                |operands| ExpressionKind::Memory {
                    primitive: *primitive,
                    operands,
                },
                value,
            ),
            _ => unreachable!("control-free values are lowered by the direct path"),
        }
    }

    fn lower_values_with(
        &mut self,
        values: &[checked::Expression],
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        build: impl Fn(Vec<Expression>) -> ExpressionKind + Copy,
        source: &checked::Expression,
    ) -> Expression {
        let bindings = values
            .iter()
            .map(|value| (self.temporary(), value.ty.clone(), value.span))
            .collect::<Vec<_>>();
        let elements = bindings
            .iter()
            .map(|(id, ty, span)| self.reference(*id, ty.clone(), *span))
            .collect();
        let mut body = continuation(
            self,
            Expression {
                kind: build(elements),
                ty: source.ty.clone(),
                span: source.span,
            },
        );
        for (value, (id, ty, span)) in values.iter().zip(bindings).rev() {
            let mut rest = Some(body);
            let mut next = |_: &mut Lowerer, value: Expression| Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(Binding {
                        pattern: Pattern::Binding { id, ty: ty.clone() },
                        value,
                        span,
                    }),
                    body: Box::new(rest.take().expect("a join continuation is lowered once")),
                },
                ty: result_type.clone(),
                span,
            };
            body = self.lower_value_with(value, result_type, &mut next);
        }
        body
    }
}

fn body_item_value(item: &checked::BodyItem) -> &checked::Expression {
    match item {
        checked::BodyItem::Binding(binding) => &binding.value,
        checked::BodyItem::Expression(value) => value,
    }
}
