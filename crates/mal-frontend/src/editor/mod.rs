use crate::resolve::ast as resolved;
use mal_syntax::diagnostic::Diagnostic;
use std::collections::HashSet;

use mal_syntax::source::{FileId, SourceFile, SourceGraph, Span};

mod documentation;
mod index;
mod syntax;

pub use documentation::declaration_documentation;
pub use syntax::{SyntaxDocument, SyntaxRequirement, SyntaxToken};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SymbolId {
    Type(resolved::TypeId),
    Value(resolved::ValueId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolKind {
    Type,
    Value,
    Function,
    Parameter,
    /// A name introduced by a direct result block; applying it leaves that block.
    ResultBinder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrenceRole {
    Declaration,
    Reference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Occurrence {
    pub id: SymbolId,
    pub name: String,
    pub span: Span,
    pub kind: SymbolKind,
    pub role: OccurrenceRole,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub declaration_span: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hover<'a> {
    pub span: Span,
    pub ty: &'a str,
    pub occurrence: Option<&'a Occurrence>,
}

#[derive(Clone, Debug)]
pub struct SemanticDocument {
    file: Option<FileId>,
    occurrences: Vec<Occurrence>,
    typed_regions: Vec<(Span, String)>,
    exits: Vec<Span>,
    document_symbols: Vec<Symbol>,
    completions: Vec<Symbol>,
}

/// Builds the semantic index of one file. Fails with the first diagnostic when the file does not parse, resolve, or check.
pub fn analyze(source: &SourceFile) -> Result<SemanticDocument, Diagnostic> {
    let analysis = crate::analysis::analyze(source)?;
    Ok(from_analysis_for_file(&analysis, source.id()))
}

/// Token-level information that needs only lexing, so it stays available while the file does not type-check.
pub fn analyze_syntax(source: &SourceFile) -> Result<SyntaxDocument, Diagnostic> {
    syntax::analyze(source)
}

pub fn from_analysis(analysis: &crate::analysis::Analysis) -> SemanticDocument {
    index::build(&analysis.resolved, &analysis.checked, None, None)
}

pub fn from_analysis_for_file(
    analysis: &crate::analysis::Analysis,
    file: FileId,
) -> SemanticDocument {
    index::build(&analysis.resolved, &analysis.checked, Some(file), None)
}

/// Builds the index of `file` within a multi-file analysis. Only names declared in `file` and in the files it requires
/// directly are visible.
pub fn from_graph_analysis(
    graph: &SourceGraph,
    analysis: &crate::analysis::Analysis,
    file: FileId,
) -> SemanticDocument {
    let visible = graph
        .requirements(file)
        .iter()
        .map(|requirement| requirement.target)
        .chain(std::iter::once(file))
        .collect::<HashSet<_>>();
    index::build(
        &analysis.resolved,
        &analysis.checked,
        Some(file),
        Some(&visible),
    )
}

impl SemanticDocument {
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }

    pub fn document_occurrences(&self) -> impl Iterator<Item = &Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| self.in_document(occurrence.span))
    }

    /// Where control leaves the enclosing block: the branches and continuations that leave when another one
    /// continues, or a whole choice when every one of them leaves. Positions to annotate are the span ends.
    pub fn exits(&self) -> impl Iterator<Item = Span> + '_ {
        self.exits
            .iter()
            .copied()
            .filter(|span| self.in_document(*span))
    }

    pub fn occurrence_at(&self, byte_offset: usize) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| {
                self.in_document(occurrence.span) && contains(occurrence.span, byte_offset)
            })
            .min_by_key(|occurrence| occurrence.span.end() - occurrence.span.start())
    }

    pub fn hover_at(&self, byte_offset: usize) -> Option<Hover<'_>> {
        if let Some(occurrence) = self.occurrence_at(byte_offset)
            && let Some(ty) = occurrence.detail.as_deref()
        {
            return Some(Hover {
                span: occurrence.span,
                ty,
                occurrence: Some(occurrence),
            });
        }
        self.typed_regions
            .iter()
            .filter(|(span, _)| self.in_document(*span) && contains(*span, byte_offset))
            .min_by_key(|(span, _)| span.end() - span.start())
            .map(|(span, ty)| Hover {
                span: *span,
                ty,
                occurrence: None,
            })
    }

    pub fn definition(&self, id: SymbolId) -> Option<&Occurrence> {
        self.occurrences.iter().find(|occurrence| {
            occurrence.id == id && occurrence.role == OccurrenceRole::Declaration
        })
    }

    pub fn references(&self, id: SymbolId, include_declaration: bool) -> Vec<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| {
                occurrence.id == id
                    && (include_declaration || occurrence.role == OccurrenceRole::Reference)
            })
            .collect()
    }

    pub fn rename_spans(&self, byte_offset: usize) -> Option<Vec<Span>> {
        let occurrence = self.occurrence_at(byte_offset)?;
        self.definition(occurrence.id)?;
        Some(
            self.references(occurrence.id, true)
                .into_iter()
                .map(|occurrence| occurrence.span)
                .collect(),
        )
    }

    pub fn document_symbols(&self) -> &[Symbol] {
        &self.document_symbols
    }

    pub fn completions(&self) -> &[Symbol] {
        &self.completions
    }

    fn in_document(&self, span: Span) -> bool {
        self.file.is_none_or(|file| span.file() == file)
    }
}

fn contains(span: Span, byte_offset: usize) -> bool {
    span.start() <= byte_offset && byte_offset < span.end()
}
