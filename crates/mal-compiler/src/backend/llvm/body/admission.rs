use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Block, Operation};
use crate::execution;
use mal_syntax::diagnostic::Diagnostic;

use crate::backend::llvm::TargetLayout;
pub(super) fn admit(program: &execution::Program, target: TargetLayout) -> Result<(), Diagnostic> {
    let maximum = match target.index_size {
        1 => u8::MAX as u128,
        2 => u16::MAX as u128,
        4 => u32::MAX as u128,
        8 => u64::MAX as u128,
        _ => unreachable!("target layout admits only supported index widths"),
    };
    let mut blocks = program
        .lowered
        .bindings
        .iter()
        .map(|binding| &binding.value)
        .chain(program.lowered.functions.iter().flat_map(|function| {
            std::iter::once(&function.body).chain(function.joins.iter().map(|join| &join.body))
        }))
        .collect::<Vec<_>>();
    while let Some(block) = blocks.pop() {
        admit_atom(&block.result, maximum)?;
        for binding in &block.bindings {
            admit_operation(&binding.operation, maximum, &mut blocks)?;
        }
    }
    Ok(())
}

fn admit_operation<'a>(
    operation: &'a Operation,
    maximum: u128,
    blocks: &mut Vec<&'a Block>,
) -> Result<(), Diagnostic> {
    match operation {
        Operation::Atom(value)
        | Operation::Goto { value, .. }
        | Operation::SymbolLength { value }
        | Operation::SymbolAt { argument: value }
        | Operation::ExternalCall {
            argument: value, ..
        }
        | Operation::NumericConversion { operand: value }
        | Operation::SumInjection { value, .. }
        | Operation::PrimitiveUnary { operand: value, .. } => admit_atom(value, maximum)?,
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for capture in captures {
                admit_atom(capture, maximum)?;
            }
        }
        Operation::Memory { operands, .. } | Operation::Buffer { operands, .. } => {
            for operand in operands {
                admit_atom(operand, maximum)?;
            }
        }
        Operation::Call { callee, argument } => {
            admit_atom(callee, maximum)?;
            admit_atom(argument, maximum)?;
        }
        Operation::Case { scrutinee, arms } => {
            admit_atom(scrutinee, maximum)?;
            blocks.extend(arms.iter().map(|arm| &arm.value));
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            admit_atom(left, maximum)?;
            admit_atom(right, maximum)?;
            blocks.push(otherwise);
            blocks.push(then);
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            admit_atom(left, maximum)?;
            admit_atom(right, maximum)?;
        }
    }
    Ok(())
}

fn admit_atom(atom: &Atom, maximum: u128) -> Result<(), Diagnostic> {
    let value = match (&atom.ty, &atom.kind) {
        (Type::ByteSize | Type::USize, AtomKind::Integer(value)) => u128::try_from(*value).ok(),
        _ => return Ok(()),
    };
    if value.is_some_and(|value| value <= maximum) {
        return Ok(());
    }
    Err(
        Diagnostic::error("value is not representable for the target").with_primary(
            atom.span,
            format!(
                "this `{}` value exceeds the target maximum {maximum}",
                crate::check::type_name(&atom.ty)
            ),
        ),
    )
}
