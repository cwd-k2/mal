use crate::ast::BinaryOperator;
use crate::check::ast as checked;

use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern};

type Continuation<'a> = dyn FnMut(&mut Lowerer, Expression) -> Expression + 'a;

mod presence;
mod value;

use presence::contains_control;

impl Lowerer {
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
        let direct = items
            .iter()
            .take_while(|item| !contains_control(body_item_value(item)))
            .count();
        if direct != 0 {
            let mut body =
                self.lower_items_with(&items[direct..], result, result_type, continuation);
            for item in items[..direct].iter().rev() {
                body = self.prepend_body_item(item, body);
            }
            return body;
        }
        let Some((first, rest)) = items.split_first() else {
            return self.lower_completion_with(result, result_type, continuation);
        };
        match first {
            checked::BodyItem::Binding(binding) => {
                let mut next = |lowerer: &mut Lowerer, value: Expression| {
                    let body = lowerer.lower_items_with(rest, result, result_type, continuation);
                    let ty = body.ty.clone();
                    Expression {
                        kind: ExpressionKind::Let {
                            binding: Box::new(Binding {
                                pattern: lowerer.lower_pattern(&binding.pattern),
                                value,
                                span: binding.span,
                            }),
                            body: Box::new(body),
                        },
                        ty,
                        span: binding.span,
                    }
                };
                self.lower_value_with(&binding.value, result_type, &mut next)
            }
            checked::BodyItem::Expression(value) => {
                let mut next = |lowerer: &mut Lowerer, lowered: Expression| {
                    let body = lowerer.lower_items_with(rest, result, result_type, continuation);
                    let ty = body.ty.clone();
                    Expression {
                        kind: ExpressionKind::Let {
                            binding: Box::new(Binding {
                                pattern: Pattern::Wildcard {
                                    ty: lowered.ty.clone(),
                                    span: lowered.span,
                                },
                                value: lowered,
                                span: value.span,
                            }),
                            body: Box::new(body),
                        },
                        ty,
                        span: value.span,
                    }
                };
                self.lower_value_with(value, result_type, &mut next)
            }
        }
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

    fn lower_abrupt(
        &mut self,
        abrupt: &checked::AbruptExpression,
        result_type: &checked::Type,
    ) -> Expression {
        let terminal = match &abrupt.kind {
            checked::AbruptExpressionKind::Return { value } => {
                let mut identity = |_: &mut Lowerer, value: Expression| value;
                self.lower_value_with(value, result_type, &mut identity)
            }
            checked::AbruptExpressionKind::EmptyElimination { scrutinee } => {
                let span = abrupt.span;
                let mut eliminate = |_lowerer: &mut Lowerer, value: Expression| Expression {
                    kind: ExpressionKind::Case {
                        scrutinee: Box::new(value),
                        arms: Vec::new(),
                    },
                    ty: result_type.clone(),
                    span,
                };
                self.lower_value_with(scrutinee, result_type, &mut eliminate)
            }
            checked::AbruptExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_control_if(
                condition,
                then_branch,
                else_branch,
                result_type,
                &mut |_: &mut Lowerer, value| value,
                abrupt.span,
            ),
        };
        self.lower_preceding(&abrupt.preceding, terminal, result_type, abrupt.span)
    }

