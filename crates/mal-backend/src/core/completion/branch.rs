use mal_frontend::check::ast as checked;
use mal_syntax::ast::BinaryOperator;

use super::{Continuation, Lowerer};
use crate::core::ast::{Expression, ExpressionKind, Pattern};

impl Lowerer {
    pub(super) fn lower_sum_elimination_body(
        &mut self,
        scrutinee: &checked::Expression,
        continuations: &[checked::Expression],
        value_type: &checked::Type,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: mal_syntax::source::Span,
    ) -> Expression {
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
                            ty: value_type.clone(),
                            span: branch.span,
                        },
                    )
                };
                let branch = lowerer.lower_value_with(branch, result_type, &mut branch_next);
                arms.push(crate::core::ast::CaseArm {
                    index,
                    pattern: Pattern::Binding {
                        id: payload_id,
                        ty: member.clone(),
                    },
                    value: branch,
                    span,
                });
            }
            Expression {
                kind: ExpressionKind::Case {
                    scrutinee: Box::new(scrutinee),
                    arms,
                },
                ty: result_type.clone(),
                span,
            }
        };
        self.lower_value_with(scrutinee, result_type, &mut scrutinee_next)
    }

    pub(super) fn lower_control_if_body(
        &mut self,
        condition: &checked::Expression,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: mal_syntax::source::Span,
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

    pub(super) fn lower_logical_with(
        &mut self,
        operator: BinaryOperator,
        left: &checked::Expression,
        right: &checked::Expression,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: mal_syntax::source::Span,
    ) -> Expression {
        let value_type = checked::Type::Sum(vec![checked::Type::Unit, checked::Type::Unit].into());
        self.lower_with_join(
            &value_type,
            result_type,
            span,
            continuation,
            |lowerer, continuation| {
                lowerer.lower_logical_body(operator, left, right, result_type, continuation, span)
            },
        )
    }

    fn lower_logical_body(
        &mut self,
        operator: BinaryOperator,
        left: &checked::Expression,
        right: &checked::Expression,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: mal_syntax::source::Span,
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
