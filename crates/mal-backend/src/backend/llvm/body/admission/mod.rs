use crate::closure::ast::{Atom, AtomKind, Block, Operation, Pattern};
use crate::execution;
use mal_frontend::check::ast::Type;
use mal_syntax::diagnostic::Diagnostic;

mod layout;
mod operation;

use layout::{admit_external_storage, admit_runtime_storage};
use operation::{admit_atom, admit_operation};

use crate::backend::llvm::TargetLayout;
use crate::backend::llvm::body::types::Types;
pub(super) fn admit(program: &execution::Program, target: TargetLayout) -> Result<(), Diagnostic> {
    let maximum = match target.index_size {
        1 => u8::MAX as u128,
        2 => u16::MAX as u128,
        4 => u32::MAX as u128,
        8 => u64::MAX as u128,
        _ => unreachable!("target layout admits only supported index widths"),
    };
    let types = Types::for_target(target);
    admit_external_storage(program, &types, maximum)?;
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
