//! Frontend entry points that compose parsing, resolution, checking, and editor-ready outcomes.

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{SourceFile, SourceGraph};

/// The result of analyzing a program: the resolved program keeps source identities for editors, and the checked program is
/// what lowering consumes.
pub struct Analysis {
    /// Identity-annotated source structure retained for navigation and editor queries.
    pub resolved: crate::resolve::ast::Program,
    /// Canonically typed program consumed by specialization and lowering.
    pub checked: crate::check::ast::Program,
}

/// Parses, resolves, and checks one file. `require` declarations are not followed; use `analyze_graph` for several files.
pub fn analyze(source: &SourceFile) -> Result<Analysis, Diagnostic> {
    let parsed = mal_syntax::parser::parse(source)?;
    let resolved = crate::resolve::resolve(&parsed)?;
    let checked = crate::check::check(&resolved)?;
    Ok(Analysis { resolved, checked })
}

/// Parses every file of `graph`, resolves them together, and checks the whole program.
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

/// The source error that specialization reports for an executable program, such as a reached operation key without an
/// implementation. A program without `main` is a library and is not specialized, so it has none.
pub fn specialization_error(checked: &crate::check::ast::Program) -> Option<Diagnostic> {
    checked.entry?;
    crate::check::specialize(checked.clone()).err()
}

/// `analyze` reduced to the checked program. A program without `main` checks; only specialization needs an entry point.
pub fn check(source: &SourceFile) -> Result<crate::check::ast::Program, Diagnostic> {
    analyze(source).map(|analysis| analysis.checked)
}

/// `analyze_graph` reduced to the checked program.
pub fn check_graph(graph: &SourceGraph) -> Result<crate::check::ast::Program, Diagnostic> {
    analyze_graph(graph).map(|analysis| analysis.checked)
}
