//! One admitted UTF-8 source file: its text, line index, and position conversions, and the failure to load one.

use super::*;

#[derive(Debug)]
/// One admitted UTF-8 source and its precomputed line index.
pub struct SourceFile {
    id: FileId,
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
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
