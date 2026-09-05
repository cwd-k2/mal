use malc::ast;
use malc::parser::parse;
use malc::resolve;
use malc::resolve::ast::{
    self as resolved, BYTE_AT_VALUE, BYTE_LENGTH_VALUE, FALSE_VALUE, INT8_TYPE, INT16_TYPE,
    INT32_TYPE, INT64_TYPE, LOAD_INT64_VALUE, LOAD_PTR_VALUE, LOAD_UINT8_VALUE, OFFSET_VALUE,
    PTR_TYPE, STORE_INT64_VALUE, STORE_PTR_VALUE, STORE_UINT8_VALUE, STRING_TYPE, TopItem,
    UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, ValueOwner,
};
use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(19), "resolve-test.mal", text.into())
}

fn resolve_ok(text: &str) -> resolved::Program {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)))
}

fn resolve_error(text: &str) -> malc::diagnostic::Diagnostic {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    resolve::resolve(&parsed).expect_err("name resolution should fail")
}

fn top_binding(item: &ast::Node<TopItem>) -> &resolved::Binding {
    let TopItem::Binding(binding) = &item.kind else {
        panic!("expected a top-level binding");
    };
    binding
}

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
    let program = resolve_ok(r"value := b'\x7f';");
    assert!(matches!(
        top_binding(&program.items[0]).value.kind,
        resolved::Expression::Byte(127)
    ));
}

