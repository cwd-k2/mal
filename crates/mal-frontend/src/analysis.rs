use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{SourceFile, SourceGraph};

pub struct Analysis {
    pub resolved: crate::resolve::ast::Program,
    pub checked: crate::check::ast::Program,
}

pub fn analyze(source: &SourceFile) -> Result<Analysis, Diagnostic> {
    let parsed = mal_syntax::parser::parse(source)?;
    let resolved = crate::resolve::resolve(&parsed)?;
    let checked = crate::check::check(&resolved)?;
    Ok(Analysis { resolved, checked })
}

pub fn analyze_graph(graph: &SourceGraph) -> Result<Analysis, Diagnostic> {
    let parsed = graph
        .files()
        .iter()
        .map(mal_syntax::parser::parse)
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
