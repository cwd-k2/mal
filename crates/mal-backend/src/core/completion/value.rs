use mal_frontend::check::ast as checked;
use mal_syntax::ast::{BinaryOperator, UnaryOperator};

use super::super::Lowerer;
use super::super::ast::{Expression, ExpressionKind};

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
            UnaryOperator::Length | UnaryOperator::Star => {
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
