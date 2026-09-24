use crate::diagnostic::Diagnostic;
use crate::source::{SourceFile, Span};

mod number;
mod symbol;
mod token;

pub use token::{
    DecimalFloatLiteral, FloatSuffix, IntegerLiteral, IntegerSuffix, Lexed, Lexeme, LexemeKind,
    Radix, Token, TokenKind,
};

/// Lexes `source` into the tokens the parser reads. Comments and whitespace are dropped.
pub fn lex(source: &SourceFile) -> Result<Vec<Token>, Diagnostic> {
    Lexer::new(source, false).lex().map(|lexed| lexed.tokens)
}

/// Like `lex`, but also keeps whitespace and comments so the formatter can reproduce them.
pub fn lex_lossless(source: &SourceFile) -> Result<Lexed, Diagnostic> {
    Lexer::new(source, true).lex()
}

struct Lexer<'a> {
    source: &'a SourceFile,
    bytes: &'a [u8],
    offset: usize,
    tokens: Vec<Token>,
    lexemes: Vec<Lexeme>,
    track_lexemes: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a SourceFile, track_lexemes: bool) -> Self {
        Self {
            source,
            bytes: source.text().as_bytes(),
            offset: 0,
            tokens: Vec::new(),
            lexemes: Vec::new(),
            track_lexemes,
        }
    }

    fn lex(mut self) -> Result<Lexed, Diagnostic> {
        while self.offset < self.bytes.len() {
            if self.skip_trivia() {
                continue;
            }

            let start = self.offset;
            let byte = self.bytes[start];
            if byte == b'\'' {
                self.lex_byte(start)?;
            } else if byte == b'"' {
                self.lex_symbol(start)?;
            } else if byte.is_ascii_alphabetic() {
                self.lex_identifier(start)?;
            } else if byte.is_ascii_digit() {
                self.lex_number(start)?;
            } else {
                self.lex_punctuation(start)?;
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.span(self.offset, self.offset),
        });
        Ok(Lexed {
            tokens: self.tokens,
            lexemes: self.lexemes,
        })
    }

    fn skip_trivia(&mut self) -> bool {
        let start = self.offset;
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.offset += 1;
        }
        if self.offset != start {
            self.push_lexeme(LexemeKind::Whitespace, start, self.offset);
        }
        if self.peek() == Some(b'/') && self.peek_next() == Some(b'/') {
            let comment_start = self.offset;
            self.offset += 2;
            while !matches!(self.peek(), None | Some(b'\r' | b'\n')) {
                self.offset += 1;
            }
            self.push_lexeme(LexemeKind::LineComment, comment_start, self.offset);
        }
        self.offset != start
    }

    fn lex_identifier(&mut self, start: usize) -> Result<(), Diagnostic> {
        self.offset += 1;
        while matches!(self.peek(), Some(byte) if byte.is_ascii_alphanumeric()) {
            self.offset += 1;
        }
        if self.peek() == Some(b'_') {
            self.consume_identifier_like();
            return Err(self.error(
                start,
                self.offset,
                "invalid identifier",
                "underscores are not allowed in identifiers",
            ));
        }

        let text = &self.source.text()[start..self.offset];
        let kind = match text {
            "require" => TokenKind::Require,
            "extern" => TokenKind::Extern,
            "if" => TokenKind::If,
            "when" => TokenKind::When,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            _ if self.bytes[start].is_ascii_uppercase() => TokenKind::TypeIdentifier,
            _ => TokenKind::ValueIdentifier,
        };
        self.push(kind, start);
        Ok(())
    }

    fn lex_byte(&mut self, start: usize) -> Result<(), Diagnostic> {
        self.offset += 1;
        let value = match self.peek() {
            Some(b'\\') => {
                self.offset += 1;
                match self.peek() {
                    Some(b'\\') => b'\\',
                    Some(b'\'') => b'\'',
                    Some(b'n') => b'\n',
                    Some(b'r') => b'\r',
                    Some(b't') => b'\t',
                    Some(b'0') => b'\0',
                    Some(b'x') => {
                        self.offset += 1;
                        let Some(high) = self.peek().and_then(hex_value) else {
                            return Err(self.invalid_byte(start, "expected two hexadecimal digits"));
                        };
                        self.offset += 1;
                        let Some(low) = self.peek().and_then(hex_value) else {
                            return Err(self.invalid_byte(start, "expected two hexadecimal digits"));
                        };
                        high * 16 + low
                    }
                    _ => return Err(self.invalid_byte(start, "unknown byte escape")),
                }
            }
            Some(byte @ 0x20..=0x7e) if !matches!(byte, b'\'' | b'\\') => byte,
            _ => return Err(self.invalid_byte(start, "expected one printable ASCII byte")),
        };
        self.offset += 1;
        if self.peek() != Some(b'\'') {
            return Err(self.invalid_byte(start, "byte literal must contain exactly one byte"));
        }
        self.offset += 1;
        self.push(TokenKind::Byte(value), start);
        Ok(())
    }

    fn invalid_byte(&mut self, start: usize, label: &str) -> Diagnostic {
        while let Some(byte) = self.peek() {
            self.offset += 1;
            if byte == b'\'' || matches!(byte, b'\n' | b'\r') {
                break;
            }
        }
        self.error(start, self.offset, "invalid byte literal", label)
    }

    fn lex_symbol(&mut self, start: usize) -> Result<(), Diagnostic> {
        match symbol::decode(self.bytes, start) {
            Ok(decoded) => {
                self.offset = decoded.end;
                self.push(TokenKind::Symbol(decoded.value), start);
                Ok(())
            }
            Err(error) => {
                self.offset = error.end;
                Err(self.error(start, error.end, "invalid Symbol literal", error.label))
            }
        }
    }

    fn lex_punctuation(&mut self, start: usize) -> Result<(), Diagnostic> {
        let (kind, width) = match (self.peek(), self.peek_next()) {
            (Some(b':'), Some(b':')) => (TokenKind::DoubleColon, 2),
            (Some(b':'), Some(b'=')) => (TokenKind::Bind, 2),
            (Some(b'-'), Some(b'>')) => (TokenKind::Arrow, 2),
            (Some(b'='), Some(b'>')) => (TokenKind::FatArrow, 2),
            (Some(b'='), Some(b'=')) => (TokenKind::EqualEqual, 2),
            (Some(b'!'), Some(b'=')) => (TokenKind::BangEqual, 2),
            (Some(b'<'), Some(b'=')) => (TokenKind::LessEqual, 2),
            (Some(b'>'), Some(b'=')) => (TokenKind::GreaterEqual, 2),
            (Some(b'<'), Some(b'<')) => (TokenKind::ShiftLeft, 2),
            (Some(b'>'), Some(b'>')) => (TokenKind::ShiftRight, 2),
            (Some(b'&'), Some(b'&')) => (TokenKind::AmpersandAmpersand, 2),
            (Some(b'|'), Some(b'|')) => (TokenKind::PipePipe, 2),
            (Some(b'_'), _) => {
                self.offset += 1;
                if self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
                    let first = self.offset;
                    self.offset += 1;
                    while matches!(self.peek(), Some(byte) if byte.is_ascii_alphanumeric()) {
                        self.offset += 1;
                    }
                    if self.peek() == Some(b'_') {
                        self.consume_identifier_like();
                        return Err(self.error(
                            start,
                            self.offset,
                            "invalid identifier",
                            "underscores are only allowed at the start of identifiers",
                        ));
                    }
                    let kind = if self.bytes[first].is_ascii_uppercase() {
                        TokenKind::TypeIdentifier
                    } else {
                        TokenKind::ValueIdentifier
                    };
                    self.push(kind, start);
                    return Ok(());
                }
                if self
                    .peek()
                    .is_some_and(|byte| byte.is_ascii_digit() || byte == b'_')
                {
                    self.consume_identifier_like();
                    return Err(self.error(
                        start,
                        self.offset,
                        "invalid identifier",
                        "expected an ASCII letter after `_`",
                    ));
                }
                self.push(TokenKind::Underscore, start);
                return Ok(());
            }
            (Some(b'('), _) => (TokenKind::LeftParen, 1),
            (Some(b')'), _) => (TokenKind::RightParen, 1),
            (Some(b'{'), _) => (TokenKind::LeftBrace, 1),
            (Some(b'}'), _) => (TokenKind::RightBrace, 1),
            (Some(b'['), _) => (TokenKind::LeftBracket, 1),
            (Some(b']'), _) => (TokenKind::RightBracket, 1),
            (Some(b'<'), _) => (TokenKind::Less, 1),
            (Some(b'>'), _) => (TokenKind::Greater, 1),
            (Some(b','), _) => (TokenKind::Comma, 1),
            (Some(b';'), _) => (TokenKind::Semicolon, 1),
            (Some(b'+'), _) => (TokenKind::Plus, 1),
            (Some(b'-'), _) => (TokenKind::Minus, 1),
            (Some(b'*'), _) => (TokenKind::Star, 1),
            (Some(b'/'), _) => (TokenKind::Slash, 1),
            (Some(b'%'), _) => (TokenKind::Percent, 1),
            (Some(b'!'), _) => (TokenKind::Bang, 1),
            (Some(b'~'), _) => (TokenKind::Tilde, 1),
            (Some(b'&'), _) => (TokenKind::Ampersand, 1),
            (Some(b'|'), _) => (TokenKind::Pipe, 1),
            (Some(b'^'), _) => (TokenKind::Caret, 1),
            (Some(b'.'), _) => (TokenKind::Dot, 1),
            (Some(b'#'), _) => (TokenKind::Hash, 1),
            _ => {
                let character = self.source.text()[start..]
                    .chars()
                    .next()
                    .expect("the lexer only reads within the source");
                self.offset += character.len_utf8();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid token",
                    format!("unexpected character `{character}`"),
                ));
            }
        };

        self.offset += width;
        self.push(kind, start);
        Ok(())
    }

    fn consume_identifier_like(&mut self) {
        while matches!(self.peek(), Some(byte) if byte.is_ascii_alphanumeric() || byte == b'_') {
            self.offset += 1;
        }
    }

    fn starts_with(&self, bytes: &[u8]) -> bool {
        self.bytes[self.offset..].starts_with(bytes)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.offset + 1).copied()
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let span = self.span(start, self.offset);
        self.tokens.push(Token { kind, span });
        if self.track_lexemes {
            self.lexemes.push(Lexeme {
                kind: LexemeKind::Token,
                span,
            });
        }
    }

    fn push_lexeme(&mut self, kind: LexemeKind, start: usize, end: usize) {
        if self.track_lexemes {
            self.lexemes.push(Lexeme {
                kind,
                span: self.span(start, end),
            });
        }
    }

    fn error(
        &self,
        start: usize,
        end: usize,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic::error(message).with_primary(self.span(start, end), label)
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.source.id(), start, end)
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
