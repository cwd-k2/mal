use crate::closure::ast::{Atom, AtomKind, Block, Operation, Pattern};
use crate::execution;
use mal_frontend::check::ast::Type;
use mal_syntax::diagnostic::Diagnostic;

use crate::backend::llvm::TargetLayout;
use crate::backend::llvm::body::types::Types;
use crate::backend::source_layout::SourceLayouts;
pub(super) fn admit(program: &execution::Program, target: TargetLayout) -> Result<(), Diagnostic> {
    let maximum = match target.index_size {
        1 => u8::MAX as u128,
        2 => u16::MAX as u128,
        4 => u32::MAX as u128,
        8 => u64::MAX as u128,
        _ => unreachable!("target layout admits only supported index widths"),
    };
    let layouts = SourceLayouts::new(target);
    let types = Types::for_target(target);
    admit_host_memory_layouts(program, layouts, &types, maximum)?;
    admit_control_frames(program, &types, maximum)?;
    admit_runtime_slots(program, &types, maximum)?;
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
            admit_operation(
                &binding.operation,
                binding.pattern.ty(),
                binding.span,
                maximum,
                layouts,
                &types,
                &mut blocks,
            )?;
        }
    }
    Ok(())
}

fn admit_runtime_slots(
    program: &execution::Program,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    for function in &program.control.functions {
        if matches!(
            program.parameters.destination(function.id),
            Some(crate::execution::ParameterDestination::Bind(_))
        ) {
            admit_runtime_storage(
                &function.parameter.ty,
                function.parameter.span,
                types,
                maximum,
                "parameter slot",
            )?;
        }
        for site in &function.states {
            for binding in &program.control.states[site.0].bindings {
                admit_pattern_storage(&binding.pattern, binding.span, types, maximum)?;
            }
        }
    }
    Ok(())
}

fn admit_pattern_storage(
    pattern: &Pattern,
    span: mal_syntax::source::Span,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    match pattern {
        Pattern::Binding { ty, .. } => {
            admit_runtime_storage(ty, span, types, maximum, "binding slot")
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                admit_pattern_storage(element, span, types, maximum)?;
            }
            Ok(())
        }
        Pattern::Wildcard { .. } => Ok(()),
    }
}

fn admit_control_frames(
    program: &execution::Program,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    for function in &program.control.functions {
        let common_region = program
            .control_regions
            .function_region(function.id)
            .filter(|region| program.control_calls.requires_common_control(*region));
        let function_ids = common_region.map_or_else(
            || vec![function.id],
            |region| program.control_regions.functions(region).to_vec(),
        );
        let frame_sites = function_ids
            .iter()
            .filter_map(|id| {
                program
                    .control
                    .functions
                    .iter()
                    .find(|candidate| candidate.id == *id)
            })
            .flat_map(|function| function.states.iter().copied())
            .filter(|site| program.control_frames.frame(*site).is_some())
            .collect::<Vec<_>>();
        let tagged = frame_sites.len() != 1;
        for site in frame_sites {
            let frame = program
                .control_frames
                .frame(site)
                .expect("collected control frame site");
            let pass_through = super::frame::physical_frame_pass_through(frame, &program.ownership);
            let span = program.control.states[site.0].span;
            let Some(layout) =
                super::frame::FrameLayout::new(frame, types.clone(), tagged, &pass_through)
            else {
                return Err(Diagnostic::error(
                    "control frame layout is not representable for the target",
                )
                .with_primary(
                    span,
                    "this suspended call's frame exceeds the target object-size range",
                ));
            };
            if layout.size as u128 > maximum {
                return Err(Diagnostic::error(
                    "control frame layout is not representable for the target",
                )
                .with_primary(
                    span,
                    format!(
                        "this suspended call needs a {}-byte frame, exceeding the target maximum {maximum}",
                        layout.size
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn admit_host_memory_layouts(
    program: &execution::Program,
    layouts: SourceLayouts,
    types: &Types,
    maximum: u128,
) -> Result<(), Diagnostic> {
    for alias in program
        .lowered
        .interface
        .type_aliases
        .iter()
        .filter(|alias| alias.host_memory_access)
    {
        admit_canonical_layout(&alias.ty, alias.span, layouts, maximum)?;
    }
    for external in &program.lowered.interface.externals {
        for ty in [&external.parameter, &external.result] {
            if let Some(layout) = layouts.layout(ty) {
                admit_canonical_stride(layout.stride, external.span, maximum)?;
            }
            admit_runtime_storage(ty, external.span, types, maximum, "external value")?;
        }
    }
    Ok(())
}

fn admit_runtime_storage(
    ty: &Type,
    span: mal_syntax::source::Span,
    types: &Types,
    maximum: u128,
    description: &str,
) -> Result<(), Diagnostic> {
    let Some(value) = types.value(ty) else {
        return Ok(());
    };
    if value.size as u128 <= maximum {
        return Ok(());
    }
    Err(
        Diagnostic::error("runtime value layout is not representable for the target").with_primary(
            span,
            format!(
                "this {description} needs {} bytes, exceeding the target maximum {maximum}",
                value.size
            ),
        ),
    )
}

fn admit_canonical_layout(
    ty: &Type,
    span: mal_syntax::source::Span,
    layouts: SourceLayouts,
    maximum: u128,
) -> Result<(), Diagnostic> {
    let Some(layout) = layouts.layout(ty) else {
        return Err(Diagnostic::error(
            "canonical memory layout is not representable for the target",
        )
        .with_primary(
            span,
            "this type's layout exceeds the target object-size range",
        ));
    };
    admit_canonical_stride(layout.stride, span, maximum)
}

fn admit_canonical_stride(
    stride: usize,
    span: mal_syntax::source::Span,
    maximum: u128,
) -> Result<(), Diagnostic> {
    if stride as u128 <= maximum {
        return Ok(());
    }
    Err(
        Diagnostic::error("canonical memory layout is not representable for the target")
            .with_primary(
                span,
                format!(
                    "this type has a {stride}-byte stride, exceeding the target maximum {maximum}"
                ),
            ),
    )
}

fn admit_operation<'a>(
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
        | Operation::SymbolLength { value }
        | Operation::SymbolAt { argument: value }
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
        Operation::Product(captures) => {
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

fn admit_atom(atom: &Atom, maximum: u128) -> Result<(), Diagnostic> {
    if let (Type::Symbol, AtomKind::Symbol(bytes)) = (&atom.ty, &atom.kind) {
        let storage = super::symbol::literal_storage_size(bytes.len());
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
