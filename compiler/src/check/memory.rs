use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, LOAD_FLOAT32_VALUE, LOAD_FLOAT64_VALUE, LOAD_INT8_VALUE, LOAD_INT16_VALUE,
    LOAD_INT32_VALUE, LOAD_INT64_VALUE, LOAD_PTR_VALUE, LOAD_SYMBOL_VALUE, LOAD_UINT8_VALUE,
    LOAD_UINT16_VALUE, LOAD_UINT32_VALUE, LOAD_UINT64_VALUE, STORE_FLOAT32_VALUE,
    STORE_FLOAT64_VALUE, STORE_INT8_VALUE, STORE_INT16_VALUE, STORE_INT32_VALUE, STORE_INT64_VALUE,
    STORE_PTR_VALUE, STORE_SYMBOL_VALUE, STORE_UINT8_VALUE, STORE_UINT16_VALUE, STORE_UINT32_VALUE,
    STORE_UINT64_VALUE, ValueId,
};
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, MemoryPrimitive, MemoryScalar};

pub(super) fn memory_primitive(value: ValueId) -> Option<MemoryPrimitive> {
    Some(match value {
        LOAD_INT8_VALUE => MemoryPrimitive::Load(MemoryScalar::Int8),
        STORE_INT8_VALUE => MemoryPrimitive::Store(MemoryScalar::Int8),
        LOAD_INT16_VALUE => MemoryPrimitive::Load(MemoryScalar::Int16),
        STORE_INT16_VALUE => MemoryPrimitive::Store(MemoryScalar::Int16),
        LOAD_INT32_VALUE => MemoryPrimitive::Load(MemoryScalar::Int32),
        STORE_INT32_VALUE => MemoryPrimitive::Store(MemoryScalar::Int32),
        LOAD_INT64_VALUE => MemoryPrimitive::Load(MemoryScalar::Int64),
        STORE_INT64_VALUE => MemoryPrimitive::Store(MemoryScalar::Int64),
        LOAD_UINT8_VALUE => MemoryPrimitive::Load(MemoryScalar::UInt8),
        STORE_UINT8_VALUE => MemoryPrimitive::Store(MemoryScalar::UInt8),
        LOAD_UINT16_VALUE => MemoryPrimitive::Load(MemoryScalar::UInt16),
        STORE_UINT16_VALUE => MemoryPrimitive::Store(MemoryScalar::UInt16),
        LOAD_UINT32_VALUE => MemoryPrimitive::Load(MemoryScalar::UInt32),
        STORE_UINT32_VALUE => MemoryPrimitive::Store(MemoryScalar::UInt32),
        LOAD_UINT64_VALUE => MemoryPrimitive::Load(MemoryScalar::UInt64),
        STORE_UINT64_VALUE => MemoryPrimitive::Store(MemoryScalar::UInt64),
        LOAD_FLOAT32_VALUE => MemoryPrimitive::Load(MemoryScalar::Float32),
        STORE_FLOAT32_VALUE => MemoryPrimitive::Store(MemoryScalar::Float32),
        LOAD_FLOAT64_VALUE => MemoryPrimitive::Load(MemoryScalar::Float64),
        STORE_FLOAT64_VALUE => MemoryPrimitive::Store(MemoryScalar::Float64),
        LOAD_PTR_VALUE => MemoryPrimitive::LoadPtr,
        STORE_PTR_VALUE => MemoryPrimitive::StorePtr,
        LOAD_SYMBOL_VALUE => MemoryPrimitive::LoadSymbol,
        STORE_SYMBOL_VALUE => MemoryPrimitive::StoreSymbol,
        _ => return None,
    })
}

impl Checker {
    pub(super) fn check_memory_call(
        &mut self,
        primitive: ValueId,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> Option<Result<Expression, Diagnostic>> {
        let operation = memory_primitive(primitive)?;
        let (parameter, result) = operation.signature();
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
