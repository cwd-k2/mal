use super::*;

#[test]
fn capture_sources_and_environment_bindings_have_distinct_identities() {
    let program = resolve_ok(
        "make := \\(x :: Int32) {\n\
           \\(y :: Int32) { x + y; };\n\
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
fn nested_capture_is_inferred_and_forwarded_at_every_lambda_boundary() {
    let program = resolve_ok(
        "outer := \\(x :: Int32) {\n\
           \\() {\n\
             \\() { x; };\n\
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
fn infers_an_outer_local_reference() {
    let program = resolve_ok(
        "outer := \\(x :: Int32) {\n\
           \\() { x; };\n\
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
}

#[test]
fn infers_each_capture_once_and_respects_shadowing() {
    let program = resolve_ok(
        "outer := \\(x :: Int32) {\n\
           captured := \\() { if (true) then { x } else { x }; };\n\
           shadowed := \\(x :: Int32) { x; };\n\
           (captured, shadowed);\n\
         };",
    );
    let resolved::Expression::Lambda(outer) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected outer lambda");
    };
    let resolved::BodyItem::Binding(captured) = &outer.body.items[0] else {
        panic!("expected captured closure binding");
    };
    let resolved::Expression::Lambda(captured) = &captured.kind.value.kind else {
        panic!("expected captured lambda");
    };
    assert_eq!(captured.captures.len(), 1);

    let resolved::BodyItem::Binding(shadowed) = &outer.body.items[1] else {
        panic!("expected shadowed closure binding");
    };
    let resolved::Expression::Lambda(shadowed) = &shadowed.kind.value.kind else {
        panic!("expected shadowed lambda");
    };
    assert!(shadowed.captures.is_empty());
}

#[test]
fn inferred_capture_does_not_block_later_local_shadowing() {
    let program = resolve_ok(
        "outer := \\(x :: Int32) {\n\
           middle := \\() {\n\
             before := \\() { x; };\n\
             x := 2;\n\
             after := \\() { x; };\n\
             (before, after);\n\
           };\n\
           middle;\n\
         };",
    );
    let resolved::Expression::Lambda(outer) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected outer lambda");
    };
    let resolved::BodyItem::Binding(middle) = &outer.body.items[0] else {
        panic!("expected middle binding");
    };
    let resolved::Expression::Lambda(middle) = &middle.kind.value.kind else {
        panic!("expected middle lambda");
    };
    assert_eq!(middle.captures.len(), 1);

    let resolved::BodyItem::Binding(local) = &middle.body.items[1] else {
        panic!("expected shadowing local");
    };
    let resolved::BodyItem::Binding(after) = &middle.body.items[2] else {
        panic!("expected second closure");
    };
    let resolved::Expression::Lambda(after) = &after.kind.value.kind else {
        panic!("expected second lambda");
    };
    let resolved::Pattern::Binding(local) = &local.kind.pattern.kind else {
        panic!("expected local binding pattern");
    };
    assert_eq!(after.captures.len(), 1);
    assert_eq!(after.captures[0].source.id, local.id);
}

#[test]
fn rejects_parameter_and_same_scope_binding_collisions() {
    let cases = [
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
