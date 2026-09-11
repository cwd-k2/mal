use super::*;

#[test]
fn parses_parameters_and_lambda_body_items() {
    let expression = binding_value(
        "make := (x, y) {\n\
           sum :: Int32 := x + y;\n\
           observe(sum);\n\
           outer(sum);\n\
         };",
    );
    let Expression::Lambda(lambda) = expression else {
        panic!("expected lambda");
    };
    assert!(matches!(
        lambda.parameter.as_deref(),
        Some(malc::ast::Node {
            kind: Pattern::Product(elements),
            ..
        }) if elements.len() == 2
    ));
    assert!(matches!(lambda.body.items[0], BodyItem::Binding(_)));
    assert!(matches!(lambda.body.items[1], BodyItem::Expression(_)));
    assert!(matches!(lambda.body.result.kind, Expression::Call { .. }));
}

#[test]
fn parses_lambda_patterns_from_parameter_lists() {
    let expression = binding_value("make := ((x, _), y) { x + y };");
    let Expression::Lambda(lambda) = expression else {
        panic!("expected lambda");
    };
    assert!(matches!(
        lambda.parameter.as_deref(),
        Some(malc::ast::Node {
            kind: Pattern::Product(elements),
            ..
        }) if elements.len() == 2
    ));
}

#[test]
fn parses_postfix_and_unit_continuation_applications() {
    for text in ["value := 1[convert];", "value := [initialize];"] {
        assert!(parse(&source(text)).is_ok(), "input should parse: {text}");
    }
    let expression = binding_value("value := choice[first, second];");
    let Expression::ContinuationApplication { continuations, .. } = expression else {
        panic!("expected continuation application");
    };
    assert_eq!(continuations.len(), 2);
}

#[test]
fn parses_if_blocks_with_local_bindings() {
    let expression = binding_value(
        "value := if (condition) then {\n\
           x := 1;\n\
           x\n\
         } else {\n\
           2\n\
         };",
    );
    let Expression::If {
        then_branch,
        else_branch,
        ..
    } = expression
    else {
        panic!("expected if expression");
    };
    assert_eq!(then_branch.items.len(), 1);
    assert!(matches!(then_branch.result.kind, Expression::Name(_)));
    assert!(matches!(else_branch.result.kind, Expression::Integer(_)));
}

#[test]
fn parses_sum_constructor_and_elimination_continuations() {
    let expression = binding_value(
        "value := 1[MaybeInt32](42)[\n\
           () { 0 },\n\
           (x) {\n\
             y := x;\n\
             y\n\
           }];",
    );
    let Expression::ContinuationApplication {
        value,
        continuations,
    } = expression
    else {
        panic!("expected continuation application");
    };
    assert!(matches!(value.kind, Expression::Call { .. }));
    assert_eq!(continuations.len(), 2);
    assert!(matches!(continuations[0].kind, Expression::Lambda(_)));
    let Expression::Lambda(second) = &continuations[1].kind else {
        panic!("expected lambda continuation");
    };
    assert_eq!(second.body.items.len(), 1);
    assert!(matches!(second.body.result.kind, Expression::Name(_)));
}

#[test]
fn rejects_single_member_sums_and_trailing_commas() {
    for text in [
        "Only :: [Unit];",
        "Pair :: [Unit, Int32,];",
        "value := f(1,);",
    ] {
        assert!(parse(&source(text)).is_err(), "input should fail: {text}");
    }
}

#[test]
fn accepts_block_results_with_or_without_a_terminal_semicolon() {
    for text in [
        "value := () { 0 };",
        "value := () { 0; };",
        "value := () { if (true) then { 0; } else { 1 }; };",
        "value := () { 0[Bool]()[() { 0 }, () { 1; }]; };",
    ] {
        assert!(parse(&source(text)).is_ok(), "input should parse: {text}");
    }
}

#[test]
fn rejects_a_lambda_without_a_result_expression() {
    for text in ["value := () {};", "value := () { item := 0; };"] {
        let source = source(text);
        let error = parse(&source).expect_err("a result expression is required");
        assert_eq!(error.message, "expected a block result expression");
    }
}

#[test]
fn treats_return_as_an_ordinary_identifier() {
    assert!(parse(&source("value := () { return := 1; return; };")).is_ok());
    assert!(parse(&source("value := () { return 1; };")).is_err());
}
