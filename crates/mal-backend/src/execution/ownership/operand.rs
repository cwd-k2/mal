use crate::closure::ast::Atom;
use crate::control::ast::{Operation, Terminator};

use super::identity::{BindingOperand, TerminatorOperand, UseEffect};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OperationEffect {
    Borrow,
    Store,
}

pub(super) fn terminator_argument(terminator: &Terminator) -> Option<&Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
}

/// Enumerates operands with their stable identity and operation-level responsibility effect.
/// Aggregate construction stores its members. `Buffer::new` and `Buffer::put` store exactly one
/// element, while the other primitive, host, memory, and `Buffer` operands remain borrowed.
pub(super) fn binding_operands(
    operation: &Operation,
) -> Vec<(BindingOperand, &Atom, OperationEffect)> {
    match operation {
        Operation::Atom(atom) => vec![(BindingOperand::Atom, atom, OperationEffect::Store)],
        Operation::MakeClosure { captures, .. } => captures
            .iter()
            .enumerate()
            .map(|(index, atom)| (BindingOperand::Capture(index), atom, OperationEffect::Store))
            .collect(),
        Operation::Product(elements) => elements
            .iter()
            .enumerate()
            .map(|(index, atom)| {
                (
                    BindingOperand::ProductElement(index),
                    atom,
                    OperationEffect::Store,
                )
            })
            .collect(),
        Operation::SumInjection { value, .. } => {
            vec![(BindingOperand::SumValue, value, OperationEffect::Store)]
        }
        Operation::Symbol { operands, .. } => operands
            .iter()
            .enumerate()
            .map(|(index, operand)| {
                (
                    BindingOperand::SymbolOperand(index),
                    operand,
                    OperationEffect::Borrow,
                )
            })
            .collect(),
        Operation::Memory {
            primitive: _,
            operands,
            ..
        } => operands
            .iter()
            .enumerate()
            .map(|(index, operand)| {
                (
                    BindingOperand::MemoryOperand(index),
                    operand,
                    OperationEffect::Borrow,
                )
            })
            .collect(),
        Operation::Buffer {
            operation,
            operands,
            ..
        } => operands
            .iter()
            .enumerate()
            .map(|(index, operand)| {
                (
                    BindingOperand::BufferOperand(index),
                    operand,
                    buffer_operand_effect(*operation, index),
                )
            })
            .collect(),
        Operation::ExternalCall { argument, .. } => {
            vec![(
                BindingOperand::ExternalArgument,
                argument,
                OperationEffect::Borrow,
            )]
        }
        Operation::NumericConversion { operand } => {
            vec![(
                BindingOperand::NumericOperand,
                operand,
                OperationEffect::Borrow,
            )]
        }
        Operation::PrimitiveUnary { operand, .. } => {
            vec![(
                BindingOperand::UnaryOperand,
                operand,
                OperationEffect::Borrow,
            )]
        }
        Operation::PrimitiveBinary { left, right, .. } => vec![
            (BindingOperand::BinaryLeft, left, OperationEffect::Borrow),
            (BindingOperand::BinaryRight, right, OperationEffect::Borrow),
        ],
    }
}

fn buffer_operand_effect(
    operation: crate::core::ast::BufferOperation,
    index: usize,
) -> OperationEffect {
    match (operation, index) {
        (crate::core::ast::BufferOperation::New, 1)
        | (crate::core::ast::BufferOperation::Put, 2) => OperationEffect::Store,
        _ => OperationEffect::Borrow,
    }
}

pub(super) fn terminator_operands<'a>(
    terminator: &'a Terminator,
    effective_argument: Option<&'a Atom>,
) -> Vec<(TerminatorOperand, &'a Atom, UseEffect)> {
    match terminator {
        Terminator::Return(atom) => {
            vec![(TerminatorOperand::Return, atom, UseEffect::Share)]
        }
        Terminator::Jump { value, .. } => {
            vec![(TerminatorOperand::JumpValue, value, UseEffect::Share)]
        }
        Terminator::Call {
            callee, argument, ..
        } => vec![
            (TerminatorOperand::CallCallee, callee, UseEffect::Borrow),
            (
                TerminatorOperand::CallArgument,
                effective_argument.unwrap_or(argument),
                UseEffect::Borrow,
            ),
        ],
        Terminator::TailCall { callee, argument } => vec![
            (TerminatorOperand::TailCallee, callee, UseEffect::Borrow),
            (
                TerminatorOperand::TailArgument,
                effective_argument.unwrap_or(argument),
                UseEffect::Borrow,
            ),
        ],
        Terminator::Case { scrutinee, .. } => vec![(
            TerminatorOperand::CaseScrutinee,
            scrutinee,
            UseEffect::Borrow,
        )],
        Terminator::PrimitiveBranch { left, right, .. } => vec![
            (TerminatorOperand::BranchLeft, left, UseEffect::Borrow),
            (TerminatorOperand::BranchRight, right, UseEffect::Borrow),
        ],
        Terminator::Goto(_) => Vec::new(),
    }
}
