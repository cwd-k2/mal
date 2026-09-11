use super::*;

#[test]
fn checks_sum_injection_payload_and_index() {
    let program = check_ok(
        "Maybe :: [Unit, Int32];\n\
         some :: Maybe := 1[Maybe](42);",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Sum(vec![Type::Unit, Type::Int32])
    );

    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := 2[Maybe](0);").message,
        "sum variant index is out of range"
    );
    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := 0[Maybe](0);").message,
        "type mismatch"
    );
}

#[test]
fn checks_sum_elimination_arity_and_result_types() {
    check_ok(
        "Maybe :: [Unit, Int32];\n\
         get :: Maybe -> Int32 := (value) {\n\
           value[\n\
             () { 0 },\n\
             (x) { y := x; y }];\n\
         };",
    );

    let prefix = "Maybe :: [Unit, Int32]; get :: Maybe -> Int32 := (value) { ";
    assert_eq!(
        check_error(&format!(
            "{prefix}value[() {{ 0 }}, (x) {{ x }}, (_) {{ 1 }}]; }};"
        ))
        .message,
        "sum continuation count does not match its type"
    );
    assert_eq!(
        check_error(&format!("{prefix}value[() {{ () }}, (x) {{ x }}]; }};")).message,
        "type mismatch"
    );
}

#[test]
fn checks_if_condition_and_branch_types() {
    check_ok(
        "choose :: Bool -> Int32 := (condition) {\n\
           if (condition) then { 1 } else { 2 };\n\
         };",
    );
    assert_eq!(
        check_error(
            "bad :: Int32 -> Int32 := (condition) {\n\
               if (condition) then { 1 } else { 2 };\n\
             };"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "bad :: Bool -> Int32 := (condition) {\n\
               if (condition) then { 1 } else { () };\n\
             };"
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_explicit_return_binders_and_completion() {
    let program = check_ok(
        "Result :: [Int32, Symbol];\n\
         compute :: Bool -> Result := (enabled)[ok, err] {\n\
           when (enabled) { ok(42) };\n\
           err(\"disabled\")\n\
         };",
    );
    let ExpressionKind::Lambda(lambda) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(lambda.return_binders.as_ref().expect("binders").len(), 2);
    assert!(matches!(
        lambda.body.result.as_ref(),
        check::ast::Completion::Abrupt(_)
    ));

    check_ok(
        "absolute :: Int32 -> Int32 := (x)[return] {\n\
           when (x >= 0) { return(x) };\n\
           return(-x)\n\
         };",
    );
}

#[test]
fn checks_empty_elimination_without_conflating_it_with_abrupt_completion() {
    check_ok("never :: Unit -> [] := ()[] { never()[] };");
    assert_eq!(
        check_error("bad :: Unit -> [] := () { bad()[] };").message,
        "lambda without return binders must produce a value"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[] { 1 };").message,
        "empty return binder group requires `[]` result"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] { 1 };").message,
        "lambda with return binders cannot fall through"
    );
}

#[test]
fn rejects_return_binder_arity_and_value_use() {
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[left, right] { left(1) };").message,
        "multiple return binders require a sum result"
    );
    assert_eq!(
        check_error("Choice :: [Int32, Symbol]; bad :: Unit -> Choice := ()[a, b, c] { a(1) };")
            .message,
        "return binder count does not match the sum result"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] { value := return; return(1) };").message,
        "return binder is not a value"
    );
}

#[test]
fn preserves_completion_through_strict_and_short_circuit_contexts() {
    check_ok(
        "finish :: Bool -> Int32 := (condition)[return] {\n\
           return(1 + if (condition) then { return(10) } else { 2 })\n\
         };",
    );
    assert_eq!(
        check_error("bad :: Bool -> Bool := (condition)[return] { condition && return(true) };")
            .message,
        "lambda with return binders cannot fall through"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] { (return(1), 2) };").message,
        "unreachable expression after abrupt completion"
    );
}

#[test]
fn reports_the_first_item_after_abrupt_completion_as_unreachable() {
    let text = "bad :: Unit -> Int32 := ()[return] { return(1); 2; 3 };";
    let diagnostic = check_error(text);
    assert_eq!(
        diagnostic.message,
        "unreachable code after abrupt completion"
    );
    assert_eq!(
        diagnostic.primary.expect("primary label").span.start(),
        text.find("2; 3").expect("unreachable expression")
    );
}
