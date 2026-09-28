//! Syntax and semantic indexes shaped for editor queries.

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
/// A resolved identity in either the type or value namespace.
pub enum SymbolId {
    /// A type declaration or reference.
    Type(resolved::TypeId),
    /// A value declaration or reference.
    Value(resolved::ValueId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The presentation role of a name in editor features.
pub enum SymbolKind {
    /// A type, alias, parameter, or external type.
    Type,
    /// A non-callable value.
    Value,
    /// A callable value or operation.
    Function,
    /// A function or continuation parameter.
    Parameter,
    /// A name introduced by a direct result block; applying it leaves that block.
    ResultBinder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Whether a source occurrence introduces or uses an identity.
pub enum OccurrenceRole {
    /// The source occurrence that introduces the identity.
    Declaration,
    /// A use resolved to an existing declaration.
    Reference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One identity-bearing source occurrence used by navigation and hover.
pub struct Occurrence {
    /// The resolved symbol identity.
    pub id: SymbolId,
    /// The source spelling at this occurrence.
    pub name: String,
    /// The exact identifier range.
    pub span: Span,
    /// The editor presentation role.
    pub kind: SymbolKind,
    /// Whether this occurrence introduces or uses the identity.
    pub role: OccurrenceRole,
    /// A compact type or signature string for hover and completion.
    pub detail: Option<String>,
    /// User-facing declaration documentation, when available.
    pub documentation: Option<String>,
    /// The defining identifier range, absent for predefined symbols.
    pub declaration_span: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A declaration-shaped entry for document symbols or completion.
pub struct Symbol {
    /// The resolved symbol identity.
    pub id: SymbolId,
    /// The inserted or displayed name.
    pub name: String,
    /// The editor presentation role.
    pub kind: SymbolKind,
    /// A compact type or signature string.
    pub detail: Option<String>,
    /// User-facing declaration documentation, when available.
    pub documentation: Option<String>,
    /// The source declaration range, absent for predefined symbols.
    pub span: Option<Span>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The narrowest typed region selected for a hover query.
pub struct Hover<'a> {
    /// The source range to highlight.
    pub span: Span,
    /// The formatted canonical or declared type.
    pub ty: &'a str,
    /// Identity information when the region is a named occurrence.
    pub occurrence: Option<&'a Occurrence>,
}

/// A unit where control leaves the enclosing result block, and the result binders it transfers to. `targets` is empty
/// when the unit never returns normally without naming a binder, as after an empty elimination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exit {
    /// The source unit whose end should receive the exit annotation.
    pub span: Span,
    /// Result binder names that may receive control; empty means no normal return.
    pub targets: Vec<String>,
}

#[derive(Clone, Debug)]
/// Program-wide semantic index with queries optionally restricted to one document.
pub struct SemanticDocument {
    file: Option<FileId>,
    occurrences: Vec<Occurrence>,
    typed_regions: Vec<(Span, String)>,
    exits: Vec<Exit>,
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

/// Builds a whole-program semantic index, including occurrences from every source file.
pub fn from_analysis(analysis: &crate::analysis::Analysis) -> SemanticDocument {
    index::build(&analysis.resolved, &analysis.checked, None, None)
}

/// Builds an index whose document-local queries are restricted to `file` while definitions remain program-wide.
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
    /// Returns every declaration and reference retained for cross-file navigation.
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }

    /// Iterates only occurrences belonging to the document selected when this index was built.
    pub fn document_occurrences(&self) -> impl Iterator<Item = &Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| self.in_document(occurrence.span))
    }

    /// Where control leaves its result block, and to which binders: the branches and continuations that leave when
    /// another one continues, or a whole choice when every one of them leaves. A choice that only ends an already marked
    /// unit is not repeated, while an exit among the statements of a marked unit is, since control may leave there early.
    /// The positions to annotate are the span ends.
    pub fn exits(&self) -> impl Iterator<Item = &Exit> + '_ {
        self.exits.iter().filter(|exit| self.in_document(exit.span))
    }

    /// Returns the narrowest named occurrence containing the byte offset.
    pub fn occurrence_at(&self, byte_offset: usize) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| {
                self.in_document(occurrence.span) && contains(occurrence.span, byte_offset)
            })
            .min_by_key(|occurrence| occurrence.span.end() - occurrence.span.start())
    }

    /// Returns the narrowest named or typed region containing the byte offset, preferring names with declarations.
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

    /// Finds the unique declaration occurrence for a resolved identity.
    pub fn definition(&self, id: SymbolId) -> Option<&Occurrence> {
        self.occurrences.iter().find(|occurrence| {
            occurrence.id == id && occurrence.role == OccurrenceRole::Declaration
        })
    }

    /// Collects program-wide occurrences of an identity, optionally including its declaration.
    pub fn references(&self, id: SymbolId, include_declaration: bool) -> Vec<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| {
                occurrence.id == id
                    && (include_declaration || occurrence.role == OccurrenceRole::Reference)
            })
            .collect()
    }

    /// Returns every span safe to rename when the offset resolves to an identity with a source declaration.
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

    /// Returns declarations owned by the selected document in source order.
    pub fn document_symbols(&self) -> &[Symbol] {
        &self.document_symbols
    }

    /// Returns values visible from the selected document, including direct requirements and predefined names.
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
