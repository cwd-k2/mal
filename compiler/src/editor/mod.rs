use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::{SourceFile, Span};

mod index;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SymbolId {
    Type(resolved::TypeId),
    Value(resolved::ValueId),
    ExternalOperation(resolved::ExternalOperationId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolKind {
    Type,
    Value,
    Function,
    Parameter,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub span: Option<Span>,
}

#[derive(Clone, Debug)]
pub struct SemanticDocument {
    occurrences: Vec<Occurrence>,
    typed_regions: Vec<(Span, String)>,
    document_symbols: Vec<Symbol>,
    completions: Vec<Symbol>,
}

pub fn analyze(source: &SourceFile) -> Result<SemanticDocument, Diagnostic> {
    let analysis = crate::pipeline::analyze(source)?;
    Ok(index::build(&analysis.resolved, &analysis.checked))
}

impl SemanticDocument {
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }

    pub fn occurrence_at(&self, byte_offset: usize) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| contains(occurrence.span, byte_offset))
            .min_by_key(|occurrence| occurrence.span.end() - occurrence.span.start())
    }

    pub fn hover_at(&self, byte_offset: usize) -> Option<&str> {
        if let Some(detail) = self
            .occurrence_at(byte_offset)
            .and_then(|occurrence| occurrence.detail.as_deref())
        {
            return Some(detail);
        }
        self.typed_regions
            .iter()
            .filter(|(span, _)| contains(*span, byte_offset))
            .min_by_key(|(span, _)| span.end() - span.start())
            .map(|(_, ty)| ty.as_str())
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
}

fn contains(span: Span, byte_offset: usize) -> bool {
    span.start() <= byte_offset && byte_offset < span.end()
}
