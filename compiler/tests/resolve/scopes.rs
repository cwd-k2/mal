use super::*;

#[test]
fn local_bindings_enter_scope_only_after_their_initializer() {
    for text in [
        "first := second; second := 2;",
        "main := \\() { x := x; x; };",
    ] {
        let error = resolve_error(text);
        assert!(error.message.starts_with("unknown value"), "input: {text}");
    }
}

#[test]
fn rejects_mutual_recursion_as_a_forward_reference() {
    let error = resolve_error(
        "first :: Unit -> Unit := \\() { second(); };\n\
         second :: Unit -> Unit := \\() { first(); };",
    );
    assert_eq!(error.message, "unknown value `second`");
}

#[test]
fn local_scope_can_shadow_predefined_and_outer_names() {
    let program = resolve_ok(
        "main := \\(x :: Int32) {\n\
           result := if (true) then {\n\
             x := 1;\n\
             false := x;\n\
             false\n\
           } else { x };\n\
           result;\n\
         };",
    );
    let resolved::Expression::Lambda(main) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(main.body.items[0], resolved::BodyItem::Binding(_)));
}

#[test]
fn branch_bindings_do_not_escape_their_expression_block() {
    let error = resolve_error(
        "main := \\() {\n\
           if (true) then { local := 1; local } else { 0 };\n\
           local;\n\
         };",
    );
    assert_eq!(error.message, "unknown value `local`");
}

#[test]
fn case_pattern_and_block_bindings_share_an_arm_local_scope() {
    resolve_ok(
        "Choice :: [Unit, Int32];\n\
         main := \\(value :: Choice) {\n\
           case (value)\n\
             [0](_) { 0 }\n\
             [1](item) { local := item; local };\n\
         };",
    );

    let duplicate = resolve_error(
        "Choice :: [Unit, Int32];\n\
         main := \\(value :: Choice) {\n\
           case (value)\n\
             [0](_) { 0 }\n\
             [1](item) { item := 1; item };\n\
         };",
    );
    assert!(duplicate.message.starts_with("duplicate value"));

    let escaped = resolve_error(
        "main := \\() {\n\
           case (true) [0](_) { local := 1; local } [1](_) { 0 };\n\
           local;\n\
         };",
    );
    assert_eq!(escaped.message, "unknown value `local`");
}

#[test]
fn rejects_unknown_names_and_reserved_top_level_redefinitions() {
    let cases = [
        ("value :: Missing := 0;", "unknown type `Missing`"),
        (
            "main := \\() { extern missing(); (); };",
            "unknown external operation `missing`",
        ),
        ("Bool :: Int32;", "duplicate type `Bool`"),
        ("false := 0;", "duplicate value `false`"),
        (
            "extern run :: Unit -> Unit; run := 0;",
            "duplicate value `run`",
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(resolve_error(text).message, expected, "input: {text}");
    }
}

#[test]
fn name_errors_retain_the_reference_span() {
    let source = source("value := missing;");
    let parsed = parse(&source).expect("source should parse");
    let error = resolve::resolve(&parsed).expect_err("name should be unresolved");

    assert!(error.render(&source).contains("resolve-test.mal:1:10"));
    assert_eq!(error.primary.expect("primary label").span.start(), 9);
}
