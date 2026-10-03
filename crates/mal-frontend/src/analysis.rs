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

/// Editor analysis for the current source version. Its checked program may contain only successfully admitted
/// top-level items and is intentionally not exposed to compiler lowering or specialization.
pub struct EditorAnalysis {
    pub(crate) resolved: crate::resolve::ast::Program,
    pub(crate) checked: crate::check::ast::Program,
    check_diagnostic: Option<Diagnostic>,
}

impl EditorAnalysis {
    /// Returns the first top-level checking diagnostic, if checking recovered after an invalid item.
    pub fn check_diagnostic(&self) -> Option<&Diagnostic> {
        self.check_diagnostic.as_ref()
    }

    /// Returns a specialization diagnostic only when the complete program passed checking.
    pub fn specialization_error(&self) -> Option<Diagnostic> {
        self.check_diagnostic
            .is_none()
            .then(|| specialization_error(&self.checked))
            .flatten()
    }
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

/// Parses and resolves one file, then retains successfully checked top-level items if another item has a type error.
/// Parse, resolution, and checker-wide declaration failures are returned as errors.
pub fn analyze_for_editor(source: &SourceFile) -> Result<EditorAnalysis, Diagnostic> {
    let parsed = mal_syntax::parser::parse(source)?;
    let resolved = crate::resolve::resolve(&parsed)?;
    editor_analysis(resolved)
}

/// Parses and resolves a source graph, then retains successfully checked top-level items if another item has a type
/// error. Parse, resolution, and checker-wide declaration failures are returned as errors.
pub fn analyze_graph_for_editor(graph: &SourceGraph) -> Result<EditorAnalysis, Diagnostic> {
    let parsed = graph
        .files()
        .iter()
        .map(mal_syntax::parser::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let resolved = crate::resolve::resolve_graph(graph, &parsed)?;
    editor_analysis(resolved)
}

fn editor_analysis(resolved: crate::resolve::ast::Program) -> Result<EditorAnalysis, Diagnostic> {
    let (checked, check_diagnostic) = crate::check::check_for_editor(&resolved)?;
    Ok(EditorAnalysis {
        resolved,
        checked,
        check_diagnostic,
    })
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
