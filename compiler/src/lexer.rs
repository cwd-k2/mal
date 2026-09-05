use crate::diagnostic::Diagnostic;
use crate::source::{SourceFile, Span};

mod string;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Radix {
    Binary,
    Decimal,
    Hexadecimal,
}

impl Radix {
    pub const fn value(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerLiteral {
    pub radix: Radix,
    pub digits: String,
    pub suffix: Option<IntegerSuffix>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecimalFloatLiteral {
    pub digits: String,
    pub fractional_digits: usize,
    pub exponent_negative: bool,
    pub exponent_digits: String,
    pub suffix: Option<FloatSuffix>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatSuffix {
    Float32,
    Float64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerSuffix {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    TypeIdentifier,
    ValueIdentifier,
    Integer(IntegerLiteral),
    Float(DecimalFloatLiteral),
    Byte(u8),
    String(Vec<u8>),
    Extern,
    If,
    Then,
    Else,
    Case,
    Return,
    Underscore,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Comma,
    Semicolon,
    DoubleColon,
    Bind,
    Arrow,
    FatArrow,
    Backslash,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    BangEqual,
    EqualEqual,
    Tilde,
    Ampersand,
    AmpersandAmpersand,
    Pipe,
    PipePipe,
    Caret,
    ShiftLeft,
    ShiftRight,
    Eof,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn lex(source: &SourceFile) -> Result<Vec<Token>, Diagnostic> {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a SourceFile,
    bytes: &'a [u8],
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a SourceFile) -> Self {
        Self {
            source,
            bytes: source.text().as_bytes(),
            offset: 0,
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Result<Vec<Token>, Diagnostic> {
        while self.offset < self.bytes.len() {
            if self.skip_trivia() {
                continue;
            }

            let start = self.offset;
            let byte = self.bytes[start];
            if byte == b'b' && self.peek_next() == Some(b'\'') {
                self.lex_byte(start)?;
            } else if byte == b'"' {
                self.lex_string(start)?;
            } else if byte.is_ascii_alphabetic() {
                self.lex_identifier(start)?;
            } else if byte.is_ascii_digit() {
                self.lex_number(start)?;
            } else {
                self.lex_symbol(start)?;
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.span(self.offset, self.offset),
        });
        Ok(self.tokens)
    }

    fn skip_trivia(&mut self) -> bool {
        let start = self.offset;
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.offset += 1;
        }
        if self.peek() == Some(b'/') && self.peek_next() == Some(b'/') {
            self.offset += 2;
            while !matches!(self.peek(), None | Some(b'\r' | b'\n')) {
                self.offset += 1;
            }
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
            "extern" => TokenKind::Extern,
            "if" => TokenKind::If,
            "then" => TokenKind::Then,
            "else" => TokenKind::Else,
            "case" => TokenKind::Case,
            "return" => TokenKind::Return,
            _ if self.bytes[start].is_ascii_uppercase() => TokenKind::TypeIdentifier,
            _ => TokenKind::ValueIdentifier,
        };
        self.push(kind, start);
        Ok(())
    }

    fn lex_number(&mut self, start: usize) -> Result<(), Diagnostic> {
        let radix = if self.starts_with(b"0x") {
            self.offset += 2;
            Radix::Hexadecimal
        } else if self.starts_with(b"0b") {
            self.offset += 2;
            Radix::Binary
        } else {
            Radix::Decimal
        };
        let digits_start = self.offset;
        self.lex_digit_sequence(start, radix)?;
        let digits_end = self.offset;

        if radix != Radix::Decimal {
            return self.finish_integer(start, radix, digits_start, digits_end);
        }

        let mut fractional_digits = 0;
        let mut is_float = false;
        let mut digits = self.source.text()[digits_start..digits_end].replace('_', "");
        if self.peek() == Some(b'.') {
            is_float = true;
            self.offset += 1;
            let fraction_start = self.offset;
            if self.peek() == Some(b'_') {
                self.consume_number_like();
                return Err(self.invalid_separator(start));
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected digits after the decimal point",
                ));
            }
            let count = self.lex_digit_sequence(start, Radix::Decimal)?;
            fractional_digits = count;
            digits.push_str(&self.source.text()[fraction_start..self.offset].replace('_', ""));
        }

        let mut exponent_negative = false;
        let mut exponent_digits = String::new();
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                exponent_negative = self.peek() == Some(b'-');
                self.offset += 1;
            }
            let exponent_start = self.offset;
            if self.peek() == Some(b'_') {
                self.consume_number_like();
                return Err(self.invalid_separator(start));
            }
            if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected decimal exponent digits",
                ));
            }
            self.lex_digit_sequence(start, Radix::Decimal)?;
            exponent_digits = self.source.text()[exponent_start..self.offset].replace('_', "");
        }

