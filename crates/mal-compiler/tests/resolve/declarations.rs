use super::*;

#[test]
fn resolves_predefined_and_source_ordered_top_level_values() {
    let program = resolve_ok("first := false; second := first;");
    let first = top_binding(&program.items[0]);
    let second = top_binding(&program.items[1]);
    let resolved::Expression::Reference(false_ref) = &first.value.kind else {
        panic!("expected false reference");
    };
    assert_eq!(false_ref.id, FALSE_VALUE);

    let resolved::Pattern::Binding(first_binding) = &first.pattern.kind else {
        panic!("expected first binding");
    };
    let resolved::Expression::Reference(first_ref) = &second.value.kind else {
        panic!("expected first reference");
    };
    assert_eq!(first_ref.id, first_binding.id);
    assert_eq!(first_binding.owner, ValueOwner::TopLevel);
}

#[test]
fn preserves_byte_literals_during_name_resolution() {
    let program = resolve_ok(r"value := '\x7f';");
    assert!(matches!(
        top_binding(&program.items[0]).value.kind,
        resolved::Expression::Byte(127)
    ));
}

#[test]
fn preserves_symbol_literals_and_resolves_the_predefined_type() {
    let program = resolve_ok(r#"value :: Symbol := "a\0";"#);
    let binding = top_binding(&program.items[0]);
    assert!(matches!(
        binding.value.kind,
        resolved::Expression::Symbol(ref value) if value == b"a\0"
    ));
    let resolved::TypeExpression::Named(reference) =
        &binding.annotation.as_ref().expect("annotation").kind
    else {
        panic!("expected named type");
    };
    assert_eq!(reference.id, SYMBOL_TYPE);
}

#[test]
fn type_and_external_declarations_are_visible_across_the_unit() {
    let program = resolve_ok(
        "Alias :: Later;\n\
         useLater :: Alias := 0;\n\
         extern run :: Alias -> Unit;\n\
         Later :: Int32;\n\
         invoke := () -> { run(useLater); (); };",
    );
    let TopItem::TypeAlias { value, .. } = &program.items[0].kind else {
        panic!("expected alias");
    };
    let resolved::TypeExpression::Named(later_ref) = &value.kind else {
        panic!("expected type reference");
    };
    let TopItem::TypeAlias { binding, .. } = &program.items[3].kind else {
        panic!("expected Later alias");
    };
    assert_eq!(later_ref.id, binding.id);

    let invoke = top_binding(&program.items[4]);
    let resolved::Expression::Lambda(lambda) = &invoke.value.kind else {
        panic!("expected lambda");
    };
    let resolved::BodyItem::Expression(call) = &lambda.body.items[0] else {
        panic!("expected call");
    };
    let resolved::Expression::Call { callee, .. } = &call.kind else {
        panic!("expected application");
    };
    let resolved::Expression::Reference(reference) = &callee.kind else {
        panic!("expected external function reference");
    };
    let TopItem::ExternalOperation { binding, .. } = &program.items[2].kind else {
        panic!("expected external declaration");
    };
    assert_eq!(reference.id, binding.id);
}

#[test]
fn resolves_every_predefined_fixed_width_integer_type() {
    let program =
        resolve_ok("Types :: [Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32, UInt64];");
    let TopItem::TypeAlias { value, .. } = &program.items[0].kind else {
        panic!("expected alias");
    };
    let resolved::TypeExpression::Sum(members) = &value.kind else {
        panic!("expected sum");
    };
    let ids = members
        .iter()
        .map(|member| match &member.kind {
            resolved::TypeExpression::Named(reference) => reference.id,
            _ => panic!("expected named integer type"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            INT8_TYPE,
            INT16_TYPE,
            INT32_TYPE,
            INT64_TYPE,
            UINT8_TYPE,
            UINT16_TYPE,
            UINT32_TYPE,
            UINT64_TYPE,
        ]
    );
}
