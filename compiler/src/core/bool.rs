use crate::ast::BinaryOperator;
use crate::check::ast as checked;
use crate::source::Span;

use super::Lowerer;
use super::ast::{Binding, Expression, ExpressionKind, Pattern, ValueId};

impl Lowerer {
    pub(super) fn lower_if(
        &mut self,
        condition: &checked::Expression,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        result_type: &checked::Type,
        span: Span,
    ) -> Expression {
        let otherwise = self.lower_body(&else_branch.items, &else_branch.result);
        let then = self.lower_body(&then_branch.items, &then_branch.result);
        if let checked::ExpressionKind::Binary {
            operator,
            left,
            right,
        } = &condition.kind
            && is_comparison(operator.kind)
            && left.ty != bool_type()
        {
            return Expression {
                kind: ExpressionKind::PrimitiveBranch {
                    operator: super::primitive::lower_binary_primitive(operator.kind),
                    left: Box::new(self.lower_expression(left)),
                    right: Box::new(self.lower_expression(right)),
                    otherwise: Box::new(otherwise),
                    then: Box::new(then),
                },
                ty: result_type.clone(),
                span,
            };
        }
        let condition = self.lower_expression(condition);
        self.case(
            condition,
            vec![
                self.wildcard_arm(0, otherwise, else_branch.span),
                self.wildcard_arm(1, then, then_branch.span),
            ],
            result_type.clone(),
            span,
        )
    }

    pub(super) fn lower_logical_not(
        &mut self,
        operand: &checked::Expression,
        span: Span,
    ) -> Expression {
        let scrutinee = self.lower_expression(operand);
        let false_value = self.bool_value(false, span);
        let true_value = self.bool_value(true, span);
        self.case(
            scrutinee,
            vec![
                self.wildcard_arm(0, true_value, span),
                self.wildcard_arm(1, false_value, span),
            ],
            bool_type(),
            span,
        )
    }

    pub(super) fn lower_short_circuit_after_left(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: &checked::Expression,
        span: Span,
    ) -> Expression {
        let right = self.lower_expression(right);
        let false_value = self.bool_value(false, span);
        let true_value = self.bool_value(true, span);
        let arms = match operator {
            BinaryOperator::LogicalAnd => vec![
                self.wildcard_arm(0, false_value, span),
                self.wildcard_arm(1, right, span),
            ],
            BinaryOperator::LogicalOr => vec![
                self.wildcard_arm(0, right, span),
                self.wildcard_arm(1, true_value, span),
            ],
            _ => unreachable!("caller restricts short-circuit operators"),
        };
        self.case(left, arms, bool_type(), span)
    }

    pub(super) fn lower_bool_equality_after_left(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: &checked::Expression,
        span: Span,
    ) -> Expression {
        let left_id = self.temporary();
        let right_id = self.temporary();
        let equal = operator == BinaryOperator::Equal;
        let right_when_false = self.bool_case_reference(right_id, equal, !equal, span);
        let right_when_true = self.bool_case_reference(right_id, !equal, equal, span);
        let comparison = self.case(
            self.reference(left_id, bool_type(), span),
            vec![
                self.wildcard_arm(0, right_when_false, span),
                self.wildcard_arm(1, right_when_true, span),
            ],
            bool_type(),
            span,
        );
        let right = self.lower_expression(right);
        let right_let = self.temporary_let(right_id, right, comparison, span);
        self.temporary_let(left_id, left, right_let, span)
    }

    fn bool_case_reference(&self, id: ValueId, zero: bool, one: bool, span: Span) -> Expression {
        self.case(
            self.reference(id, bool_type(), span),
            vec![
                self.wildcard_arm(0, self.bool_value(zero, span), span),
                self.wildcard_arm(1, self.bool_value(one, span), span),
            ],
            bool_type(),
            span,
        )
    }

    pub(super) fn temporary_let(
        &self,
        id: ValueId,
        value: Expression,
        body: Expression,
        span: Span,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: value.ty.clone(),
                    },
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty: bool_type(),
            span,
        }
    }

    pub(super) fn bool_value(&self, value: bool, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::SumInjection {
                index: usize::from(value),
                value: Box::new(Expression {
                    kind: ExpressionKind::Unit,
                    ty: checked::Type::Unit,
                    span,
                }),
            },
            ty: bool_type(),
            span,
        }
    }
}

fn is_comparison(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual
    )
}

pub(super) fn bool_type() -> checked::Type {
    checked::Type::Sum(vec![checked::Type::Unit, checked::Type::Unit].into())
}
