//! Lossless lexer output and normalized literal payloads consumed by the parser and formatter.

use crate::source::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The supported integer-literal radices.
pub enum Radix {
    /// Base two, introduced by `0b`.
    Binary,
    /// Base ten with no prefix.
    Decimal,
    /// Base sixteen, introduced by `0x`.
    Hexadecimal,
}

impl Radix {
    /// Returns the numeric base used to accumulate an integer literal's digits.
    pub const fn value(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// An admitted integer token before expected-type checking.
pub struct IntegerLiteral {
    /// The radix selected by the source prefix.
    pub radix: Radix,
    /// Separator-free digits, excluding prefix and suffix.
    pub digits: String,
    /// An explicit scalar suffix, when present.
    pub suffix: Option<IntegerSuffix>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A decimal float token split for exact, host-independent rounding.
pub struct DecimalFloatLiteral {
    /// All separator-free coefficient digits with the decimal point removed.
    pub digits: String,
    /// The number of coefficient digits originally after the decimal point.
    pub fractional_digits: usize,
    /// Whether the explicit exponent is negative.
    pub exponent_negative: bool,
    /// Separator-free exponent digits; empty when no exponent was written.
    pub exponent_digits: String,
    /// An explicit binary floating-point suffix, when present.
    pub suffix: Option<FloatSuffix>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// An explicit floating-point literal type suffix.
pub enum FloatSuffix {
    /// IEEE 754 binary32.
    Float32,
    /// IEEE 754 binary64.
    Float64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// An explicit integer or target-quantity literal type suffix.
pub enum IntegerSuffix {
    /// Signed 8-bit integer.
    Int8,
    /// Signed 16-bit integer.
    Int16,
    /// Signed 32-bit integer.
    Int32,
    /// Signed 64-bit integer.
    Int64,
    /// Unsigned 8-bit integer.
    UInt8,
    /// Unsigned 16-bit integer.
    UInt16,
    /// Unsigned 32-bit integer.
    UInt32,
    /// Unsigned 64-bit integer.
    UInt64,
    /// Target-width byte quantity.
    ByteSize,
    /// Target-width element count or index.
    USize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A semantic token produced after whitespace and comments are separated into lexemes.
pub enum TokenKind {
    /// An uppercase-leading type identifier.
    TypeIdentifier,
    /// A lowercase-leading value identifier.
    ValueIdentifier,
    /// An admitted integer literal.
    Integer(IntegerLiteral),
    /// An admitted decimal floating-point literal.
    Float(DecimalFloatLiteral),
    /// A decoded byte literal.
    Byte(u8),
    /// A decoded Symbol literal.
    Symbol(Vec<u8>),
    /// The `require` keyword.
    Require,
    /// The `extern` keyword.
    Extern,
    /// The `if` keyword.
    If,
    /// The `when` keyword.
    When,
    /// The `then` keyword.
    Then,
    /// The `else` keyword.
    Else,
    /// The wildcard `_`.
    Underscore,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `<`.
    Less,
    /// `<=`.
    LessEqual,
    /// `>`.
    Greater,
    /// `>=`.
    GreaterEqual,
    /// `,`.
    Comma,
    /// `;`.
    Semicolon,
    /// `::`.
    DoubleColon,
    /// `:=`.
    Bind,
    /// `->`.
    Arrow,
    /// `=>`.
    FatArrow,
    /// `+`.
    Plus,
    /// `-`.
    Minus,
    /// `*`.
    Star,
    /// `/`.
    Slash,
    /// `%`.
    Percent,
    /// `!`.
    Bang,
    /// `!=`.
    BangEqual,
    /// `==`.
    EqualEqual,
    /// `~`.
    Tilde,
    /// `&`.
    Ampersand,
    /// `&&`.
    AmpersandAmpersand,
    /// `|`.
    Pipe,
    /// `||`.
    PipePipe,
    /// `^`.
    Caret,
    /// `.`.
    Dot,
    /// `#`.
    Hash,
    /// `<<`.
    ShiftLeft,
    /// `>>`.
    ShiftRight,
    /// The zero-width end-of-file sentinel.
    Eof,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One parser-visible token and its exact source range.
pub struct Token {
    /// The token category and decoded payload.
    pub kind: TokenKind,
    /// The token's source range.
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A lossless source segment retained for formatting.
pub enum LexemeKind {
    /// A parser-visible token.
    Token,
    /// One contiguous ASCII whitespace segment.
    Whitespace,
    /// A line comment including its delimiter but excluding the line ending.
    LineComment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The category and range of one lossless source segment.
pub struct Lexeme {
    /// How the segment participates in parsing or formatting.
    pub kind: LexemeKind,
    /// The segment's exact source range.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Lexer output shared by the parser and lossless formatter.
pub struct Lexed {
    /// Parser-visible tokens, ending in exactly one [`TokenKind::Eof`].
    pub tokens: Vec<Token>,
    /// Tokens, whitespace, and comments covering the admitted source in order.
    pub lexemes: Vec<Lexeme>,
}
