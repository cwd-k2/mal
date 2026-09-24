use crate::ast::{Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::{CheckResult, Checker};

mod access;
mod intrinsic;

impl Checker {
    pub(super) fn check_memory_unary(
        &mut self,
        operator: UnaryOperator,
        operand: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let operand = self.check_expression(operand, None)?;
        let (primitive, ty) = match (operator, &operand.ty) {
            (UnaryOperator::Star, Type::Buffer(element)) if **element == Type::UInt8 => {
                (MemoryPrimitive::BufferToSymbol, Type::Symbol)
            }
            (UnaryOperator::Star, Type::Symbol) => (
                MemoryPrimitive::SymbolToBuffer,
                Type::Buffer(Type::UInt8.into()),
            ),
            _ => {
                return Err(
                    Diagnostic::error("memory operator is not defined for this type")
                        .with_primary(operand.span, "use Buffer<UInt8> or Symbol")
                        .into(),
                );
            }
        };
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                operands: vec![operand],
            },
            ty,
            span,
        })
    }
}
