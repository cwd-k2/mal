//! Closure operations as control operations.

use super::*;

pub(super) fn lower_operation(operation: &closure::Operation) -> Operation {
    match operation {
        closure::Operation::Atom(value) => Operation::Atom(value.clone()),
        closure::Operation::MakeClosure { function, captures } => Operation::MakeClosure {
            function: *function,
            captures: captures.clone(),
        },
        closure::Operation::Symbol {
            primitive,
            operands,
        } => Operation::Symbol {
            primitive: *primitive,
            operands: operands.clone(),
        },
        closure::Operation::Memory {
            primitive,
            operands,
        } => Operation::Memory {
            primitive: *primitive,
            operands: operands.clone(),
        },
        closure::Operation::Buffer {
            operation,
            element,
            operands,
        } => Operation::Buffer {
            operation: *operation,
            element: element.clone(),
            operands: operands.clone(),
        },
        closure::Operation::ExternalCall { id, argument } => Operation::ExternalCall {
            id: *id,
            argument: argument.clone(),
        },
        closure::Operation::NumericConversion { operand } => Operation::NumericConversion {
            operand: operand.clone(),
        },
        closure::Operation::Product(elements) => Operation::Product(elements.clone()),
        closure::Operation::SumInjection { index, value } => Operation::SumInjection {
            index: *index,
            value: value.clone(),
        },
        closure::Operation::PrimitiveUnary { operator, operand } => Operation::PrimitiveUnary {
            operator: *operator,
            operand: operand.clone(),
        },
        closure::Operation::PrimitiveBinary {
            operator,
            left,
            right,
        } => Operation::PrimitiveBinary {
            operator: *operator,
            left: left.clone(),
            right: right.clone(),
        },
        closure::Operation::Call { .. }
        | closure::Operation::Goto { .. }
        | closure::Operation::Case { .. }
        | closure::Operation::PrimitiveBranch { .. } => {
            unreachable!("control operations become terminators")
        }
    }
}
