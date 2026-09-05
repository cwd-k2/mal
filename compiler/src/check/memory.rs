use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, LOAD_INT64_VALUE, LOAD_UINT8_VALUE, OFFSET_VALUE, STORE_INT64_VALUE,
    STORE_UINT8_VALUE, ValueId,
};
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};

impl Checker {
    pub(super) fn check_memory_call(
        &mut self,
        primitive: ValueId,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> Option<Result<Expression, Diagnostic>> {
        let (operation, parameter, result) = match primitive {
            OFFSET_VALUE => (
                MemoryPrimitive::Offset,
                Type::Product(vec![Type::Ptr, Type::UInt64]),
                Type::Ptr,
            ),
            LOAD_INT64_VALUE => (MemoryPrimitive::LoadInt64, Type::Ptr, Type::Int64),
            STORE_INT64_VALUE => (
                MemoryPrimitive::StoreInt64,
                Type::Product(vec![Type::Ptr, Type::Int64]),
                Type::Unit,
            ),
            LOAD_UINT8_VALUE => (MemoryPrimitive::LoadUInt8, Type::Ptr, Type::UInt8),
            STORE_UINT8_VALUE => (
                MemoryPrimitive::StoreUInt8,
                Type::Product(vec![Type::Ptr, Type::UInt8]),
                Type::Unit,
            ),
            _ => return None,
        };
        Some(
            self.check_argument(arguments, &parameter, span)
                .map(|argument| Expression {
                    kind: ExpressionKind::Memory {
                        primitive: operation,
                        argument: Box::new(argument),
                    },
                    ty: result,
                    span,
                }),
        )
    }
}
