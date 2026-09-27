use crate::closure::ast::{Atom, Block, Operation};

pub(super) fn for_nested_blocks(operation: &Operation, mut visit: impl FnMut(&Block)) {
    match operation {
        Operation::Case { arms, .. } => {
            for arm in arms {
                visit(&arm.value);
            }
        }
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            visit(otherwise);
            visit(then);
        }
        _ => {}
    }
}

pub(super) fn for_atoms(operation: &Operation, mut visit: impl FnMut(&Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::SymbolLength { value: atom }
        | Operation::SymbolAt { argument: atom }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Buffer {
            operands: captures, ..
        } => {
            for atom in captures {
                visit(atom);
            }
        }
        Operation::Call { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Operation::Case { scrutinee, .. } => visit(scrutinee),
        Operation::PrimitiveBranch { left, right, .. }
        | Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}

pub(super) fn for_atoms_mut(operation: &mut Operation, mut visit: impl FnMut(&mut Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::SymbolLength { value: atom }
        | Operation::SymbolAt { argument: atom }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Buffer {
            operands: captures, ..
        } => {
            for atom in captures {
                visit(atom);
            }
        }
        Operation::Call { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Operation::Case { scrutinee, .. } => visit(scrutinee),
        Operation::PrimitiveBranch { left, right, .. }
        | Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}
