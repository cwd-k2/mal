use super::*;

#[test]
fn lexes_the_basic_host_example() {
    assert_eq!(
        kinds(
            "extern printInt32 :: Int32 -> Unit;\n\
             main :: Unit -> Int32 := \\() {\n\
               printInt32(42);\n\
               0;\n\
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
            TokenKind::ValueIdentifier,
            TokenKind::LeftParen,
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "42".into(),
                suffix: None,
            }),
            TokenKind::RightParen,
            TokenKind::Semicolon,
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
        kinds("require required if ifValue Bool false true then else case return extern"),
        vec![
            TokenKind::Require,
            TokenKind::ValueIdentifier,
            TokenKind::If,
            TokenKind::ValueIdentifier,
            TokenKind::TypeIdentifier,
            TokenKind::ValueIdentifier,
            TokenKind::ValueIdentifier,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::Case,
            TokenKind::ValueIdentifier,
            TokenKind::Extern,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_every_operator_and_delimiter() {
    assert_eq!(
        kinds("_ ( ) { } [ ] < <= > >= , ; :: := -> \\ + - * / % ! != == ~ & && | || ^ . # << >>"),
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
            TokenKind::Dot,
            TokenKind::Hash,
            TokenKind::ShiftLeft,
            TokenKind::ShiftRight,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_the_unsupported_fat_arrow_token() {
    let error = lex(&source("=>")).expect_err("the unsupported token should be rejected");
    assert_eq!(error.message, "invalid token");
}

#[test]
fn skips_ascii_whitespace_and_line_comments() {
    let tokens = lex(&source("main // until the line ending\r\n  return"))
        .expect("comments and whitespace should lex");
    assert_eq!(tokens[0].kind, TokenKind::ValueIdentifier);
    assert_eq!(tokens[0].span, Span::new(FileId::new(7), 0, 4));
    assert_eq!(tokens[1].kind, TokenKind::ValueIdentifier);
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
fn accepts_leading_underscore_identifiers() {
    assert_eq!(
        kinds("_private _Private"),
        vec![
            TokenKind::ValueIdentifier,
            TokenKind::TypeIdentifier,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_non_ascii_and_internal_underscore_identifiers() {
    for text in ["café", "snake_case", "_snake_case", "_1"] {
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