    fn lower_preceding(
        &mut self,
        preceding: &[checked::Expression],
        terminal: Expression,
        result_type: &checked::Type,
        span: crate::source::Span,
    ) -> Expression {
        let Some((first, rest)) = preceding.split_first() else {
            return terminal;
        };
        let mut next = |lowerer: &mut Lowerer, value: Expression| {
            let body = lowerer.lower_preceding(rest, terminal.clone(), result_type, span);
            Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(Binding {
                        pattern: Pattern::Wildcard {
                            ty: value.ty.clone(),
                            span: value.span,
                        },
                        value,
                        span: first.span,
                    }),
                    body: Box::new(body),
                },
                ty: result_type.clone(),
                span,
            }
        };
        self.lower_value_with(first, result_type, &mut next)
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
            checked::ExpressionKind::Product(elements) => self.lower_values_with(
                elements,
                result_type,
                Vec::new(),
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
            } => {
                let checked::Type::Sum(members) = &scrutinee.ty else {
                    unreachable!("checked sum elimination scrutinee")
                };
                let mut scrutinee_next = |lowerer: &mut Lowerer, scrutinee: Expression| {
                    let mut arms = Vec::with_capacity(continuations.len());
                    for (index, (member, branch)) in members.iter().zip(continuations).enumerate() {
                        let payload_id = lowerer.temporary();
                        let payload = lowerer.reference(payload_id, member.clone(), branch.span);
                        let mut branch_next = |lowerer: &mut Lowerer, callee: Expression| {
                            continuation(
                                lowerer,
                                Expression {
                                    kind: ExpressionKind::Call {
                                        callee: Box::new(callee),
                                        argument: Box::new(payload.clone()),
                                    },
                                    ty: value.ty.clone(),
                                    span: branch.span,
                                },
                            )
                        };
                        let branch =
                            lowerer.lower_value_with(branch, result_type, &mut branch_next);
                        arms.push(super::ast::CaseArm {
                            index,
                            pattern: Pattern::Binding {
                                id: payload_id,
                                ty: member.clone(),
                            },
                            value: branch,
                            span: value.span,
                        });
                    }
                    Expression {
                        kind: ExpressionKind::Case {
                            scrutinee: Box::new(scrutinee),
                            arms,
                        },
                        ty: result_type.clone(),
                        span: value.span,
                    }
                };
                self.lower_value_with(scrutinee, result_type, &mut scrutinee_next)
            }
            checked::ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_control_if(
                condition,
                then_branch,
                else_branch,
                result_type,
                continuation,
                value.span,
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
            checked::ExpressionKind::SymbolAt { argument }
            | checked::ExpressionKind::Memory { argument, .. } => {
                let mut next = |lowerer: &mut Lowerer, argument: Expression| {
                    let kind = match &value.kind {
                        checked::ExpressionKind::SymbolAt { .. } => ExpressionKind::SymbolAt {
                            argument: Box::new(argument),
                        },
                        checked::ExpressionKind::Memory { primitive, .. } => {
                            ExpressionKind::Memory {
                                primitive: *primitive,
                                argument: Box::new(argument),
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
                self.lower_value_with(argument, result_type, &mut next)
            }
            _ => unreachable!("control-free values are lowered by the direct path"),
        }
    }

    fn lower_values_with(
        &mut self,
        values: &[checked::Expression],
        result_type: &checked::Type,
        lowered: Vec<Expression>,
        continuation: &mut Continuation<'_>,
        build: impl Fn(Vec<Expression>) -> ExpressionKind + Copy,
        source: &checked::Expression,
    ) -> Expression {
        let Some((first, rest)) = values.split_first() else {
            return continuation(
                self,
                Expression {
                    kind: build(lowered),
                    ty: source.ty.clone(),
                    span: source.span,
                },
            );
        };
        let mut next = |lowerer: &mut Lowerer, value: Expression| {
            let mut accumulated = lowered.clone();
            accumulated.push(value);
            lowerer.lower_values_with(rest, result_type, accumulated, continuation, build, source)
        };
        self.lower_value_with(first, result_type, &mut next)
    }

    fn lower_control_if(
        &mut self,
        condition: &checked::Expression,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: crate::source::Span,
    ) -> Expression {
        let mut next = |lowerer: &mut Lowerer, condition: Expression| {
            let otherwise = lowerer.lower_items_with(
                &else_branch.items,
                &else_branch.result,
                result_type,
                continuation,
            );
            let then = lowerer.lower_items_with(
                &then_branch.items,
                &then_branch.result,
                result_type,
                continuation,
            );
            lowerer.case(
                condition,
                vec![
                    lowerer.wildcard_arm(0, otherwise, else_branch.span),
                    lowerer.wildcard_arm(1, then, then_branch.span),
                ],
                result_type.clone(),
                span,
            )
        };
        self.lower_value_with(condition, result_type, &mut next)
    }

    fn lower_logical_with(
        &mut self,
        operator: BinaryOperator,
        left: &checked::Expression,
        right: &checked::Expression,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: crate::source::Span,
    ) -> Expression {
        let mut next = |lowerer: &mut Lowerer, left: Expression| {
            let constant = lowerer.bool_value(operator == BinaryOperator::LogicalOr, span);
            let constant = continuation(lowerer, constant);
            let right = lowerer.lower_value_with(right, result_type, continuation);
            let (zero, one) = if operator == BinaryOperator::LogicalAnd {
                (constant, right)
            } else {
                (right, constant)
            };
            lowerer.case(
                left,
                vec![
                    lowerer.wildcard_arm(0, zero, span),
                    lowerer.wildcard_arm(1, one, span),
                ],
                result_type.clone(),
                span,
            )
        };
        self.lower_value_with(left, result_type, &mut next)
    }
}

fn body_item_value(item: &checked::BodyItem) -> &checked::Expression {
    match item {
        checked::BodyItem::Binding(binding) => &binding.value,
        checked::BodyItem::Expression(value) => value,
    }
}
