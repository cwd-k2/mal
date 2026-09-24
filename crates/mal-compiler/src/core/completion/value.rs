use mal_frontend::check::ast as checked;
use mal_syntax::ast::{BinaryOperator, UnaryOperator};

use super::super::Lowerer;
use super::super::ast::{Binding, Expression, ExpressionKind, Pattern};

impl Lowerer {
    pub(in crate::core::completion) fn lower_unary_value(
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
                        super::super::ast::UnaryPrimitive::Negate
                    } else {
                        super::super::ast::UnaryPrimitive::BitwiseNot
                    },
                    operand: Box::new(operand),
                },
                ty: source.ty.clone(),
                span: source.span,
            },
            UnaryOperator::SymbolLength | UnaryOperator::Star => {
                unreachable!("specialized unary operation has a dedicated node")
            }
        }
    }

    pub(in crate::core::completion) fn lower_binary_value(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
        source: &checked::Expression,
    ) -> Expression {
        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
            && left.ty == super::super::bool::bool_type()
        {
            let left_id = self.temporary();
            let right_id = self.temporary();
            let equal = operator == BinaryOperator::Equal;
            let compare_right = |lowerer: &Lowerer, zero, one| {
                lowerer.case(
                    lowerer.reference(right_id, super::super::bool::bool_type(), source.span),
                    vec![
                        lowerer.wildcard_arm(0, lowerer.bool_value(zero, source.span), source.span),
                        lowerer.wildcard_arm(1, lowerer.bool_value(one, source.span), source.span),
                    ],
                    super::super::bool::bool_type(),
                    source.span,
                )
            };
            let comparison = self.case(
                self.reference(left_id, super::super::bool::bool_type(), source.span),
                vec![
                    self.wildcard_arm(0, compare_right(self, equal, !equal), source.span),
                    self.wildcard_arm(1, compare_right(self, !equal, equal), source.span),
                ],
                super::super::bool::bool_type(),
                source.span,
            );
            let bind = |id, value: Expression, body: Expression| Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(Binding {
                        pattern: Pattern::Binding {
                            id,
                            ty: super::super::bool::bool_type(),
                        },
                        value,
                        span: source.span,
                    }),
                    body: Box::new(body),
                },
                ty: super::super::bool::bool_type(),
                span: source.span,
            };
            return bind(left_id, left, bind(right_id, right, comparison));
        }
        Expression {
            kind: ExpressionKind::PrimitiveBinary {
                operator: super::super::primitive::lower_binary_primitive(operator),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: source.ty.clone(),
            span: source.span,
        }
    }
}
