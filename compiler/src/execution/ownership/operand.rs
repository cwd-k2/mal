use crate::closure::ast::Atom;
use crate::control::ast::{Operation, Terminator};

use super::identity::{BindingOperand, TerminatorOperand, UseEffect};

pub(super) fn terminator_argument(terminator: &Terminator) -> Option<&Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
}

pub(super) fn binding_operands(operation: &Operation) -> Vec<(BindingOperand, &Atom, bool)> {
    match operation {
        Operation::Atom(atom) => vec![(BindingOperand::Atom, atom, true)],
        Operation::MakeClosure { captures, .. } => captures
            .iter()
            .enumerate()
            .map(|(index, atom)| (BindingOperand::Capture(index), atom, true))
            .collect(),
        Operation::Product(elements) => elements
            .iter()
            .enumerate()
            .map(|(index, atom)| (BindingOperand::ProductElement(index), atom, true))
            .collect(),
        Operation::SumInjection { value, .. } => {
            vec![(BindingOperand::SumValue, value, true)]
        }
        Operation::SymbolLength { value } => {
            vec![(BindingOperand::SymbolLength, value, false)]
        }
        Operation::SymbolAt { argument } => {
            vec![(BindingOperand::SymbolAt, argument, false)]
        }
        Operation::Memory {
            primitive: _,
            operands,
            ..
        } => operands
            .iter()
            .enumerate()
            .map(|(index, operand)| (BindingOperand::MemoryOperand(index), operand, false))
            .collect(),
        Operation::Buffer { argument, .. } => {
            vec![(BindingOperand::BufferArgument, argument, false)]
        }
        Operation::ExternalCall { argument, .. } => {
            vec![(BindingOperand::ExternalArgument, argument, false)]
        }
        Operation::NumericConversion { operand } => {
            vec![(BindingOperand::NumericOperand, operand, false)]
        }
        Operation::PrimitiveUnary { operand, .. } => {
            vec![(BindingOperand::UnaryOperand, operand, false)]
        }
        Operation::PrimitiveBinary { left, right, .. } => vec![
            (BindingOperand::BinaryLeft, left, false),
            (BindingOperand::BinaryRight, right, false),
        ],
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
