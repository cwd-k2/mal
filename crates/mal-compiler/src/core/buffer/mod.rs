use super::Lowerer;
use super::ast::{BufferOperation, Expression, ExpressionKind};
use mal_frontend::check::ast as checked;

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
                    operands: vec![self.lower_expression(capacity)],
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
        let operation = match (primitive, rest) {
            (checked::MemoryPrimitive::BufferNew, [_]) => BufferOperation::New,
            (checked::MemoryPrimitive::BufferGet, [_]) => BufferOperation::Get,
            (checked::MemoryPrimitive::BufferPut, [_, _]) => BufferOperation::Put,
            (checked::MemoryPrimitive::BufferFill, [_, _, _]) => BufferOperation::Fill,
            (checked::MemoryPrimitive::BufferCopy, [_, _, _, _]) => BufferOperation::Copy,
            _ => unreachable!("checked Buffer operation has valid operands"),
        };
        Expression {
            kind: ExpressionKind::Buffer {
                operation,
                element: element.as_ref().clone(),
                operands: operands
                    .iter()
                    .map(|operand| self.lower_expression(operand))
                    .collect(),
            },
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }
}
