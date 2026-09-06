use crate::diagnostic::Diagnostic;
use crate::source::{SourceFile, SourceGraph};

pub struct Analysis {
    pub resolved: crate::resolve::ast::Program,
    pub checked: crate::check::ast::Program,
}

pub fn analyze(source: &SourceFile) -> Result<Analysis, Diagnostic> {
    let parsed = crate::parser::parse(source)?;
    let resolved = crate::resolve::resolve(&parsed)?;
    let checked = crate::check::check(&resolved)?;
    Ok(Analysis { resolved, checked })
}

pub fn analyze_graph(graph: &SourceGraph) -> Result<Analysis, Diagnostic> {
    let parsed = graph
        .files()
        .iter()
        .map(crate::parser::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let resolved = crate::resolve::resolve_graph(graph, &parsed)?;
    let checked = crate::check::check(&resolved)?;
    Ok(Analysis { resolved, checked })
}

pub fn check(source: &SourceFile) -> Result<crate::check::ast::Program, Diagnostic> {
    analyze(source).map(|analysis| analysis.checked)
}

pub fn check_graph(graph: &SourceGraph) -> Result<crate::check::ast::Program, Diagnostic> {
    analyze_graph(graph).map(|analysis| analysis.checked)
}

pub fn emit_c(source: &SourceFile) -> Result<crate::c_emit::Output, Diagnostic> {
    let closure = lower_for_c(source)?;
    crate::c_emit::emit(&closure)
}

pub fn emit_c_graph(graph: &SourceGraph) -> Result<crate::c_emit::Output, Diagnostic> {
    let closure = lower_graph_for_c(graph)?;
    crate::c_emit::emit(&closure)
}

pub fn emit_header(source: &SourceFile) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    Ok(crate::c_emit::emit_header(&interface))
}

pub fn emit_header_graph(graph: &SourceGraph) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    Ok(crate::c_emit::emit_header(&interface))
}

pub fn emit_host(source: &SourceFile, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_interface(source)?;
    crate::c_emit::emit_host(&interface, header_name)
}

pub fn emit_host_graph(graph: &SourceGraph, header_name: &str) -> Result<String, Diagnostic> {
    let interface = lower_graph_interface(graph)?;
    crate::c_emit::emit_host(&interface, header_name)
}

fn lower_interface(source: &SourceFile) -> Result<crate::core::ast::ProgramInterface, Diagnostic> {
    let checked = check(source)?;
    Ok(crate::core::lower_interface(&checked))
}

fn lower_graph_interface(
    graph: &SourceGraph,
) -> Result<crate::core::ast::ProgramInterface, Diagnostic> {
    let checked = check_graph(graph)?;
    Ok(crate::core::lower_interface(&checked))
}

fn lower_for_c(source: &SourceFile) -> Result<crate::closure::ast::Program, Diagnostic> {
    let checked = check(source)?;
    let core = crate::core::lower(&checked);
    let anf = crate::anf::lower(&core);
    Ok(crate::closure::convert(&anf))
}

fn lower_graph_for_c(graph: &SourceGraph) -> Result<crate::closure::ast::Program, Diagnostic> {
    let checked = check_graph(graph)?;
    let core = crate::core::lower(&checked);
    let anf = crate::anf::lower(&core);
    Ok(crate::closure::convert(&anf))
}
