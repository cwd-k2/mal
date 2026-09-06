use super::*;

#[test]
fn checks_symbol_literals_as_immutable_bytes() {
    let program = check_ok(r#"empty :: Symbol := ""; bytes := "あ\0\xff";"#);
    assert_eq!(top_binding(&program, 0).value.ty, Type::Symbol);
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::Symbol(ref value) if value == &[0xe3, 0x81, 0x82, 0, 255]
    ));
}

#[test]
fn checks_symbol_operators_and_byte_wise_equality() {
    let program = check_ok(
        r#"length :: Symbol -> UInt64 := \(value :: Symbol) { #value; };
item :: (Symbol, UInt64) -> UInt8 := \(value :: Symbol, index :: UInt64) {
  value # index;
};
same :: Unit -> Bool := \() { "a\0" == "a\x00"; };
different :: Unit -> Bool := \() { "a" != "b"; };
literal :: Unit -> UInt64 := \() { #"hoge" + UInt64("hoge" # 1); };
concatenate :: (Symbol, Symbol) -> Symbol := \(left :: Symbol, right :: Symbol) {
  left + right;
};"#,
    );
    let ExpressionKind::Lambda(length) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        length.body.result.kind,
        ExpressionKind::SymbolLength { .. }
    ));
    let ExpressionKind::Lambda(item) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        item.body.result.kind,
        ExpressionKind::SymbolAt { .. }
    ));
    for index in 2..=3 {
        let Type::Function { result, .. } = &top_binding(&program, index).value.ty else {
            panic!("expected function type");
        };
        assert_eq!(result.as_ref(), &Type::Sum(vec![Type::Unit, Type::Unit]));
    }
    let Type::Function { result, .. } = &top_binding(&program, 4).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::UInt64);
    let Type::Function { result, .. } = &top_binding(&program, 5).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::Symbol);
}

#[test]
fn rejects_unsupported_or_mistyped_symbol_operations() {
    for text in [
        r#"bad := \() { "a" + 1; };"#,
        r#"bad := "a" < "b";"#,
        r#"bad := #1;"#,
        r#"bad := "a" # 0u8;"#,
        r#"bad := 1 # 0u64;"#,
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}
