use malc::lexer::{
    DecimalFloatLiteral, FloatSuffix, IntegerLiteral, IntegerSuffix, Radix, TokenKind, lex,
};
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
fn lexes_integer_radices_separators_and_all_fixed_width_suffixes() {
    assert_eq!(
        kinds("1_000 0xff_ffu32 0b1010_0001u8"),
        vec![
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "1000".into(),
                suffix: None,
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Hexadecimal,
                digits: "ffff".into(),
                suffix: Some(IntegerSuffix::UInt32),
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Binary,
                digits: "10100001".into(),
                suffix: Some(IntegerSuffix::UInt8),
            }),
            TokenKind::Eof,
        ]
    );
    let suffixes = kinds("0i8 0i16 0i32 0i64 0u8 0u16 0u32 0u64");
    assert_eq!(
        suffixes,
        [
            IntegerSuffix::Int8,
            IntegerSuffix::Int16,
            IntegerSuffix::Int32,
            IntegerSuffix::Int64,
            IntegerSuffix::UInt8,
            IntegerSuffix::UInt16,
            IntegerSuffix::UInt32,
            IntegerSuffix::UInt64,
        ]
        .into_iter()
        .map(|suffix| TokenKind::Integer(IntegerLiteral {
            radix: Radix::Decimal,
            digits: "0".into(),
            suffix: Some(suffix),
        }))
        .chain([TokenKind::Eof])
        .collect::<Vec<_>>()
    );
}

#[test]
fn lexes_decimal_float_forms_and_separators() {
    assert_eq!(
        kinds("1.5 1_000.25f32 2e3 4E-2f64 6f32"),
        vec![
            TokenKind::Float(DecimalFloatLiteral {
                digits: "15".into(),
                fractional_digits: 1,
                exponent_negative: false,
                exponent_digits: String::new(),
                suffix: None,
            }),
            TokenKind::Float(DecimalFloatLiteral {
                digits: "100025".into(),
                fractional_digits: 2,
                exponent_negative: false,
                exponent_digits: String::new(),
                suffix: Some(FloatSuffix::Float32),
            }),
            TokenKind::Float(DecimalFloatLiteral {
                digits: "2".into(),
                fractional_digits: 0,
                exponent_negative: false,
                exponent_digits: "3".into(),
                suffix: None,
            }),
            TokenKind::Float(DecimalFloatLiteral {
                digits: "4".into(),
                fractional_digits: 0,
                exponent_negative: true,
                exponent_digits: "2".into(),
                suffix: Some(FloatSuffix::Float64),
            }),
            TokenKind::Float(DecimalFloatLiteral {
                digits: "6".into(),
                fractional_digits: 0,
                exponent_negative: false,
                exponent_digits: String::new(),
                suffix: Some(FloatSuffix::Float32),
            }),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_malformed_decimal_float_literals() {
    for text in ["1.", "1e", "1e+", "1.0i32", "1.0f32x"] {
        let error = lex(&source(text)).expect_err("float literal should be rejected");
        assert_eq!(error.message, "invalid float literal", "input: {text}");
    }
    for text in ["1._0", "1.0_", "1e_2", "1e2_"] {
        let error = lex(&source(text)).expect_err("separator should be rejected");
        assert_eq!(error.message, "invalid numeric separator", "input: {text}");
    }
}

#[test]
fn lexes_byte_literals_and_every_escape() {
    assert_eq!(
        kinds(r"b'a' b')' b'\\' b'\'' b'\n' b'\r' b'\t' b'\0' b'\x00' b'\xff'"),
        vec![
            TokenKind::Byte(b'a'),
            TokenKind::Byte(b')'),
            TokenKind::Byte(b'\\'),
            TokenKind::Byte(b'\''),
            TokenKind::Byte(b'\n'),
            TokenKind::Byte(b'\r'),
            TokenKind::Byte(b'\t'),
            TokenKind::Byte(b'\0'),
            TokenKind::Byte(0),
            TokenKind::Byte(255),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_malformed_byte_literals_at_the_lexer_boundary() {
    for text in [
        "b''", "b'ab'", "b'あ'", r"b'\q'", r"b'\x0'", r"b'\xgg'", "b'a",
    ] {
        let error = lex(&source(text)).expect_err("byte literal should be rejected");
        assert_eq!(error.message, "invalid byte literal", "input: {text}");
        assert_eq!(
            error.primary.expect("primary label").span,
            Span::new(FileId::new(7), 0, text.len()),
            "input: {text}"
        );
    }
}

#[test]
fn lexes_string_bytes_and_every_escape() {
    assert_eq!(
        kinds(r#""" "hello" "あ" "\\\"\n\r\t\0\x00\xff""#),
        vec![
            TokenKind::String(Vec::new()),
            TokenKind::String(b"hello".to_vec()),
            TokenKind::String("あ".as_bytes().to_vec()),
            TokenKind::String(vec![b'\\', b'"', b'\n', b'\r', b'\t', 0, 0, 255]),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_malformed_string_literals_at_the_lexer_boundary() {
    for text in [
        r#""\q""#,
        r#""\x0""#,
        r#""\xgg""#,
        "\"unterminated",
        "\"line\nbreak\"",
    ] {
        let error = lex(&source(text)).expect_err("string literal should be rejected");
        assert_eq!(error.message, "invalid string literal", "input: {text:?}");
        assert_eq!(
            error.primary.expect("primary label").span.start(),
            0,
            "input: {text:?}"
        );
    }
}

#[test]
fn lexes_every_operator_and_delimiter() {
    assert_eq!(
        kinds("_ ( ) { } [ ] < <= > >= , ; :: := -> \\ + - * / % ! != == ~ & && | || ^ << >>"),
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
fn rejects_the_removed_case_arrow() {
    let error = lex(&source("=>")).expect_err("the removed case arrow should be rejected");
    assert_eq!(error.message, "invalid token");
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
fn lossless_lexing_covers_tokens_whitespace_and_comments_in_source_order() {
    use malc::lexer::LexemeKind;

    let source = source("  value// note\r\n :: Int32 := 0xffu32;\n");
    let lexed = malc::lexer::lex_lossless(&source).unwrap();
    let mut restored = String::new();
    let mut end = 0;
    for lexeme in &lexed.lexemes {
        assert_eq!(lexeme.span.start(), end);
        restored.push_str(&source.text()[lexeme.span.start()..lexeme.span.end()]);
        end = lexeme.span.end();
    }

    assert_eq!(end, source.text().len());
    assert_eq!(restored, source.text());
    assert_eq!(
        lexed
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.kind == LexemeKind::Token)
            .count(),
        lexed.tokens.len() - 1
    );
    assert!(
        lexed
            .lexemes
            .iter()
            .any(|lexeme| lexeme.kind == LexemeKind::Whitespace)
    );
    assert!(
        lexed
            .lexemes
            .iter()
            .any(|lexeme| lexeme.kind == LexemeKind::LineComment)
    );
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
    for text in [
        "0x", "0b2", "12Byte", "12Int32", "12UInt8", "12i32x", "1Float32",
    ] {
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
