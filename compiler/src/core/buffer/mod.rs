use super::Lowerer;
use super::ast::{BufferOperation, Expression, ExpressionKind};
use crate::check::ast as checked;

impl Lowerer {
    pub(super) fn lower_buffer_operation(
        &mut self,
        primitive: checked::MemoryPrimitive,
        operands: &[checked::Expression],
        expression: &checked::Expression,
    ) -> Expression {
        if primitive == checked::MemoryPrimitive::BufferMake {
            let [capacity] = operands else {
                unreachable!("checked make has one capacity operand")
            };
            let checked::Type::Buffer(element) = &expression.ty else {
                unreachable!("checked make returns Buffer")
            };
            return Expression {
                kind: ExpressionKind::Buffer {
                    operation: BufferOperation::Make,
                    element: element.as_ref().clone(),
                    argument: Box::new(self.lower_expression(capacity)),
                },
                ty: expression.ty.clone(),
                span: expression.span,
            };
        }

        let [buffer, rest @ ..] = operands else {
            unreachable!("checked Buffer operation has a receiver")
        };
        let checked::Type::Buffer(element) = &buffer.ty else {
            unreachable!("checked Buffer operation has a Buffer receiver")
        };
        let buffer = self.lower_expression(buffer);
        let (operation, argument) = match (primitive, rest) {
            (checked::MemoryPrimitive::BufferNew, [value]) => {
                let value = self.lower_expression(value);
                (
                    BufferOperation::New,
                    self.product(vec![buffer, value], expression.span),
                )
            }
            (checked::MemoryPrimitive::BufferGet, [index]) => {
                let index = self.lower_expression(index);
                (
                    BufferOperation::Get,
                    self.product(vec![buffer, index], expression.span),
                )
            }
            (checked::MemoryPrimitive::BufferPut, [index, value]) => {
                let index = self.lower_expression(index);
                let value = self.lower_expression(value);
                let put = self.product(vec![index, value], expression.span);
                (
                    BufferOperation::Put,
                    self.product(vec![buffer, put], expression.span),
                )
            }
            _ => unreachable!("checked Buffer operation has valid operands"),
        };
        Expression {
            kind: ExpressionKind::Buffer {
                operation,
                element: element.as_ref().clone(),
                argument: Box::new(argument),
            },
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }

    fn product(&self, elements: Vec<Expression>, span: crate::source::Span) -> Expression {
        Expression {
            ty: checked::Type::Product(
                elements
                    .iter()
                    .map(|element| element.ty.clone())
                    .collect::<Vec<_>>()
                    .into(),
            ),
            kind: ExpressionKind::Product(elements),
            span,
        }
    }
}
