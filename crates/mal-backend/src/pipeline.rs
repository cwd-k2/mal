use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{SourceFile, SourceGraph};

pub use crate::backend::artifact::{LlvmArtifacts, RuntimeSource};
pub use crate::backend::c::{GENERATED_HEADER_NAME, is_valid_header_name};
pub use crate::backend::llvm::{Error as BackendError, Target};

pub fn emit_header(source: &SourceFile) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    Ok(crate::backend::c::emit_header(&interface))
}

pub fn emit_header_graph(graph: &SourceGraph) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    Ok(crate::backend::c::emit_header(&interface))
}

pub fn emit_host(source: &SourceFile, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    crate::backend::c::emit_host(&interface, header_name)
}

pub fn emit_host_graph(graph: &SourceGraph, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    crate::backend::c::emit_host(&interface, header_name)
}

fn lower_interface(source: &SourceFile) -> Result<crate::core::ast::ProgramInterface, Diagnostic> {
    let checked = mal_frontend::analysis::check(source)?;
    Ok(crate::core::lower_interface(&checked))
}

fn lower_graph_interface(
    graph: &SourceGraph,
) -> Result<crate::core::ast::ProgramInterface, Diagnostic> {
    let checked = mal_frontend::analysis::check_graph(graph)?;
    Ok(crate::core::lower_interface(&checked))
}

fn lower_graph_execution(
    graph: &SourceGraph,
    optimizations: crate::execution::OptimizationSet,
) -> Result<crate::execution::Program, Diagnostic> {
    let checked = mal_frontend::analysis::check_graph(graph)?;
    let specialized = mal_frontend::check::specialize(checked)?;
    let core = crate::core::lower(&specialized);
    let anf = crate::anf::lower(&core);
    Ok(crate::execution::lower(
        crate::closure::convert(&anf),
        optimizations,
    ))
}

/// Optimization selection for generated programs. `Baseline` enables no optional technique.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Optimization {
    Baseline,
    Production,
}

/// Generates the LLVM module, C shim, public header, and runtime sources for a checked program.
pub fn generate(
    graph: &SourceGraph,
    optimization: Optimization,
    target: Target<'_>,
) -> Result<LlvmArtifacts, GenerateError> {
    let (execution_optimizations, llvm_optimizations) = match optimization {
        Optimization::Baseline => (
            crate::execution::OptimizationSet::none(),
            crate::backend::llvm::OptimizationSet::none(),
        ),
        Optimization::Production => (
            crate::execution::OptimizationSet::production(),
            crate::backend::llvm::OptimizationSet::production(),
        ),
    };
    let execution =
        lower_graph_execution(graph, execution_optimizations).map_err(GenerateError::Diagnostic)?;
    crate::backend::llvm::generate(&execution, target, llvm_optimizations)
        .map_err(GenerateError::Backend)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenerateError {
    /// The program was rejected while lowering; render it against the source graph.
    Diagnostic(Diagnostic),
    Backend(BackendError),
}
