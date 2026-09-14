use super::*;

#[test]
fn checks_expression_bodies_for_binder_and_control_forms() {
    check_ok(
        "identity :: Int32 -> Int32 := (value) -> value;\n\
         choose :: Bool -> Int32 := (condition) -> if (condition) then 1 else 2;\n\
         finish :: Bool -> Int32 := (condition)[return] -> {\n\
           when (condition) return(1);\n\
           return(0);\n\
         };\n\
         result :: Unit -> Int32 := () -> [done] -> done(0);",
    );
}

#[test]
fn constructs_sum_values_through_return_binders() {
    let program = check_ok(
        "Maybe :: [Unit, Int32];\n\
         none :: Unit -> Maybe := ()[none, some] -> [none];\n\
         some :: Int32 -> Maybe := (value)[none, some] -> some(value);\n\
         makeTrue :: Unit -> Bool := () -> [()[cont0, cont1] -> [cont1]];",
    );
    assert!(matches!(
        top_binding(&program, 1).value.ty,
        Type::Function { .. }
    ));
    assert!(matches!(
        top_binding(&program, 2).value.ty,
        Type::Function { .. }
    ));
    assert!(matches!(
        top_binding(&program, 3).value.ty,
        Type::Function { .. }
    ));

    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := 1[Maybe](0);").message,
        "sum values must be constructed through return binders"
    );
}

#[test]
fn checks_sum_elimination_arity_and_result_types() {
    check_ok(
        "Maybe :: [Unit, Int32];\n\
         get :: Maybe -> Int32 := (value) -> {\n\
           value[\n\
             () -> { 0 },\n\
             (x) -> { y := x; y }];\n\
         };",
    );

    let prefix = "Maybe :: [Unit, Int32]; get :: Maybe -> Int32 := (value) -> { ";
    assert_eq!(
        check_error(&format!(
            "{prefix}value[() -> {{ 0 }}, (x) -> {{ x }}, (_) -> {{ 1 }}]; }};"
        ))
        .message,
        "sum continuation count does not match its type"
    );
    assert_eq!(
        check_error(&format!(
            "{prefix}value[() -> {{ () }}, (x) -> {{ x }}]; }};"
        ))
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_if_condition_and_branch_types() {
    check_ok(
        "choose :: Bool -> Int32 := (condition) -> {\n\
           if (condition) then { 1 } else { 2 };\n\
         };",
    );
    assert_eq!(
        check_error(
            "bad :: Int32 -> Int32 := (condition) -> {\n\
               if (condition) then { 1 } else { 2 };\n\
             };"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "bad :: Bool -> Int32 := (condition) -> {\n\
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
         compute :: Bool -> Result := (enabled)[ok, err] -> {\n\
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
        "absolute :: Int32 -> Int32 := (x)[return] -> {\n\
           when (x >= 0) { return(x) };\n\
           return(-x)\n\
         };",
    );
}

#[test]
fn checks_direct_blocks_without_treating_them_as_lambdas() {
    check_ok(
        "main :: Unit -> Int32 := ()[return] -> {
           value :: Int32 := { when (false) { return(9) }; local :: Int32 := 40; local + 2 };
           return(value)
         };",
    );
    assert_eq!(
        check_error("bad :: Unit -> (Unit -> Int32) := () -> { { 1 } };").message,
        "type mismatch"
    );
}

#[test]
fn checks_direct_result_blocks_against_their_expected_type() {
    let program = check_ok(
        "Choice :: [Int32, Symbol];
         choose :: Bool -> Choice := (condition) -> {
           [integer, symbol] -> {
             when (condition) { integer(42) };
             symbol(\"no\")
           }
         };
         answer :: Unit -> Int32 := () -> { [done] -> { done(42) } };",
    );
    let ExpressionKind::Lambda(choose) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    let check::ast::Completion::Value(value) = choose.body.result.as_ref() else {
        panic!("expected value completion");
    };
    assert!(matches!(value.kind, ExpressionKind::ResultBlock { .. }));

    assert_eq!(
        check_error("bad :: Unit -> Int32 := () -> { value := [done] -> { done(1) }; value };")
            .message,
        "result block requires an expected result type"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := () -> { [done] -> { 1 } };").message,
        "result block cannot fall through"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] -> { [done] -> { return(1) } };").message,
        "result block does not produce a result"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] -> { [done] -> { done(return(1)) } };")
            .message,
        "result block does not produce a result"
    );
}

#[test]
fn checks_empty_elimination_without_conflating_it_with_abrupt_completion() {
    check_ok("never :: Unit -> [] := ()[] -> never()[];");
    assert_eq!(
        check_error("bad :: Unit -> [] := () -> { bad()[] };").message,
        "lambda without return binders must produce a value"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[] -> { 1 };").message,
        "empty return binder group requires `[]` result"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] -> { 1 };").message,
        "lambda with return binders cannot fall through"
    );
}

#[test]
fn rejects_return_binder_arity_and_value_use() {
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[left, right] -> { left(1) };").message,
        "multiple return binders require a sum result"
    );
    assert_eq!(
        check_error("Choice :: [Int32, Symbol]; bad :: Unit -> Choice := ()[a, b, c] -> { a(1) };")
            .message,
        "return binder count does not match the sum result"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] -> { value := return; return(1) };")
            .message,
        "return binder is not a value"
    );
}

#[test]
fn preserves_completion_through_strict_and_short_circuit_contexts() {
    check_ok(
        "finish :: Bool -> Int32 := (condition)[return] -> {\n\
           return(1 + if (condition) then { return(10) } else { 2 })\n\
         };",
    );
    assert_eq!(
        check_error("bad :: Bool -> Bool := (condition)[return] -> { condition && return(true) };")
            .message,
        "lambda with return binders cannot fall through"
    );
    assert_eq!(
        check_error("bad :: Unit -> Int32 := ()[return] -> { (return(1), 2) };").message,
        "unreachable expression after abrupt completion"
    );
}

#[test]
fn reports_the_first_item_after_abrupt_completion_as_unreachable() {
    let text = "bad :: Unit -> Int32 := ()[return] -> { return(1); 2; 3 };";
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
