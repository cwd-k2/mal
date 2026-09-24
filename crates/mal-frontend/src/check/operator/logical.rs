use crate::resolve::ast as resolved;
use mal_syntax::ast::{BinaryOperator, Node};
use mal_syntax::source::Span;

use super::super::ast::{Completion, Expression, ExpressionBlock, ExpressionKind, Type};
use super::super::types::bool_type;
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_logical(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = bool_type();
        let left = self.check_before(left, Some(&ty), right.span)?;
        self.check_logical_after_left(operator, left, right, span)
    }

    pub(super) fn check_logical_after_left(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let ty = bool_type();
        match self.check_expression(right, Some(&ty)) {
            Ok(right) => Ok(Expression {
                kind: ExpressionKind::Binary {
                    operator: operator.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                ty,
                span,
            }),
            Err(CheckFailure::Abrupt(abrupt)) => {
                let constant = |index| Expression {
                    kind: ExpressionKind::SumInjection {
                        index,
                        value: Box::new(Expression {
                            kind: ExpressionKind::Unit,
                            ty: Type::Unit,
                            span,
                        }),
                    },
                    ty: ty.clone(),
                    span,
                };
                let abrupt = Completion::Abrupt(*abrupt);
                let (otherwise, then) = if operator.kind == BinaryOperator::LogicalAnd {
                    (Completion::Value(constant(0)), abrupt)
                } else {
                    (abrupt, Completion::Value(constant(1)))
                };
                Ok(Expression {
                    kind: ExpressionKind::If {
                        condition: Box::new(left),
                        then_branch: ExpressionBlock {
                            items: Vec::new(),
                            result: Box::new(then),
                            span,
                        },
                        else_branch: ExpressionBlock {
                            items: Vec::new(),
                            result: Box::new(otherwise),
                            span,
                        },
                    },
                    ty,
                    span,
                })
            }
            Err(error) => Err(error),
        }
    }
}
