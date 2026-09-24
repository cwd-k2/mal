use super::*;

#[test]
fn lexes_byte_literals_and_every_escape() {
    assert_eq!(
        kinds(r"'a' ')' '\\' '\'' '\n' '\r' '\t' '\0' '\x00' '\xff'"),
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
    for text in ["''", "'ab'", "'あ'", r"'\q'", r"'\x0'", r"'\xgg'", "'a"] {
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
fn lexes_symbol_bytes_and_every_escape() {
    assert_eq!(
        kinds(r#""" "hello" "あ" "\\\"\n\r\t\0\x00\xff""#),
        vec![
            TokenKind::Symbol(Vec::new()),
            TokenKind::Symbol(b"hello".to_vec()),
            TokenKind::Symbol("あ".as_bytes().to_vec()),
            TokenKind::Symbol(vec![b'\\', b'"', b'\n', b'\r', b'\t', 0, 0, 255]),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_malformed_symbol_literals_at_the_lexer_boundary() {
    for text in [
        r#""\q""#,
        r#""\x0""#,
        r#""\xgg""#,
        "\"unterminated",
        "\"line\nbreak\"",
    ] {
        let error = lex(&source(text)).expect_err("Symbol literal should be rejected");
        assert_eq!(error.message, "invalid Symbol literal", "input: {text:?}");
        assert_eq!(
            error.primary.expect("primary label").span.start(),
            0,
            "input: {text:?}"
        );
    }
}
