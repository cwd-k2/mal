//! Admitted UTF-8 source, stable file identity, byte spans, and user/editor position conversion.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
/// One admitted UTF-8 source and its precomputed line index.
pub struct SourceFile {
    id: FileId,
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
}

/// The files of one program in load order, with the `.mal` requirements of each file and the `.c` files to link.
///
/// A `FileId` is the position of its file in `files`, and `requirements[i]` lists the requirements of file `i` in source
/// order. `SourceGraph::new` panics when these invariants do not hold.
#[derive(Debug)]
pub struct SourceGraph {
    root: FileId,
    files: Vec<SourceFile>,
    requirements: Vec<Vec<SourceRequirement>>,
    c_sources: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A resolved mal-source requirement edge.
pub struct SourceRequirement {
    /// The required file's graph identity.
    pub target: FileId,
    /// The requirement path span in the requiring file.
    pub span: Span,
}

impl SourceGraph {
    /// Builds a graph from files whose ids equal their positions; `root` must be one of them.
    pub fn new(
        root: FileId,
        files: Vec<SourceFile>,
        requirements: Vec<Vec<SourceRequirement>>,
        c_sources: Vec<PathBuf>,
    ) -> Self {
        assert_eq!(files.len(), requirements.len());
        assert!(root.index() < files.len() as u32);
        for (index, file) in files.iter().enumerate() {
            assert_eq!(file.id().index(), index as u32);
        }
        Self {
            root,
            files,
            requirements,
            c_sources,
        }
    }

    /// Returns the entry source selected by the caller.
    pub const fn root(&self) -> FileId {
        self.root
    }

    /// Borrows the entry source; construction guarantees that it exists.
    pub fn root_source(&self) -> &SourceFile {
        self.source(self.root)
            .expect("a source graph always contains its root")
    }

    /// Returns files in stable identity and admission order.
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Looks up a file by its graph identity.
    pub fn source(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.index() as usize)
    }

    /// Returns direct mal source requirements in source order, or an empty slice for an unknown identity.
    pub fn requirements(&self, id: FileId) -> &[SourceRequirement] {
        self.requirements
            .get(id.index() as usize)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Returns deduplicated C build inputs reached while loading the requirement graph.
    pub fn c_sources(&self) -> &[PathBuf] {
        &self.c_sources
    }
}

/// Source lookup needed to render diagnostics without coupling them to graph storage.
pub trait SourceProvider {
    /// Looks up the source with `id`.
    fn source(&self, id: FileId) -> Option<&SourceFile>;
}

impl SourceProvider for SourceFile {
    fn source(&self, id: FileId) -> Option<&SourceFile> {
        (self.id == id).then_some(self)
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

impl SourceFile {
    /// Admits already decoded UTF-8 text and precomputes line starts for repeated position queries.
    pub fn new(id: FileId, path: impl Into<PathBuf>, text: String) -> Self {
        let mut line_starts = vec![0];
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                    // Treat CRLF as one terminator so no position can address the byte between the pair as a line.
                    index += 2;
                    line_starts.push(index);
                }
                b'\r' | b'\n' => {
                    index += 1;
                    line_starts.push(index);
                }
                _ => index += 1,
            }
        }
        Self {
            id,
            path: path.into(),
            text,
            line_starts,
        }
    }

    /// Reads one file and rejects bytes that are not UTF-8.
    pub fn load(id: FileId, path: impl AsRef<Path>) -> Result<Self, SourceLoadError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| SourceLoadError::Io {
            path: path.to_owned(),
            source,
        })?;
        let text = String::from_utf8(bytes).map_err(|_| SourceLoadError::InvalidUtf8 {
            path: path.to_owned(),
        })?;
        Ok(Self::new(id, path, text))
    }

    /// Returns this source's graph identity.
    pub const fn id(&self) -> FileId {
        self.id
    }

    /// Returns the path used for diagnostics and relative requirements.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the admitted UTF-8 source text unchanged.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Converts a UTF-8 boundary to a one-based line and Unicode-scalar column.
    pub fn location(&self, byte_offset: usize) -> Option<Location> {
        if byte_offset > self.text.len() || !self.text.is_char_boundary(byte_offset) {
            return None;
        }
        let line_index = self
            .line_starts
            .partition_point(|start| *start <= byte_offset)
            - 1;
        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..byte_offset].chars().count() + 1;
        Some(Location {
            line: line_index + 1,
            column,
        })
    }

    /// Converts a UTF-8 boundary to the zero-based UTF-16 position required by LSP.
    pub fn utf16_position(&self, byte_offset: usize) -> Option<Utf16Position> {
        if byte_offset > self.text.len() || !self.text.is_char_boundary(byte_offset) {
            return None;
        }
        let line_index = self
            .line_starts
            .partition_point(|start| *start <= byte_offset)
            - 1;
        let line = self.line(line_index + 1)?;
        if byte_offset > line.start + line.text.len() {
            return None;
        }
        let character = self.text[line.start..byte_offset].encode_utf16().count();
        Some(Utf16Position {
            line: line_index,
            character,
        })
    }

    /// Converts an LSP UTF-16 position to a UTF-8 boundary, rejecting positions inside a surrogate pair.
    pub fn byte_offset_utf16(&self, position: Utf16Position) -> Option<usize> {
        let line = self.line(position.line.checked_add(1)?)?;
        let mut utf16_offset = 0;
        for (byte_offset, character) in line.text.char_indices() {
            if utf16_offset == position.character {
                return Some(line.start + byte_offset);
            }
            utf16_offset += character.len_utf16();
            if utf16_offset > position.character {
                return None;
            }
        }
        (utf16_offset == position.character).then_some(line.start + line.text.len())
    }

    /// Returns one line without its CR/LF terminator.
    pub fn line(&self, one_based_line: usize) -> Option<SourceLine<'_>> {
        let line_index = one_based_line.checked_sub(1)?;
        let start = *self.line_starts.get(line_index)?;
        let end = self
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(self.text.len());
        let text = self.text[start..end]
            .strip_suffix('\n')
            .unwrap_or(&self.text[start..end]);
        let text = text.strip_suffix('\r').unwrap_or(text);
        Some(SourceLine {
            number: one_based_line,
            start,
            text,
        })
    }

    /// Returns whether the span belongs to this file and both endpoints are valid UTF-8 boundaries.
    pub fn contains(&self, span: Span) -> bool {
        span.file == self.id
            && span.end <= self.text.len()
            && self.text.is_char_boundary(span.start)
            && self.text.is_char_boundary(span.end)
    }
}

