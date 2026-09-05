use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, LOAD_FLOAT32_VALUE, LOAD_FLOAT64_VALUE, LOAD_INT8_VALUE, LOAD_INT16_VALUE,
    LOAD_INT32_VALUE, LOAD_INT64_VALUE, LOAD_PTR_VALUE, LOAD_STRING_VALUE, LOAD_UINT8_VALUE,
    LOAD_UINT16_VALUE, LOAD_UINT32_VALUE, LOAD_UINT64_VALUE, OFFSET_VALUE, STORE_FLOAT32_VALUE,
    STORE_FLOAT64_VALUE, STORE_INT8_VALUE, STORE_INT16_VALUE, STORE_INT32_VALUE, STORE_INT64_VALUE,
    STORE_PTR_VALUE, STORE_STRING_VALUE, STORE_UINT8_VALUE, STORE_UINT16_VALUE, STORE_UINT32_VALUE,
    STORE_UINT64_VALUE, ValueId,
};
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, MemoryPrimitive, MemoryScalar, Type};

pub(super) fn is_memory_primitive(value: ValueId) -> bool {
    matches!(
        value,
        OFFSET_VALUE
            | LOAD_INT8_VALUE
            | STORE_INT8_VALUE
            | LOAD_INT16_VALUE
            | STORE_INT16_VALUE
            | LOAD_INT32_VALUE
            | STORE_INT32_VALUE
            | LOAD_INT64_VALUE
            | STORE_INT64_VALUE
            | LOAD_UINT8_VALUE
            | STORE_UINT8_VALUE
            | LOAD_UINT16_VALUE
            | STORE_UINT16_VALUE
            | LOAD_UINT32_VALUE
            | STORE_UINT32_VALUE
            | LOAD_UINT64_VALUE
            | STORE_UINT64_VALUE
            | LOAD_FLOAT32_VALUE
            | STORE_FLOAT32_VALUE
            | LOAD_FLOAT64_VALUE
            | STORE_FLOAT64_VALUE
            | LOAD_PTR_VALUE
            | STORE_PTR_VALUE
            | LOAD_STRING_VALUE
            | STORE_STRING_VALUE
    )
}

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
            LOAD_INT8_VALUE => load(MemoryScalar::Int8, Type::Int8),
            STORE_INT8_VALUE => store(MemoryScalar::Int8, Type::Int8),
            LOAD_INT16_VALUE => load(MemoryScalar::Int16, Type::Int16),
            STORE_INT16_VALUE => store(MemoryScalar::Int16, Type::Int16),
            LOAD_INT32_VALUE => load(MemoryScalar::Int32, Type::Int32),
            STORE_INT32_VALUE => store(MemoryScalar::Int32, Type::Int32),
            LOAD_INT64_VALUE => load(MemoryScalar::Int64, Type::Int64),
            STORE_INT64_VALUE => store(MemoryScalar::Int64, Type::Int64),
            LOAD_UINT8_VALUE => load(MemoryScalar::UInt8, Type::UInt8),
            STORE_UINT8_VALUE => store(MemoryScalar::UInt8, Type::UInt8),
            LOAD_UINT16_VALUE => load(MemoryScalar::UInt16, Type::UInt16),
            STORE_UINT16_VALUE => store(MemoryScalar::UInt16, Type::UInt16),
            LOAD_UINT32_VALUE => load(MemoryScalar::UInt32, Type::UInt32),
            STORE_UINT32_VALUE => store(MemoryScalar::UInt32, Type::UInt32),
            LOAD_UINT64_VALUE => load(MemoryScalar::UInt64, Type::UInt64),
            STORE_UINT64_VALUE => store(MemoryScalar::UInt64, Type::UInt64),
            LOAD_FLOAT32_VALUE => load(MemoryScalar::Float32, Type::Float32),
            STORE_FLOAT32_VALUE => store(MemoryScalar::Float32, Type::Float32),
            LOAD_FLOAT64_VALUE => load(MemoryScalar::Float64, Type::Float64),
            STORE_FLOAT64_VALUE => store(MemoryScalar::Float64, Type::Float64),
            LOAD_PTR_VALUE => (MemoryPrimitive::LoadPtr, Type::Ptr, Type::Ptr),
            STORE_PTR_VALUE => (
                MemoryPrimitive::StorePtr,
                Type::Product(vec![Type::Ptr, Type::Ptr]),
                Type::Unit,
            ),
            LOAD_STRING_VALUE => (MemoryPrimitive::LoadString, Type::Ptr, Type::String),
            STORE_STRING_VALUE => (
                MemoryPrimitive::StoreString,
                Type::Product(vec![Type::Ptr, Type::String]),
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

fn load(scalar: MemoryScalar, ty: Type) -> (MemoryPrimitive, Type, Type) {
    (MemoryPrimitive::Load(scalar), Type::Ptr, ty)
}

fn store(scalar: MemoryScalar, ty: Type) -> (MemoryPrimitive, Type, Type) {
    (
        MemoryPrimitive::Store(scalar),
        Type::Product(vec![Type::Ptr, ty]),
        Type::Unit,
    )
}
