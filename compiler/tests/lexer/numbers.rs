use super::*;

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
fn lexes_target_quantity_suffixes_in_every_integer_radix() {
    assert_eq!(
        kinds("64bytes 0x10usize 0b100bytes"),
        vec![
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "64".into(),
                suffix: Some(IntegerSuffix::ByteSize),
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Hexadecimal,
                digits: "10".into(),
                suffix: Some(IntegerSuffix::USize),
            }),
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Binary,
                digits: "100".into(),
                suffix: Some(IntegerSuffix::ByteSize),
            }),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn separates_an_integer_from_a_receiver_call_dot() {
    assert_eq!(
        kinds("1.f()"),
        vec![
            TokenKind::Integer(IntegerLiteral {
                radix: Radix::Decimal,
                digits: "1".into(),
                suffix: None,
            }),
            TokenKind::Dot,
            TokenKind::ValueIdentifier,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Eof,
        ]
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
    for text in ["1.", "1.\n5", "1e", "1e+", "1.0i32", "1.0f32x"] {
        let error = lex(&source(text)).expect_err("float literal should be rejected");
        assert_eq!(error.message, "invalid float literal", "input: {text}");
    }
    for text in ["1._0", "1.0_", "1e_2", "1e2_"] {
        let error = lex(&source(text)).expect_err("separator should be rejected");
        assert_eq!(error.message, "invalid numeric separator", "input: {text}");
    }
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
fn rejects_invalid_radix_digits_and_malformed_suffixes() {
    for text in [
        "0x", "0b2", "12Byte", "12Int32", "12UInt8", "12i32x", "1Float32",
    ] {
        let error = lex(&source(text)).expect_err("literal should be rejected");
        assert_eq!(error.message, "invalid integer literal", "input: {text}");
    }
}
