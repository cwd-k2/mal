use super::*;

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