        let float_suffixes = [("f32", FloatSuffix::Float32), ("f64", FloatSuffix::Float64)];
        let float_suffix = float_suffixes
            .into_iter()
            .find(|(text, _)| self.starts_with(text.as_bytes()))
            .map(|(text, suffix)| {
                self.offset += text.len();
                suffix
            });
        is_float |= float_suffix.is_some();

        if is_float {
            if self
                .peek()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                self.consume_number_like();
                return Err(self.error(
                    start,
                    self.offset,
                    "invalid float literal",
                    "expected `f32`, `f64`, or the end of the literal",
                ));
            }
            self.push(
                TokenKind::Float(DecimalFloatLiteral {
                    digits,
                    fractional_digits,
                    exponent_negative,
                    exponent_digits,
                    suffix: float_suffix,
                }),
                start,
            );
            return Ok(());
        }

        self.finish_integer(start, radix, digits_start, digits_end)
    }

    fn lex_digit_sequence(&mut self, start: usize, radix: Radix) -> Result<usize, Diagnostic> {
        let mut previous_was_digit = false;
        let mut digit_count = 0;

        while let Some(byte) = self.peek() {
            if is_digit_for_radix(byte, radix) {
                previous_was_digit = true;
                digit_count += 1;
                self.offset += 1;
            } else if byte == b'_' {
                if !previous_was_digit
                    || !self
                        .peek_next()
                        .is_some_and(|next| is_digit_for_radix(next, radix))
                {
                    self.consume_number_like();
                    return Err(self.invalid_separator(start));
                }
                previous_was_digit = false;
                self.offset += 1;
            } else {
                break;
            }
        }

        if digit_count == 0 {
            self.consume_number_like();
            return Err(self.error(
                start,
                self.offset,
                "invalid integer literal",
                "expected a digit after the radix prefix",
            ));
        }

        Ok(digit_count)
    }

    fn finish_integer(
        &mut self,
        start: usize,
        radix: Radix,
        digits_start: usize,
        digits_end: usize,
    ) -> Result<(), Diagnostic> {
        let suffixes = [
            ("i8", IntegerSuffix::Int8),
            ("i16", IntegerSuffix::Int16),
            ("i32", IntegerSuffix::Int32),
            ("i64", IntegerSuffix::Int64),
            ("u8", IntegerSuffix::UInt8),
            ("u16", IntegerSuffix::UInt16),
            ("u32", IntegerSuffix::UInt32),
            ("u64", IntegerSuffix::UInt64),
        ];
        let suffix = suffixes
            .into_iter()
            .find(|(text, _)| self.starts_with(text.as_bytes()))
            .map(|(text, suffix)| {
                self.offset += text.len();
                suffix
            });

        if self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.consume_number_like();
            return Err(self.error(
                start,
                self.offset,
                "invalid integer literal",
                "expected a fixed-width integer suffix",
            ));
        }

        let digits = self.source.text()[digits_start..digits_end].replace('_', "");
        self.push(
            TokenKind::Integer(IntegerLiteral {
                radix,
                digits,
                suffix,
            }),
            start,
        );
        Ok(())
    }

    fn lex_byte(&mut self, start: usize) -> Result<(), Diagnostic> {
        self.offset += 2;
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

    fn lex_string(&mut self, start: usize) -> Result<(), Diagnostic> {
        match string::decode(self.bytes, start) {
            Ok(decoded) => {
                self.offset = decoded.end;
                self.push(TokenKind::String(decoded.value), start);
                Ok(())
            }
            Err(error) => {
                self.offset = error.end;
                Err(self.error(start, error.end, "invalid string literal", error.label))
            }
        }
    }

    fn lex_symbol(&mut self, start: usize) -> Result<(), Diagnostic> {
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
                if self
                    .peek()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                {
                    self.consume_identifier_like();
                    return Err(self.error(
                        start,
                        self.offset,
                        "invalid identifier",
                        "`_` is a wildcard and cannot start an identifier",
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
            (Some(b'\\'), _) => (TokenKind::Backslash, 1),
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

    fn invalid_separator(&self, start: usize) -> Diagnostic {
        self.error(
            start,
            self.offset,
            "invalid numeric separator",
            "`_` must occur once between two digits",
        )
    }

    fn consume_identifier_like(&mut self) {
        while matches!(self.peek(), Some(byte) if byte.is_ascii_alphanumeric() || byte == b'_') {
            self.offset += 1;
        }
    }

    fn consume_number_like(&mut self) {
        self.consume_identifier_like();
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
        self.tokens.push(Token {
            kind,
            span: self.span(start, self.offset),
        });
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

fn is_digit_for_radix(byte: u8, radix: Radix) -> bool {
    match radix {
        Radix::Binary => matches!(byte, b'0' | b'1'),
        Radix::Decimal => byte.is_ascii_digit(),
        Radix::Hexadecimal => byte.is_ascii_hexdigit(),
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
