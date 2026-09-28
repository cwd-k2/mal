//! In-memory composition of frontend lowering and backend artifact generation.

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{FileId, SourceFile, SourceGraph};
use std::path::{Component, Path, PathBuf};

pub use crate::backend::artifact::{LlvmArtifacts, RuntimeSource};
pub use crate::backend::c::{
    COMMON_HEADER, COMMON_HEADER_NAME, GENERATED_HEADER_NAME, common_header, is_valid_header_name,
};
pub use crate::backend::llvm::{Error as BackendError, Target};

/// Generates the public C header from the checked host interface. No `main` is required and value bindings are not lowered.
pub fn emit_header(source: &SourceFile) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    Ok(crate::backend::c::emit_header(&interface))
}

/// `emit_header` for a program of several files.
pub fn emit_header_graph(graph: &SourceGraph) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    let interface = interface.for_file(graph.root());
    let dependencies = header_dependencies(graph, graph.root())?;
    Ok(crate::backend::c::emit_file_header(
        &interface,
        &dependencies,
    ))
}

/// Generates a host implementation template that includes `header_name` and traps in every external operation until it is
/// implemented.
pub fn emit_host(source: &SourceFile, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    crate::backend::c::emit_host(&interface, header_name)
}

/// `emit_host` for a program of several files.
pub fn emit_host_graph(graph: &SourceGraph, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    crate::backend::c::emit_host(&interface.for_file(graph.root()), header_name)
}

fn header_dependencies(graph: &SourceGraph, file: FileId) -> Result<Vec<String>, Diagnostic> {
    let source = graph
        .source(file)
        .expect("source graph requirements belong to an existing file");
    let directory = source.path().parent().unwrap_or_else(|| Path::new(""));
    graph
        .requirements(file)
        .iter()
        .map(|requirement| {
            let target = graph
                .source(requirement.target)
                .expect("source graph requirements target an existing file");
            let header = relative_path(directory, &target.path().with_extension("mal.h"));
            let Some(header) = header.to_str() else {
                return Err(
                    Diagnostic::error("required file header path is not valid UTF-8")
                        .with_primary(requirement.span, "this requirement needs a C file header"),
                );
            };
            if !crate::backend::c::is_valid_header_name(header) {
                return Err(Diagnostic::error(
                    "required file header path is not valid in a quoted C include",
                )
                .with_primary(requirement.span, "this requirement needs a C file header"));
            }
            Ok(header.to_owned())
        })
        .collect()
}

/// Computes a lexical relative path for generated headers without consulting the filesystem where they do not yet exist.
fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for component in &from[common..] {
        if !matches!(component, Component::CurDir) {
            relative.push("..");
        }
    }
    for component in &to[common..] {
        relative.push(component.as_os_str());
    }
    relative
}

fn build_header_files(
    graph: &SourceGraph,
    interface: &crate::core::ast::ProgramInterface,
) -> Vec<FileId> {
    // The umbrella header always exposes the root. Beyond it, include only files that declare reachable externs and
    // their direct source dependencies; exporting the entire requirement graph would leak unrelated file surfaces.
    let mut selected = vec![false; graph.files().len()];
    let mut expanded = vec![false; graph.files().len()];
    selected[graph.root().index() as usize] = true;
    let mut pending = interface
        .externals
        .iter()
        .map(|external| external.span.file())
        .collect::<Vec<_>>();
    while let Some(file) = pending.pop() {
        let index = file.index() as usize;
        if expanded[index] {
            continue;
        }
        expanded[index] = true;
        selected[index] = true;
        pending.extend(
            graph
                .requirements(file)
                .iter()
                .map(|requirement| requirement.target),
        );
    }
    selected
        .into_iter()
        .enumerate()
        .filter(|(_, selected)| *selected)
        .map(|(index, _)| FileId::new(index as u32))
        .collect()
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

/// Runs the only path that requires an entry point and therefore specialization before backend lowering.
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
    /// Uses no optional execution or LLVM emission techniques.
    Baseline,
    /// Uses every adopted execution and LLVM emission technique.
    Production,
}

/// Generates the LLVM module, C shim, public header, and runtime sources for a checked program.
/// The program must declare `main`. `target` names the triple and data layout the module is generated for, and every artifact must
/// be compiled for that same target.
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
    let header_files = build_header_files(graph, &execution.lowered.interface);
    crate::backend::llvm::generate_with_header_files(
        &execution,
        target,
        llvm_optimizations,
        &header_files,
    )
    .map_err(GenerateError::Backend)
}

/// Failure from either source-aware lowering or target-specific artifact generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenerateError {
    /// The program was rejected while lowering; render it against the source graph.
    Diagnostic(Diagnostic),
    /// Target admission or LLVM artifact construction failed.
    Backend(BackendError),
}
