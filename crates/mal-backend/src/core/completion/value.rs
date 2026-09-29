use mal_frontend::check::ast as checked;

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
