use crate::source::Span;

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
    ByteSize,
    USize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    TypeIdentifier,
    ValueIdentifier,
    Integer(IntegerLiteral),
    Float(DecimalFloatLiteral),
    Byte(u8),
    Symbol(Vec<u8>),
    Require,
    Extern,
    If,
    When,
    Then,
    Else,
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
    Dot,
    Hash,
    At,
    Question,
    LeftArrow,
    ShiftLeft,
    ShiftRight,
    Eof,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LexemeKind {
    Token,
    Whitespace,
    LineComment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Lexeme {
    pub kind: LexemeKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub lexemes: Vec<Lexeme>,
}