#[derive(Debug)]
/// Failure to read or decode a source file.
pub enum SourceLoadError {
    /// The operating system rejected the file read.
    Io {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// The file contents were not valid UTF-8.
    InvalidUtf8 {
        /// The path containing invalid bytes.
        path: PathBuf,
    },
}

impl fmt::Display for SourceLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read '{}': {source}", path.display())
            }
            Self::InvalidUtf8 { path } => {
                write!(formatter, "'{}' is not valid UTF-8", path.display())
            }
        }
    }
}

impl std::error::Error for SourceLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidUtf8 { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locations_are_one_based_and_unicode_aware() {
        let source = SourceFile::new(FileId::new(3), "sample.mal", "aé\nxyz".into());

        assert_eq!(source.location(0), Some(Location { line: 1, column: 1 }));
        assert_eq!(source.location(3), Some(Location { line: 1, column: 3 }));
        assert_eq!(source.location(4), Some(Location { line: 2, column: 1 }));
        assert_eq!(source.location(5), Some(Location { line: 2, column: 2 }));
        assert_eq!(source.location(2), None);
    }

    #[test]
    fn lines_exclude_line_endings() {
        let source = SourceFile::new(
            FileId::new(0),
            "sample.mal",
            "first\r\nsecond\nthird\rfourth".into(),
        );

        assert_eq!(source.line(1).expect("first line").text, "first");
        assert_eq!(source.line(2).expect("second line").text, "second");
        assert_eq!(source.line(3).expect("third line").text, "third");
        assert_eq!(source.line(4).expect("fourth line").text, "fourth");
        assert!(source.line(5).is_none());
        assert_eq!(source.location(20), Some(Location { line: 4, column: 1 }));
    }

    #[test]
    fn span_membership_checks_file_bounds_and_utf8_boundaries() {
        let source = SourceFile::new(FileId::new(1), "sample.mal", "é".into());

        assert!(source.contains(Span::new(FileId::new(1), 0, 2)));
        assert!(!source.contains(Span::new(FileId::new(2), 0, 2)));
        assert!(!source.contains(Span::new(FileId::new(1), 0, 1)));
    }

    #[test]
    fn converts_between_byte_offsets_and_zero_based_utf16_positions() {
        let source = SourceFile::new(FileId::new(0), "sample.mal", "a😀\r\né\n".into());

        assert_eq!(
            source.utf16_position(5),
            Some(Utf16Position {
                line: 0,
                character: 3,
            })
        );
        assert_eq!(
            source.utf16_position(7),
            Some(Utf16Position {
                line: 1,
                character: 0,
            })
        );
        assert_eq!(
            source.byte_offset_utf16(Utf16Position {
                line: 0,
                character: 3,
            }),
            Some(5)
        );
        assert_eq!(
            source.byte_offset_utf16(Utf16Position {
                line: 1,
                character: 1,
            }),
            Some(9)
        );
    }

    #[test]
    fn rejects_positions_inside_utf8_or_utf16_characters_and_line_endings() {
        let source = SourceFile::new(FileId::new(0), "sample.mal", "a😀\r\n".into());

        assert_eq!(source.utf16_position(2), None);
        assert_eq!(source.utf16_position(6), None);
        assert_eq!(
            source.byte_offset_utf16(Utf16Position {
                line: 0,
                character: 2,
            }),
            None
        );
        assert_eq!(
            source.byte_offset_utf16(Utf16Position {
                line: 3,
                character: 0,
            }),
            None
        );
    }
}
