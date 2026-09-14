use super::*;

#[test]
fn parses_parameters_and_lambda_body_items() {
    let expression = binding_value(
        "make := (x, y) -> {\n\
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
fn parses_expression_bodies_for_binder_and_control_forms() {
    let Expression::Lambda(lambda) = binding_value("identity := (value) -> value;") else {
        panic!("expected lambda");
    };
    assert!(lambda.body.items.is_empty());
    assert!(matches!(lambda.body.result.kind, Expression::Name(_)));

    let Expression::ResultBlock { body, .. } = binding_value("value := [left, right] -> right(1);")
    else {
        panic!("expected result block");
    };
    assert!(body.items.is_empty());
    assert!(matches!(body.result.kind, Expression::Call { .. }));

    let Expression::If {
        then_branch,
        else_branch,
        ..
    } = binding_value("value := if (condition) then left else right;")
    else {
        panic!("expected if expression");
    };
    assert!(then_branch.items.is_empty());
    assert!(else_branch.items.is_empty());

    let Expression::When { body, .. } = binding_value("value := when (condition) finish();") else {
        panic!("expected when expression");
    };
    assert!(body.items.is_empty());
    assert!(matches!(body.result.kind, Expression::Call { .. }));
}

#[test]
fn parses_return_binders_when_and_empty_forms() {
    let expression = binding_value(
        "choose :: Bool -> [] := (condition)[done, failed] -> { when (condition) { done() }; failed() };",
    );
    let Expression::Lambda(lambda) = expression else {
        panic!("expected lambda");
    };
    assert_eq!(
        lambda
            .return_binders
            .as_ref()
            .expect("return binder group")
            .len(),
        2
    );
    assert!(matches!(
        lambda.body.items[0],
        BodyItem::Expression(malc::ast::Node {
            kind: Expression::When { .. },
            ..
        })
    ));

    let program = parse_ok("Empty :: []; never :: Unit -> Empty := ()[] -> never()[]; ");
    let TopItem::TypeAlias { value, .. } = &program.items[0].kind else {
        panic!("expected type alias");
    };
    assert!(matches!(&value.kind, TypeExpression::Sum(members) if members.is_empty()));
    let TopItem::Binding(binding) = &program.items[1].kind else {
        panic!("expected binding");
    };
    let Expression::Lambda(lambda) = &binding.value.kind else {
        panic!("expected lambda");
    };
    assert!(lambda.return_binders.as_ref().is_some_and(Vec::is_empty));
    assert!(
        matches!(&lambda.body.result.kind, Expression::ContinuationApplication { continuations, .. } if continuations.is_empty())
    );
}

#[test]
fn parses_direct_blocks_and_result_blocks_as_distinct_expressions() {
    let expression = binding_value("value := { local := 1; local };");
    let Expression::Block(block) = expression else {
        panic!("expected a direct block");
    };
    assert_eq!(block.items.len(), 1);

    let expression = binding_value("value := [left, right] -> { right(1) };");
    let Expression::ResultBlock {
        return_binders,
        body,
    } = expression
    else {
        panic!("expected a direct result block");
    };
    assert_eq!(return_binders.len(), 2);
    assert!(matches!(body.result.kind, Expression::Call { .. }));
}

#[test]
fn rejects_legacy_binder_bodies_and_an_empty_result_binder_group() {
    assert!(parse(&source("value := (x) { x }; ")).is_err());
    assert!(parse(&source("value := [done] { done(0) }; ")).is_err());
    assert!(parse(&source("value := [] { 0 };")).is_err());
}

#[test]
fn parses_lambda_patterns_from_parameter_lists() {
    let expression = binding_value("make := ((x, _), y) -> { x + y };");
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
fn parses_receiver_first_calls_as_normal_calls() {
    let Expression::Call { callee, arguments } =
        binding_value("value := source.decode(1).finish();")
    else {
        panic!("expected the outer receiver-first call");
    };
    assert!(matches!(&callee.kind, Expression::Name(name) if name.text == "finish"));
    assert_eq!(arguments.len(), 1);

    let Expression::Call { callee, arguments } = &arguments[0].kind else {
        panic!("expected the inner receiver-first call");
    };
    assert!(matches!(&callee.kind, Expression::Name(name) if name.text == "decode"));
    assert_eq!(arguments.len(), 2);
    assert!(matches!(&arguments[0].kind, Expression::Name(name) if name.text == "source"));
    assert!(matches!(arguments[1].kind, Expression::Integer(_)));
}

#[test]
fn requires_parentheses_on_receiver_first_calls() {
    let error = parse(&source("value := source.decode;")).expect_err("a call requires parentheses");
    assert_eq!(error.message, "expected `(`");
}

#[test]
fn rejects_a_decimal_point_split_before_its_fraction() {
    let error = parse(&source("value := 1\n.5;")).expect_err("a split decimal must not parse");
    assert_eq!(error.message, "expected a function name after `.`");
}

#[test]
fn separates_a_bare_decimal_integer_from_a_receiver_call_dot() {
    for text in ["value := 1.f();", "value := 1 . f();"] {
        assert!(parse(&source(text)).is_ok(), "input should parse: {text}");
    }
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
fn parses_sum_elimination_continuations() {
    let expression = binding_value(
        "value := choice[\n\
           () -> { 0 },\n\
           (x) -> {\n\
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
    assert!(matches!(value.kind, Expression::Name(_)));
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
        "value := () -> { 0 };",
        "value := () -> { 0; };",
        "value := () -> { if (true) then { 0; } else { 1 }; };",
        "value := () -> { false[() -> { 0 }, () -> { 1; }]; };",
    ] {
        assert!(parse(&source(text)).is_ok(), "input should parse: {text}");
    }
}

#[test]
fn rejects_a_lambda_without_a_result_expression() {
    for text in ["value := () -> {};", "value := () -> { item := 0; };"] {
        let source = source(text);
        let error = parse(&source).expect_err("a result expression is required");
        assert_eq!(error.message, "expected a block result expression");
    }
}

#[test]
fn treats_return_as_an_ordinary_identifier() {
    assert!(parse(&source("value := () -> { return := 1; return; };")).is_ok());
    assert!(parse(&source("value := () -> { return 1; };")).is_err());
}
