use super::ast::{BufferOperation, Expression, ExpressionKind};
use mal_frontend::check::ast as checked;

/// The core form of a checked memory primitive over operands that are already lowered. `Buffer` operations keep their
/// logical operands and element type; the other primitives keep their operands as they are. Every lowering path that
/// rebuilds a memory expression goes through this function, so no path can leave a `Buffer` primitive in the generic form.
pub(super) fn memory_kind(
    primitive: checked::MemoryPrimitive,
    operands: Vec<Expression>,
    result_type: &checked::Type,
) -> ExpressionKind {
    let operation = match primitive {
        checked::MemoryPrimitive::BufferMake => BufferOperation::Make,
        checked::MemoryPrimitive::BufferNew => BufferOperation::New,
        checked::MemoryPrimitive::BufferGet => BufferOperation::Get,
        checked::MemoryPrimitive::BufferPut => BufferOperation::Put,
        checked::MemoryPrimitive::BufferFill => BufferOperation::Fill,
        checked::MemoryPrimitive::BufferCopy => BufferOperation::Copy,
        _ => {
            return ExpressionKind::Memory {
                primitive,
                operands,
            };
        }
    };
    // `make` returns the Buffer; every other operation takes it as the first operand.
    let buffer_type = if operation == BufferOperation::Make {
        result_type
    } else {
        &operands
            .first()
            .expect("checked Buffer operation has a receiver")
            .ty
    };
    let checked::Type::Buffer(element) = buffer_type else {
        unreachable!("checked Buffer operation has a Buffer type")
    };
    ExpressionKind::Buffer {
        operation,
        element: element.as_ref().clone(),
        operands,
    }
}
