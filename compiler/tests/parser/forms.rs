use super::*;

#[test]
fn parses_captures_parameters_and_lambda_body_items() {
    let expression = binding_value(
        "make := \\<outer>(x :: Int32, y :: Int32) {\n\
           sum :: Int32 := x + y;\n\
           extern observe(sum);\n\
           outer(sum);\n\
         };",
    );
    let Expression::Lambda(lambda) = expression else {
        panic!("expected lambda");
    };
    assert_eq!(lambda.captures[0].text, "outer");
    assert_eq!(lambda.parameters.len(), 2);
    assert!(matches!(lambda.body.items[0], BodyItem::Binding(_)));
    assert!(matches!(lambda.body.items[1], BodyItem::Expression(_)));
    assert!(matches!(lambda.body.result.kind, Expression::Call { .. }));
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
fn parses_sum_injection_and_case_arms() {
    let expression = binding_value(
        "value := case (MaybeInt32[1](42))\n\
           [0](_) { 0 }\n\
           [1](x) {\n\
             y := x;\n\
             y\n\
           };",
    );
    let Expression::Case { scrutinee, arms } = expression else {
        panic!("expected case expression");
    };
    assert!(matches!(scrutinee.kind, Expression::SumInjection { .. }));
    assert_eq!(arms.len(), 2);
    assert!(matches!(arms[0].pattern.kind, Pattern::Wildcard));
    assert!(matches!(arms[1].pattern.kind, Pattern::Name(_)));
    assert!(arms[0].body.items.is_empty());
    assert_eq!(arms[1].body.items.len(), 1);
    assert!(matches!(arms[1].body.result.kind, Expression::Name(_)));
}

#[test]
fn rejects_case_forms_outside_the_grammar() {
    for text in [
        "value := case (value) [0](_) => 0;;",
        "value := case value { [0](_) { 0 } };",
    ] {
        assert!(parse(&source(text)).is_err(), "input should fail: {text}");
    }
}

#[test]
fn rejects_single_member_sums_and_trailing_commas() {
    for text in [
        "Only :: [Unit];",
        "Pair :: [Unit, Int32,];",
        "value := f(1,);",
        "value := \\<>() { 0; };",
    ] {
        assert!(parse(&source(text)).is_err(), "input should fail: {text}");
    }
}

#[test]
fn accepts_block_results_with_or_without_a_terminal_semicolon() {
    for text in [
        "value := \\() { 0 };",
        "value := \\() { 0; };",
        "value := \\() { if (true) then { 0; } else { 1 }; };",
        "value := \\() { case (Bool[0](())) [0](_) { 0 } [1](_) { 1; }; };",
    ] {
        assert!(parse(&source(text)).is_ok(), "input should parse: {text}");
    }
}

#[test]
fn rejects_a_lambda_without_a_result_expression() {
    for text in ["value := \\() {};", "value := \\() { item := 0; };"] {
        let source = source(text);
        let error = parse(&source).expect_err("a result expression is required");
        assert_eq!(error.message, "expected a block result expression");
    }
}

#[test]
fn treats_return_as_an_ordinary_identifier() {
    assert!(parse(&source("value := \\() { return := 1; return; };")).is_ok());
    assert!(parse(&source("value := \\() { return 1; };")).is_err());
}
