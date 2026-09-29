use crate::closure::ast::{Atom, Block, Operation, Program};

pub(in crate::call_pattern::parameter_lift) fn for_each_block<'a>(
    program: &'a Program,
    visit: &mut impl FnMut(&'a Block),
) {
    fn walk<'a>(block: &'a Block, visit: &mut impl FnMut(&'a Block)) {
        visit(block);
        for binding in &block.bindings {
            match &binding.operation {
                Operation::Case { arms, .. } => {
                    for arm in arms {
                        walk(&arm.value, visit);
                    }
                }
                Operation::PrimitiveBranch {
                    otherwise, then, ..
                } => {
                    walk(otherwise, visit);
                    walk(then, visit);
                }
                _ => {}
            }
        }
    }
    for binding in &program.bindings {
        walk(&binding.value, visit);
    }
    for function in &program.functions {
        walk(&function.body, visit);
        for join in &function.joins {
            walk(&join.body, visit);
        }
    }
}

pub(in crate::call_pattern::parameter_lift) fn operation_atoms(
    operation: &Operation,
    visit: &mut impl FnMut(&Atom),
) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Symbol {
            operands: captures, ..
        }
        | Operation::Buffer {
            operands: captures, ..
        } => captures.iter().for_each(visit),
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
