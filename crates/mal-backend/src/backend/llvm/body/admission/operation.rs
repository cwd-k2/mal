//! Admission of each operation and atom: the runtime storage and canonical layouts they need on this target.

use super::layout::admit_canonical_layout;
use super::*;

pub(super) fn admit_operation<'a>(
    operation: &'a Operation,
    result_type: &Type,
    span: mal_syntax::source::Span,
    maximum: u128,
    layouts: SourceLayouts,
    types: &Types,
    blocks: &mut Vec<&'a Block>,
) -> Result<(), Diagnostic> {
    match operation {
        Operation::Atom(value)
        | Operation::Goto { value, .. }
        | Operation::ExternalCall {
            argument: value, ..
        }
        | Operation::NumericConversion { operand: value }
        | Operation::PrimitiveUnary { operand: value, .. } => admit_atom(value, maximum)?,
        Operation::SumInjection { value, .. } => {
            admit_runtime_storage(result_type, span, types, maximum, "sum temporary")?;
            admit_atom(value, maximum)?;
        }
        Operation::MakeClosure { captures, .. } => {
            let environment =
                Type::Product(captures.iter().map(|capture| capture.ty.clone()).collect());
            admit_runtime_storage(&environment, span, types, maximum, "closure environment")?;
            for capture in captures {
                admit_atom(capture, maximum)?;
            }
        }
        Operation::Product(captures)
        | Operation::Symbol {
            operands: captures, ..
        } => {
            for capture in captures {
                admit_atom(capture, maximum)?;
            }
        }
        Operation::Memory {
            primitive,
            operands,
        } => {
            let element = match primitive {
                mal_frontend::check::ast::MemoryPrimitive::BufferFromAddress => {
                    if let Type::Buffer(element) = result_type {
                        Some(element.as_ref())
                    } else {
                        None
                    }
                }
                mal_frontend::check::ast::MemoryPrimitive::BufferIntoAddress => {
                    operands.first().and_then(|operand| match &operand.ty {
                        Type::Buffer(element) => Some(element.as_ref()),
                        _ => None,
                    })
                }
                _ => None,
            };
            if let Some(element) = element {
                admit_canonical_layout(element, span, layouts, maximum)?;
            }
            for operand in operands {
                admit_atom(operand, maximum)?;
            }
        }
        Operation::Buffer {
            operation,
            element,
            operands,
        } => {
            let stride = layouts
                .layout(element)
                .map(|layout| layout.stride)
                .or_else(|| types.value(element).map(|value| value.size));
            if let Some(stride) = stride
                && stride as u128 > maximum
            {
                return Err(Diagnostic::error(
                    "Buffer element layout is not representable for the target",
                )
                .with_primary(
                    span,
                    format!(
                        "this type has a {stride}-byte stride, exceeding the target maximum {maximum}"
                    ),
                ));
            }
            if matches!(
                operation,
                crate::core::ast::BufferOperation::New
                    | crate::core::ast::BufferOperation::Get
                    | crate::core::ast::BufferOperation::Put
                    | crate::core::ast::BufferOperation::Fill
            ) {
                admit_runtime_storage(element, span, types, maximum, "Buffer element temporary")?;
            }
            for operand in operands {
                admit_atom(operand, maximum)?;
            }
        }
        Operation::Call { callee, argument } => {
            admit_atom(callee, maximum)?;
            admit_atom(argument, maximum)?;
        }
        Operation::Case { scrutinee, arms } => {
            admit_runtime_storage(&scrutinee.ty, span, types, maximum, "sum temporary")?;
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

pub(super) fn admit_atom(atom: &Atom, maximum: u128) -> Result<(), Diagnostic> {
    if let (Type::Symbol, AtomKind::Symbol(bytes)) = (&atom.ty, &atom.kind) {
        let storage = super::super::symbol::literal_storage_size(bytes.len());
        if storage.is_some_and(|storage| storage as u128 <= maximum) {
            return Ok(());
        }
        return Err(
            Diagnostic::error("Symbol literal storage is not representable for the target")
                .with_primary(
                    atom.span,
                    storage.map_or_else(
                        || "this Symbol literal's storage exceeds the target object-size range"
                            .into(),
                        |storage| {
                            format!(
                                "this Symbol literal needs {storage} bytes, exceeding the target maximum {maximum}"
                            )
                        },
                    ),
                ),
        );
    }
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
                mal_frontend::check::type_name(&atom.ty)
            ),
        ),
    )
}
