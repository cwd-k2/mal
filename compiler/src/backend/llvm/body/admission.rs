use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Block, Operation};
use crate::diagnostic::Diagnostic;
use crate::execution;

use crate::backend::llvm::TargetLayout;
use crate::backend::source_layout::SourceLayouts;

pub(super) fn admit(program: &execution::Program, target: TargetLayout) -> Result<(), Diagnostic> {
    let layouts = SourceLayouts::new(target);
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
        admit_atom(&block.result, layouts, maximum)?;
        for binding in &block.bindings {
            admit_operation(&binding.operation, layouts, maximum, &mut blocks)?;
        }
    }
    Ok(())
}

fn admit_operation<'a>(
    operation: &'a Operation,
    layouts: SourceLayouts,
    maximum: u128,
    blocks: &mut Vec<&'a Block>,
) -> Result<(), Diagnostic> {
    match operation {
        Operation::Memory {
            primitive: crate::check::ast::MemoryPrimitive::Align,
            operands,
        } if !layouts.supports_alignment() => {
            let operand = operands.first().expect("checked align operand");
            return Err(
                Diagnostic::error("pointer alignment is not supported for the target")
                    .with_primary(
                        operand.span,
                        "this `!` operation requires integral pointers",
                    ),
            );
        }
        Operation::Atom(value)
        | Operation::Goto { value, .. }
        | Operation::SymbolLength { value }
        | Operation::SymbolAt { argument: value }
        | Operation::PackedBuilder {
            argument: value, ..
        }
        | Operation::ExternalCall {
            argument: value, ..
        }
        | Operation::NumericConversion { operand: value }
        | Operation::SumInjection { value, .. }
        | Operation::PrimitiveUnary { operand: value, .. } => admit_atom(value, layouts, maximum)?,
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            for capture in captures {
                admit_atom(capture, layouts, maximum)?;
            }
        }
        Operation::Memory { operands, .. } => {
            for operand in operands {
                admit_atom(operand, layouts, maximum)?;
            }
        }
        Operation::Call { callee, argument } => {
            admit_atom(callee, layouts, maximum)?;
            admit_atom(argument, layouts, maximum)?;
        }
        Operation::Case { scrutinee, arms } => {
            admit_atom(scrutinee, layouts, maximum)?;
            blocks.extend(arms.iter().map(|arm| &arm.value));
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            admit_atom(left, layouts, maximum)?;
            admit_atom(right, layouts, maximum)?;
            blocks.push(otherwise);
            blocks.push(then);
        }
        Operation::PrimitiveBinary { left, right, .. } => {
            admit_atom(left, layouts, maximum)?;
            admit_atom(right, layouts, maximum)?;
        }
    }
    Ok(())
}

fn admit_atom(atom: &Atom, layouts: SourceLayouts, maximum: u128) -> Result<(), Diagnostic> {
    let value = match (&atom.ty, &atom.kind) {
        (Type::ByteSize | Type::USize, AtomKind::Integer(value)) => u128::try_from(*value).ok(),
        (Type::ByteSize, AtomKind::StorageSize(ty)) => {
            layouts.layout(ty).map(|layout| layout.stride as u128)
        }
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