#[test]
fn preserves_string_literals_and_resolves_the_predefined_type() {
    let program = resolve_ok(r#"value :: String := "a\0";"#);
    let binding = top_binding(&program.items[0]);
    assert!(matches!(
        binding.value.kind,
        resolved::Expression::String(ref value) if value == b"a\0"
    ));
    let resolved::TypeExpression::Named(reference) =
        &binding.annotation.as_ref().expect("annotation").kind
    else {
        panic!("expected named type");
    };
    assert_eq!(reference.id, STRING_TYPE);
}

#[test]
fn resolves_string_primitives_as_predefined_values() {
    let program = resolve_ok(r#"length := byteLength("abc"); item := byteAt("abc", 1u64);"#);
    for (index, expected) in [(0, BYTE_LENGTH_VALUE), (1, BYTE_AT_VALUE)] {
        let resolved::Expression::Call { callee, .. } =
            &top_binding(&program.items[index]).value.kind
        else {
            panic!("expected primitive call");
        };
        let resolved::Expression::Reference(reference) = &callee.kind else {
            panic!("expected primitive reference");
        };
        assert_eq!(reference.id, expected);
    }
}

#[test]
fn resolves_memory_primitives_and_the_ptr_type() {
    let program = resolve_ok(
        "extern memory :: Unit -> Ptr;\n\
         useMemory :: Ptr -> Unit := \\(pointer :: Ptr) {\n\
           next := offset(pointer, 8u64);\n\
           value := loadInt64(next);\n\
           storeInt64(next, value);\n\
           byte := loadUInt8(next);\n\
           storeUInt8(next, byte);\n\
           target := loadPtr(next);\n\
           storePtr(next, target);\n\
           ();\n\
         };",
    );
    let TopItem::ExternalOperation { ty, .. } = &program.items[0].kind else {
        panic!("expected external operation");
    };
    let resolved::TypeExpression::Function { result, .. } = &ty.kind else {
        panic!("expected function type");
    };
    let resolved::TypeExpression::Named(reference) = &result.kind else {
        panic!("expected Ptr result");
    };
    assert_eq!(reference.id, PTR_TYPE);

    let binding = top_binding(&program.items[1]);
    let resolved::Expression::Lambda(lambda) = &binding.value.kind else {
        panic!("expected lambda");
    };
    let expected = [
        OFFSET_VALUE,
        LOAD_INT64_VALUE,
        STORE_INT64_VALUE,
        LOAD_UINT8_VALUE,
        STORE_UINT8_VALUE,
        LOAD_PTR_VALUE,
        STORE_PTR_VALUE,
    ];
    for (item, expected) in lambda.body.items.iter().zip(expected) {
        let expression = match item {
            resolved::BodyItem::Binding(binding) => &binding.kind.value,
            resolved::BodyItem::Expression(expression) => expression,
        };
        let resolved::Expression::Call { callee, .. } = &expression.kind else {
            panic!("expected primitive call");
        };
        let resolved::Expression::Reference(reference) = &callee.kind else {
            panic!("expected primitive reference");
        };
        assert_eq!(reference.id, expected);
    }
}

#[test]
fn type_and_external_declarations_are_visible_across_the_unit() {
    let program = resolve_ok(
        "Alias :: Later;\n\
         useLater :: Alias := 0;\n\
         extern run :: Alias -> Unit;\n\
         Later :: Int32;\n\
         invoke := \\() { extern run(useLater); (); };",
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
    assert!(matches!(
        lambda.body.items[0],
        resolved::BodyItem::Expression(ast::Node {
            kind: resolved::Expression::ExternalCall { .. },
            ..
        })
    ));
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

#[test]
fn capture_sources_and_environment_bindings_have_distinct_identities() {
    let program = resolve_ok(
        "make := \\(x :: Int32) {\n\
           \\<x>(y :: Int32) { x + y; };\n\
         };",
    );
    let resolved::Expression::Lambda(outer) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected outer lambda");
    };
    let resolved::Expression::Lambda(inner) = &outer.body.result.kind else {
        panic!("expected inner lambda");
    };
    assert_eq!(inner.captures.len(), 1);
    assert_eq!(inner.captures[0].source.id, outer.parameters[0].binding.id);
    assert_ne!(inner.captures[0].source.id, inner.captures[0].binding.id);
    assert_eq!(
        inner.captures[0].binding.owner,
        ValueOwner::Lambda(inner.id)
    );

    let resolved::Expression::Binary { left, .. } = &inner.body.result.kind else {
        panic!("expected binary result");
    };
    let resolved::Expression::Reference(captured_ref) = &left.kind else {
        panic!("expected captured reference");
    };
    assert_eq!(captured_ref.id, inner.captures[0].binding.id);
}

#[test]
fn nested_capture_is_forwarded_at_every_lambda_boundary() {
    let program = resolve_ok(
        "outer := \\(x :: Int32) {\n\
           \\<x>() {\n\
             \\<x>() { x; };\n\
           };\n\
         };",
    );
    let resolved::Expression::Lambda(outer) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected outer lambda");
    };
    let resolved::Expression::Lambda(middle) = &outer.body.result.kind else {
        panic!("expected middle lambda");
    };
    let resolved::Expression::Lambda(inner) = &middle.body.result.kind else {
        panic!("expected inner lambda");
    };
    assert_eq!(middle.captures[0].source.id, outer.parameters[0].binding.id);
    assert_eq!(inner.captures[0].source.id, middle.captures[0].binding.id);
}

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
fn resolves_annotated_direct_lambda_self_references() {
    let program = resolve_ok(
        "top :: Int64 -> Int64 := \\(n :: Int64) { top(n); };\n\
         main := \\() {\n\
           local :: Int64 -> Int64 := \\(n :: Int64) { local(n); };\n\
           0i32;\n\
         };",
    );

    let top = top_binding(&program.items[0]);
    let resolved::Pattern::Binding(top_value_binding) = &top.pattern.kind else {
        panic!("expected top-level name pattern");
    };
    let resolved::Expression::Lambda(top_lambda) = &top.value.kind else {
        panic!("expected top-level lambda");
    };
    assert_eq!(top_lambda.self_binding, Some(top_value_binding.id));

    let main = top_binding(&program.items[1]);
    let resolved::Expression::Lambda(main_lambda) = &main.value.kind else {
        panic!("expected main lambda");
    };
    let resolved::BodyItem::Binding(local) = &main_lambda.body.items[0] else {
        panic!("expected local binding");
    };
    let resolved::Pattern::Binding(local_binding) = &local.kind.pattern.kind else {
        panic!("expected local name pattern");
    };
    let resolved::Expression::Lambda(local_lambda) = &local.kind.value.kind else {
        panic!("expected local lambda");
    };
    assert_eq!(local_lambda.self_binding, Some(local_binding.id));
}

#[test]
fn rejects_self_reference_outside_the_annotated_direct_lambda_exception() {
    let cases = [
        ("value := \\() { value(); };", "value"),
        ("value :: Unit -> Unit := (\\() { value(); });", "value"),
        ("value :: Unit := value;", "value"),
        (
            "(first, second) :: (Unit -> Unit, Unit) := (\\() { first(); }, ());",
            "first",
        ),
        (
            "main := \\() { local := \\() { local(); }; 0i32; };",
            "local",
        ),
    ];
    for (text, name) in cases {
        assert_eq!(
            resolve_error(text).message,
            format!("unknown value `{name}`"),
            "input: {text}"
        );
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
fn rejects_an_unlisted_outer_local_reference() {
    let error = resolve_error(
        "outer := \\(x :: Int32) {\n\
           \\() { x; };\n\
         };",
    );
    assert_eq!(error.message, "value `x` is not captured");
}

#[test]
fn rejects_capture_that_skips_an_enclosing_lambda() {
    let error = resolve_error(
        "outer := \\(x :: Int32) {\n\
           \\() {\n\
             \\<x>() { x; };\n\
           };\n\
         };",
    );
    assert_eq!(error.message, "capture `x` crosses a lambda boundary");
}

#[test]
fn rejects_duplicate_invalid_and_non_local_captures() {
    let cases = [
        (
            "outer := \\(x :: Int32) { \\<x, x>() { x; }; };",
            "duplicate capture `x`",
        ),
        (
            "outer := \\(x :: Int32) { \\<missing>() { x; }; };",
            "unknown captured value `missing`",
        ),
        (
            "top := 1; closure := \\<top>() { top; };",
            "cannot capture non-local value `top`",
        ),
        (
            "closure := \\<false>() { false; };",
            "cannot capture non-local value `false`",
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(resolve_error(text).message, expected, "input: {text}");
    }
}

#[test]
fn rejects_capture_parameter_and_same_scope_binding_collisions() {
    let cases = [
        "outer := \\(x :: Int32) { \\<x>(x :: Int32) { x; }; };",
        "main := \\(x :: Int32, x :: Int32) { x; };",
        "main := \\() { x := 1; x := 2; x; };",
        "main := \\() { (x, x) := (1, 2); x; };",
    ];
    for text in cases {
        assert!(
            resolve_error(text).message.starts_with("duplicate value"),
            "input: {text}"
        );
    }
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
        ("byteLength := 0;", "duplicate value `byteLength`"),
        (
            "extern byteAt :: Unit -> Unit;",
            "duplicate top-level value `byteAt`",
        ),
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
