use malc::ast;
use malc::parser::parse;
use malc::resolve;
use malc::resolve::ast::{
    self as resolved, FALSE_VALUE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, TopItem,
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
fn type_and_external_declarations_are_visible_across_the_unit() {
    let program = resolve_ok(
        "Alias :: Later;\n\
         useLater :: Alias := 0;\n\
         extern run :: Alias -> Unit;\n\
         Later :: Int32;\n\
         invoke := \\() { extern run(useLater); return (); };",
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
           return \\<x>(y :: Int32) { return x + y; };\n\
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
           return \\<x>() {\n\
             return \\<x>() { return x; };\n\
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
        "main := \\() { x := x; return x; };",
    ] {
        let error = resolve_error(text);
        assert!(error.message.starts_with("unknown value"), "input: {text}");
    }
}

#[test]
fn rejects_an_unlisted_outer_local_reference() {
    let error = resolve_error(
        "outer := \\(x :: Int32) {\n\
           return \\() { return x; };\n\
         };",
    );
    assert_eq!(error.message, "value `x` is not captured");
}

#[test]
fn rejects_capture_that_skips_an_enclosing_lambda() {
    let error = resolve_error(
        "outer := \\(x :: Int32) {\n\
           return \\() {\n\
             return \\<x>() { return x; };\n\
           };\n\
         };",
    );
    assert_eq!(error.message, "capture `x` crosses a lambda boundary");
}

#[test]
fn rejects_duplicate_invalid_and_non_local_captures() {
    let cases = [
        (
            "outer := \\(x :: Int32) { return \\<x, x>() { return x; }; };",
            "duplicate capture `x`",
        ),
        (
            "outer := \\(x :: Int32) { return \\<missing>() { return x; }; };",
            "unknown captured value `missing`",
        ),
        (
            "top := 1; closure := \\<top>() { return top; };",
            "cannot capture non-local value `top`",
        ),
        (
            "closure := \\<false>() { return false; };",
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
        "outer := \\(x :: Int32) { return \\<x>(x :: Int32) { return x; }; };",
        "main := \\(x :: Int32, x :: Int32) { return x; };",
        "main := \\() { x := 1; x := 2; return x; };",
        "main := \\() { (x, x) := (1, 2); return x; };",
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
           return result;\n\
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
           return local;\n\
         };",
    );
    assert_eq!(error.message, "unknown value `local`");
}

#[test]
fn rejects_unknown_names_and_reserved_top_level_redefinitions() {
    let cases = [
        ("value :: Missing := 0;", "unknown type `Missing`"),
        (
            "main := \\() { extern missing(); return (); };",
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
