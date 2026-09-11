use crate::ast::{BinaryOperator, UnaryOperator};
use crate::check::ast as checked;

use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern};

type Continuation<'a> = dyn FnMut(&mut Lowerer, Expression) -> Expression + 'a;

impl Lowerer {
    pub(super) fn lower_lambda_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        result_type: &checked::Type,
    ) -> Expression {
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

    fn lower_unary_value(
        &mut self,
        operator: UnaryOperator,
        operand: Expression,
        source: &checked::Expression,
    ) -> Expression {
        match operator {
            UnaryOperator::LogicalNot => self.case(
                operand,
                vec![
                    self.wildcard_arm(0, self.bool_value(true, source.span), source.span),
                    self.wildcard_arm(1, self.bool_value(false, source.span), source.span),
                ],
                source.ty.clone(),
                source.span,
            ),
            UnaryOperator::Negate | UnaryOperator::BitwiseNot => Expression {
                kind: ExpressionKind::PrimitiveUnary {
                    operator: if operator == UnaryOperator::Negate {
                        super::ast::UnaryPrimitive::Negate
                    } else {
                        super::ast::UnaryPrimitive::BitwiseNot
                    },
                    operand: Box::new(operand),
                },
                ty: source.ty.clone(),
                span: source.span,
            },
            UnaryOperator::SymbolLength => unreachable!("symbol length has a dedicated node"),
        }
    }

    fn lower_binary_value(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
        source: &checked::Expression,
    ) -> Expression {
        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
            && left.ty == super::bool::bool_type()
        {
            let left_id = self.temporary();
            let right_id = self.temporary();
            let equal = operator == BinaryOperator::Equal;
            let compare_right = |lowerer: &Lowerer, zero, one| {
                lowerer.case(
                    lowerer.reference(right_id, super::bool::bool_type(), source.span),
                    vec![
                        lowerer.wildcard_arm(0, lowerer.bool_value(zero, source.span), source.span),
                        lowerer.wildcard_arm(1, lowerer.bool_value(one, source.span), source.span),
                    ],
                    super::bool::bool_type(),
                    source.span,
                )
            };
            let comparison = self.case(
                self.reference(left_id, super::bool::bool_type(), source.span),
                vec![
                    self.wildcard_arm(0, compare_right(self, equal, !equal), source.span),
                    self.wildcard_arm(1, compare_right(self, !equal, equal), source.span),
                ],
                super::bool::bool_type(),
                source.span,
            );
            let bind = |id, value: Expression, body: Expression| Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(Binding {
                        pattern: Pattern::Binding {
                            id,
                            ty: super::bool::bool_type(),
                        },
                        value,
                        span: source.span,
                    }),
                    body: Box::new(body),
                },
                ty: super::bool::bool_type(),
                span: source.span,
            };
            return bind(left_id, left, bind(right_id, right, comparison));
        }
        Expression {
            kind: ExpressionKind::PrimitiveBinary {
                operator: super::primitive::lower_binary_primitive(operator),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: source.ty.clone(),
            span: source.span,
        }
    }
}

fn contains_control(value: &checked::Expression) -> bool {
    use checked::ExpressionKind;
    match &value.kind {
        ExpressionKind::Parenthesized(inner) => contains_control(inner),
        ExpressionKind::Product(elements) => elements.iter().any(contains_control),
        ExpressionKind::Call { callee, argument } => {
            contains_control(callee) || contains_control(argument)
        }
        ExpressionKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_control(condition)
                || block_contains_control(then_branch)
                || block_contains_control(else_branch)
        }
        ExpressionKind::Unary { operand, .. }
        | ExpressionKind::SymbolLength { value: operand }
        | ExpressionKind::NumericConversion { value: operand }
        | ExpressionKind::SumInjection { value: operand, .. } => contains_control(operand),
        ExpressionKind::Binary { left, right, .. } => {
            contains_control(left) || contains_control(right)
        }
        ExpressionKind::SymbolAt { argument } | ExpressionKind::Memory { argument, .. } => {
            contains_control(argument)
        }
        ExpressionKind::SumElimination {
            scrutinee,
            continuations,
        } => contains_control(scrutinee) || continuations.iter().any(contains_control),
        ExpressionKind::Lambda(_)
        | ExpressionKind::Reference(_)
        | ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Symbol(_)
        | ExpressionKind::StorageSize(_)
        | ExpressionKind::Unit
        | ExpressionKind::MemoryFunction { .. }
        | ExpressionKind::InjectionConstructor { .. } => false,
    }
}

fn block_contains_control(block: &checked::ExpressionBlock) -> bool {
    block.items.iter().any(|item| match item {
        checked::BodyItem::Binding(binding) => contains_control(&binding.value),
        checked::BodyItem::Expression(value) => contains_control(value),
    }) || match block.result.as_ref() {
        checked::Completion::Value(value) => contains_control(value),
        checked::Completion::Abrupt(_) => true,
    }
}
