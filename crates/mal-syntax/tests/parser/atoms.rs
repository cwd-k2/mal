use super::*;

#[test]
fn parses_a_byte_literal_as_an_atomic_expression() {
    assert_eq!(binding_value("value := ')';"), Expression::Byte(b')'));
    assert_eq!(binding_value(r"value := '\xff';"), Expression::Byte(255));
}

#[test]
fn rejects_an_identifier_adjacent_to_a_byte_literal() {
    for text in ["value := b'a';", "value := c'a';"] {
        let error = parse(&source(text)).expect_err("adjacent expressions should be rejected");
        assert_eq!(error.message, "expected `;`", "input: {text}");
    }
}

#[test]
fn parses_a_decimal_float_as_an_atomic_expression() {
    let program = parse_ok("value := 1.25e-2f32;");
    let TopItem::Binding(binding) = &program.items[0].kind else {
        panic!("expected binding");
    };
    assert!(matches!(binding.value.kind, Expression::Float(_)));
}

#[test]
fn parses_a_symbol_literal_as_bytes() {
    assert_eq!(
        binding_value(r#"value := "a\0\xff";"#),
        Expression::Symbol(vec![b'a', 0, 255])
    );
}

#[test]
fn rejects_non_call_member_access() {
    for text in ["value := value.load;", "value := UInt8.Load;"] {
        let error = parse(&source(text)).expect_err("member syntax should be restricted");
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn parses_postfix_numeric_conversions() {
    assert!(matches!(
        binding_value("value := 1i8.u8;"),
        Expression::Conversion { .. }
    ));
}
