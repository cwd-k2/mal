use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileId(u32);

impl FileId {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    file: FileId,
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(file: FileId, start: usize, end: usize) -> Self {
        assert!(start <= end, "a span must not end before it starts");
        Self { file, start, end }
    }

    pub const fn file(self) -> FileId {
        self.file
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Utf16Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLine<'a> {
    pub number: usize,
    pub start: usize,
    pub text: &'a str,
}

#[derive(Debug)]
pub struct SourceFile {
    id: FileId,
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(id: FileId, path: impl Into<PathBuf>, text: String) -> Self {
        let mut line_starts = vec![0];
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
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

    pub const fn id(&self) -> FileId {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

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

    pub fn contains(&self, span: Span) -> bool {
        span.file == self.id
            && span.end <= self.text.len()
            && self.text.is_char_boundary(span.start)
            && self.text.is_char_boundary(span.end)
    }
}

#[derive(Debug)]
pub enum SourceLoadError {
    Io { path: PathBuf, source: io::Error },
    InvalidUtf8 { path: PathBuf },
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
