//! Reconstruction of observable operations from concrete operands.

use crate::closure::ast::{Atom, Operation};

pub(super) fn operation(
    operation: &Operation,
    mut atom: impl FnMut(&Atom) -> Option<Atom>,
) -> Option<Operation> {
    Some(match operation {
        Operation::Symbol {
            primitive,
            operands,
        } => Operation::Symbol {
            primitive: *primitive,
            operands: operands.iter().map(&mut atom).collect::<Option<_>>()?,
        },
        Operation::Memory {
            primitive,
            operands,
        } => Operation::Memory {
            primitive: *primitive,
            operands: operands.iter().map(&mut atom).collect::<Option<_>>()?,
        },
        Operation::Buffer {
            operation,
            element,
            operands,
        } => Operation::Buffer {
            operation: *operation,
            element: element.clone(),
            operands: operands.iter().map(&mut atom).collect::<Option<_>>()?,
        },
        Operation::ExternalCall { id, argument } => Operation::ExternalCall {
            id: *id,
            argument: atom(argument)?,
        },
        Operation::NumericConversion { operand } => Operation::NumericConversion {
            operand: atom(operand)?,
        },
        Operation::SumInjection { index, value } => Operation::SumInjection {
            index: *index,
            value: atom(value)?,
        },
        Operation::PrimitiveUnary { operator, operand } => Operation::PrimitiveUnary {
            operator: *operator,
            operand: atom(operand)?,
        },
        Operation::PrimitiveBinary {
            operator,
            left,
            right,
        } => Operation::PrimitiveBinary {
            operator: *operator,
            left: atom(left)?,
            right: atom(right)?,
        },
        Operation::Atom(_)
        | Operation::Goto { .. }
        | Operation::MakeClosure { .. }
        | Operation::Call { .. }
        | Operation::Product(_)
        | Operation::Case { .. }
        | Operation::PrimitiveBranch { .. } => return None,
    })
}
