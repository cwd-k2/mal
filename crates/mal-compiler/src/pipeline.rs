use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{SourceFile, SourceGraph};

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

pub(crate) fn lower_graph_execution(
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
