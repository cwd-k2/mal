//! Public-boundary tests for semantic editor indexes and navigation contracts.

use mal_frontend::editor::{OccurrenceRole, SymbolKind};
use mal_syntax::source::{FileId, SourceFile, SourceGraph, SourceRequirement, Span};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(111), "editor-test.mal", text.into())
}

#[path = "editor/exits.rs"]
mod exits;
#[path = "editor/hover.rs"]
mod hover;
#[path = "editor/navigation.rs"]
mod navigation;
