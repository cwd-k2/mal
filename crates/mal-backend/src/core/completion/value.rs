//! Values whose evaluation contains control paths, lowered with a continuation that receives each operand.

use mal_frontend::check::ast as checked;

use super::*;

use super::super::Lowerer;
use super::super::ast::{Expression, ExpressionKind};

impl Lowerer {
    pub(in crate::core::completion) fn lower_unary_value(
        &mut self,
        operator: checked::UnaryOperation,
        operand: Expression,
        source: &checked::Expression,
    ) -> Expression {
        match operator {
            checked::UnaryOperation::LogicalNot => self.case(
                operand,
                vec![
                    self.wildcard_arm(0, self.bool_value(true, source.span), source.span),
                    self.wildcard_arm(1, self.bool_value(false, source.span), source.span),
                ],
                source.ty.clone(),
                source.span,
            ),
            checked::UnaryOperation::Primitive(primitive) => Expression {
                kind: ExpressionKind::PrimitiveUnary {
                    operator: primitive,
                    operand: Box::new(operand),
                },
                ty: source.ty.clone(),
                span: source.span,
            },
        }
    }

    pub(in crate::core::completion) fn lower_binary_value(
        &mut self,
        operator: checked::BinaryPrimitive,
        left: Expression,
        right: Expression,
        source: &checked::Expression,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::PrimitiveBinary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: source.ty.clone(),
            span: source.span,
        }
    }
}

impl Lowerer {
    pub(super) fn lower_value_with(
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
                operator:
                    Node {
                        kind: checked::BinaryOperation::ShortCircuit(operator),
                        ..
                    },
                left,
                right,
            } => self.lower_logical_with(
                *operator,
                left,
                right,
                result_type,
                continuation,
                value.span,
            ),
            checked::ExpressionKind::Binary {
                operator:
                    Node {
                        kind: checked::BinaryOperation::Primitive(primitive),
                        ..
                    },
                left,
                right,
            } => {
                let mut left_next = |lowerer: &mut Lowerer, left: Expression| {
                    let mut right_next = |lowerer: &mut Lowerer, right: Expression| {
                        let expression =
                            lowerer.lower_binary_value(*primitive, left.clone(), right, value);
                        continuation(lowerer, expression)
                    };
                    lowerer.lower_value_with(right, result_type, &mut right_next)
                };
                self.lower_value_with(left, result_type, &mut left_next)
            }
            checked::ExpressionKind::BoolEquality { equal, left, right } => {
                let mut left_next = |lowerer: &mut Lowerer, left: Expression| {
                    let mut right_next = |lowerer: &mut Lowerer, right: Expression| {
                        let expression =
                            lowerer.lower_bool_equality(*equal, left.clone(), right, value.span);
                        continuation(lowerer, expression)
                    };
                    lowerer.lower_value_with(right, result_type, &mut right_next)
                };
                self.lower_value_with(left, result_type, &mut left_next)
            }
            checked::ExpressionKind::NumericConversion { value: operand }
            | checked::ExpressionKind::SumInjection { value: operand, .. } => {
                let variant = match &value.kind {
                    checked::ExpressionKind::SumInjection { index, .. } => Some(*index),
                    _ => None,
                };
                let mut next = |lowerer: &mut Lowerer, operand: Expression| {
                    let value_operand = Box::new(operand);
                    let kind = match variant {
                        Some(index) => ExpressionKind::SumInjection {
                            index,
                            value: value_operand,
                        },
                        None => ExpressionKind::NumericConversion {
                            value: value_operand,
                        },
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
            checked::ExpressionKind::SymbolOperation {
                primitive,
                operands,
            } => self.lower_values_with(
                operands,
                result_type,
                continuation,
                |operands| ExpressionKind::SymbolOperation {
                    primitive: *primitive,
                    operands,
                },
                value,
            ),
            checked::ExpressionKind::Memory {
                primitive,
                operands,
            } => self.lower_values_with(
                operands,
                result_type,
                continuation,
                |operands| super::super::buffer::memory_kind(*primitive, operands, &value.ty),
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
