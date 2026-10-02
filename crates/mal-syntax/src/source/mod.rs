//! Admitted UTF-8 source, stable file identity, byte spans, and user/editor position conversion.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod file;
mod graph;
#[cfg(test)]
mod tests;

pub use file::{SourceFile, SourceLoadError};
pub use graph::{SourceGraph, SourceRequirement};

/// Stable index of one file within a [`SourceGraph`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileId(u32);

impl FileId {
    /// Constructs an identity; graph construction later verifies that it matches the file position.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the graph index represented by this identity.
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Half-open UTF-8 byte range within one source file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    file: FileId,
    start: usize,
    end: usize,
}

impl Span {
    /// Constructs a half-open span and rejects an end before its start.
    pub fn new(file: FileId, start: usize, end: usize) -> Self {
        assert!(start <= end, "a span must not end before it starts");
        Self { file, start, end }
    }

    /// Returns the file containing the span.
    pub const fn file(self) -> FileId {
        self.file
    }

    /// Returns the inclusive UTF-8 byte start.
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive UTF-8 byte end.
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns whether start and end identify the same byte boundary.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A one-based human-readable source location.
pub struct Location {
    /// The one-based line number.
    pub line: usize,
    /// The one-based Unicode-scalar column.
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A zero-based UTF-16 position as required by LSP.
pub struct Utf16Position {
    /// The zero-based line number.
    pub line: usize,
    /// The zero-based UTF-16 code-unit offset within the line.
    pub character: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// One source line without its line terminator.
pub struct SourceLine<'a> {
    /// The one-based line number.
    pub number: usize,
    /// The line's UTF-8 byte offset in the complete source.
    pub start: usize,
    /// The line text excluding CR and LF bytes.
    pub text: &'a str,
}
/// Source lookup needed to render diagnostics without coupling them to graph storage.
pub trait SourceProvider {
    /// Looks up the source with `id`.
    fn source(&self, id: FileId) -> Option<&SourceFile>;
}

impl SourceProvider for SourceFile {
    fn source(&self, id: FileId) -> Option<&SourceFile> {
        (self.id() == id).then_some(self)
    }
}

impl SourceProvider for SourceGraph {
    fn source(&self, id: FileId) -> Option<&SourceFile> {
        self.source(id)
    }
}

impl SourceProvider for Vec<SourceFile> {
    fn source(&self, id: FileId) -> Option<&SourceFile> {
        self.get(id.index() as usize)
            .filter(|source| source.id() == id)
    }
}
