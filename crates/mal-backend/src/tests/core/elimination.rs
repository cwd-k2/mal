use super::*;

const RESULT: &str = "Res :: [Int32, Unit];\n";

/// A lambda literal continuation is a branch, so no closure appears in the lowered lambda.
fn contains_lambda(expression: &Expression) -> bool {
    format!("{:?}", expression.kind).contains("Lambda(")
}

/// Skips the destructuring of the product parameter and any local bindings before the elimination.
fn after_lets(mut expression: &Expression) -> &Expression {
    while let ExpressionKind::Let { body, .. } = &expression.kind {
        expression = body;
    }
    expression
}

#[test]
fn lowers_a_branch_to_a_case_arm_without_a_closure() {
    let program = lower_ok(&format!(
        "{RESULT}pick :: (Res, Int32) -> Int32 := (r, base) -> r[(n) -> n + base, () -> base];"
    ));
    let body = top_lambda(&program, "pick");
    assert!(!contains_lambda(body), "{body:#?}");
    let (_, arms) = case(after_lets(body));
    assert_eq!(arms.iter().map(|arm| arm.index).collect::<Vec<_>>(), [0, 1]);
    assert!(
        arms.iter()
            .all(|arm| !matches!(arm.value.kind, ExpressionKind::Call { .. }))
    );
}

#[test]
fn keeps_a_function_value_continuation_as_a_call() {
    let program = lower_ok(&format!(
        "{RESULT}pick :: (Res, Int32) -> Int32 := (r, base) -> {{\n\
           double :: Int32 -> Int32 := (n) -> n + n;\n\
           r[double, () -> base]\n\
         }};"
    ));
    let (_, arms) = case(after_lets(top_lambda(&program, "pick")));
    assert!(matches!(arms[0].value.kind, ExpressionKind::Call { .. }));
    assert!(!matches!(arms[1].value.kind, ExpressionKind::Call { .. }));
}

#[test]
fn connects_an_early_exit_in_a_branch_to_the_result_join() {
    let program = lower_ok(&format!(
        "{RESULT}pick :: (Res, Int32) -> Res := (r, base) -> [ok, fail] => {{\n\
           v := r[(n) -> n, () -> fail()];\n\
           ok(v + base)\n\
         }};"
    ));
    let lambda = top_lambda_definition(&program, "pick");
    assert!(!contains_lambda(&lambda.body), "{:#?}", lambda.body);
    assert!(!lambda.joins.is_empty());
    assert!(format!("{:?}", lambda.body.kind).contains("Goto"));
}

#[test]
fn lowers_a_forward_to_the_same_result_positions_as_a_plain_jump() {
    let program = lower_ok(&format!(
        "{RESULT}forward :: Res -> Res := (r) -> [ok, fail] => r[ok, fail];"
    ));
    let lambda = top_lambda_definition(&program, "forward");
    let body = after_lets(&lambda.body);
    assert!(
        matches!(&body.kind, ExpressionKind::Goto { value, .. }
            if matches!(value.kind, ExpressionKind::Reference(_))),
        "{body:#?}"
    );
}

#[test]
fn lowers_result_binder_names_that_move_variants_as_injections() {
    let program = lower_ok(
        "Pair :: [Int32, Int32];\n\
         swap :: Pair -> Pair := (r) -> [first, second] => r[second, first];",
    );
    let lambda = top_lambda_definition(&program, "swap");
    let (_, arms) = case(after_lets(&lambda.body));
    for (arm, expected) in arms.iter().zip([1usize, 0]) {
        let ExpressionKind::Goto { value, .. } = &arm.value.kind else {
            panic!("expected a jump to the result, found {:#?}", arm.value.kind);
        };
        assert!(matches!(
            &value.kind,
            ExpressionKind::SumInjection { index, .. } if *index == expected
        ));
    }
}
