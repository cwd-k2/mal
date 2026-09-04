use malc::lexer::{IntegerLiteral, IntegerSuffix, Radix, TokenKind, lex};
use malc::source::{FileId, SourceFile, Span};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(7), "test.mal", text.into())
}

fn kinds(text: &str) -> Vec<TokenKind> {
    lex(&source(text))
        .expect("source should lex")
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[test]
fn lexes_the_m0_host_example() {
    assert_eq!(
        kinds(
            "extern printInt32 :: Int32 -> Unit;\n\
             main :: Unit -> Int32 := \\() {\n\
               extern printInt32(42);\n\
               return 0;\n\
             };",
        ),
        vec![
            TokenKind::Extern,
            TokenKind::ValueIdentifier,
            TokenKind::DoubleColon,
            TokenKind::TypeIdentifier,
            TokenKind::Arrow,
            TokenKind::TypeIdentifier,
            TokenKind::Semicolon,
            TokenKind::ValueIdentifier,
            TokenKind::DoubleColon,
            TokenKind::TypeIdentifier,
            TokenKind::Arrow,
            TokenKind::TypeIdentifier,
            TokenKind::Bind,
            TokenKind::Backslash,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::Extern,
            TokenKind::ValueIdentifier,
            TokenKind::LeftParen,
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "42".into(),
                suffix: None,
            }),
            TokenKind::RightParen,
            TokenKind::Semicolon,
            TokenKind::Return,
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "0".into(),
                suffix: None,
            }),
            TokenKind::Semicolon,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn recognizes_keywords_only_at_identifier_boundaries() {
    assert_eq!(
        kinds("if ifValue Bool false true then else case return extern"),
        vec![
            TokenKind::If,
            TokenKind::ValueIdentifier,
            TokenKind::TypeIdentifier,
            TokenKind::ValueIdentifier,
            TokenKind::ValueIdentifier,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::Case,
            TokenKind::Return,
            TokenKind::Extern,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_integer_radices_separators_and_the_m0_suffix() {
    assert_eq!(
        kinds("1_000 0xff_ffInt32 0b1010_0001"),
        vec![
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "1000".into(),
                suffix: None,
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Hexadecimal,
                digits: "ffff".into(),
                suffix: Some(IntegerSuffix::Int32),
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Binary,
                digits: "10100001".into(),
                suffix: None,
            }),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_every_m0_operator_and_delimiter() {
    assert_eq!(
        kinds("_ ( ) { } [ ] < <= > >= , ; :: := -> => \\ + - * / % ! != == ~ & && | || ^ << >>"),
        vec![
            TokenKind::Underscore,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Comma,
            TokenKind::Semicolon,
            TokenKind::DoubleColon,
            TokenKind::Bind,
            TokenKind::Arrow,
            TokenKind::FatArrow,
            TokenKind::Backslash,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Bang,
            TokenKind::BangEqual,
            TokenKind::EqualEqual,
            TokenKind::Tilde,
            TokenKind::Ampersand,
            TokenKind::AmpersandAmpersand,
            TokenKind::Pipe,
            TokenKind::PipePipe,
            TokenKind::Caret,
            TokenKind::ShiftLeft,
            TokenKind::ShiftRight,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn skips_ascii_whitespace_and_line_comments() {
    let tokens = lex(&source("main // until the line ending\r\n  return"))
        .expect("comments and whitespace should lex");
    assert_eq!(tokens[0].kind, TokenKind::ValueIdentifier);
    assert_eq!(tokens[0].span, Span::new(FileId::new(7), 0, 4));
    assert_eq!(tokens[1].kind, TokenKind::Return);
}

#[test]
fn rejects_bad_numeric_separators_with_the_literal_span() {
    for text in ["1_", "1__0", "0x_ff", "0b1_"] {
        let error = lex(&source(text)).expect_err("separator should be rejected");
        assert_eq!(error.message, "invalid numeric separator", "input: {text}");
        assert_eq!(
            error.primary.expect("primary label").span,
            Span::new(FileId::new(7), 0, text.len()),
            "input: {text}",
        );
    }
}

#[test]
fn rejects_invalid_radix_digits_and_unsupported_suffixes() {
    for text in ["0x", "0b2", "12UInt8", "12Int32x"] {
        let error = lex(&source(text)).expect_err("literal should be rejected");
        assert_eq!(error.message, "invalid integer literal", "input: {text}");
    }
}

#[test]
fn rejects_non_ascii_and_underscore_identifiers() {
    for text in ["café", "snake_case", "_name"] {
        assert!(
            lex(&source(text)).is_err(),
            "input should be rejected: {text}"
        );
    }
}

#[test]
fn invalid_characters_produce_renderable_utf8_aligned_spans() {
    let source = source("main := あ");
    let error = lex(&source).expect_err("non-ASCII token should be rejected");
    let rendered = error.render(&source);

    assert!(rendered.contains("test.mal:1:9"));
    assert!(rendered.contains("^ unexpected character `あ`"));
}
