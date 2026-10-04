//! Target admission and coordinated generation of the LLVM module, C shim, header, and runtime set.

use super::abi::Function as AbiFunction;
use super::artifact::LlvmArtifacts;
use super::c::syntax::{Declaration, Directive};
use std::fmt;

mod body;
mod host_bridge;
mod module;
mod optimization;
mod shim;
pub(in crate::backend) mod syntax;
mod target;

pub(crate) use optimization::OptimizationSet;
pub(crate) use target::{TargetLayout, parse as target_layout};

/// Target identity and LLVM data layout obtained from the same pinned Clang invocation used to compile artifacts.
pub struct Target<'a> {
    /// LLVM target triple.
    pub triple: &'a str,
    /// LLVM data-layout string whose pointer and index widths govern admission and emission.
    pub data_layout: &'a str,
}

/// Failure to admit the target or to generate artifacts from an admitted execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A target-dependent source limit was exceeded; render the diagnostic against its source graph.
    Diagnostic(mal_syntax::diagnostic::Diagnostic),
    /// The data-layout string omits or uses an unsupported default pointer layout.
    InvalidTargetDataLayout,
    /// An internal invariant failed while emitting the named part of the program; a compiler defect, not a source error.
    InconsistentExecutionPlan(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => formatter.write_str(&diagnostic.message),
            Self::InvalidTargetDataLayout => {
                formatter.write_str("target data layout does not define a supported pointer size")
            }
            Self::InconsistentExecutionPlan(phase) => {
                write!(
                    formatter,
                    "internal compiler error: the execution plan is inconsistent during {phase}"
                )
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn supports(program: &crate::execution::Program) -> bool {
    body::supports(program)
}

#[cfg(test)]
pub(crate) fn generate(
    program: &crate::execution::Program,
    target: Target<'_>,
    optimizations: OptimizationSet,
) -> Result<LlvmArtifacts, Error> {
    generate_with_header_files(
        program,
        target,
        optimizations,
        &[program.lowered.span.file()],
    )
}

pub(crate) fn generate_with_header_files(
    program: &crate::execution::Program,
    target: Target<'_>,
    optimizations: OptimizationSet,
    header_files: &[mal_syntax::source::FileId],
) -> Result<LlvmArtifacts, Error> {
    let layout = target_layout(target.data_layout).ok_or(Error::InvalidTargetDataLayout)?;
    body::admit_target(program, layout).map_err(Error::Diagnostic)?;
    let body =
        body::generate(program, layout, optimizations).map_err(Error::InconsistentExecutionPlan)?;
    let entry = AbiFunction::program_entry();
    let raw_types = crate::backend::c::RawHostTypes::new(&program.lowered.interface);
    let external_bridges = program
        .lowered
        .interface
        .externals
        .iter()
        .map(|external| {
            host_bridge::generate(external, layout, &raw_types).ok_or(
                Error::InconsistentExecutionPlan("extern bridge emission".into()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let external_declarations = external_bridges
        .iter()
        .map(|bridge| bridge.llvm_declaration.clone())
        .collect::<Vec<_>>();
    let module = module::render(&body, target, layout, external_declarations).ok_or(
        Error::InconsistentExecutionPlan("entry argument layout".into()),
    )?;
    let types = body::types::Types::for_target(layout);
    let main = shim::entry_main(&body.main_parameter, types, entry.name()).ok_or(
        Error::InconsistentExecutionPlan("process entry emission".into()),
    )?;
    let runtime =
        crate::backend::runtime::for_program(body.uses_byte_runtime || main.uses_byte_runtime);
    let mut shim = crate::backend::c::syntax::TranslationUnit::new([
        Directive::include_quoted("program.mal.h").into(),
        Directive::include_quoted("runtime.h").into(),
        Directive::include_system("string.h").into(),
    ]);
    shim.blank_line();
    shim.push(Declaration::function(entry.c_signature()));
    for bridge in external_bridges {
        shim.blank_line();
        shim.extend(bridge.c_definitions);
    }
    shim.blank_line();
    shim.push(main.definition);
    Ok(LlvmArtifacts {
        module,
        shim: shim.render(),
        header: crate::backend::c::emit_header_for_target(
            &program.lowered.interface,
            header_files,
            layout,
        ),
        runtime,
    })
}

fn function_name(id: crate::closure::ast::FunctionId) -> String {
    let crate::closure::ast::FunctionId::Lambda(id) = id;
    format!("mal_function_{}", id.0)
}

#[cfg(test)]
mod tests;
