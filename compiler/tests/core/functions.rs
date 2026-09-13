use super::*;

#[test]
fn lowers_lambda_statements_and_a_block_result_to_lets_and_a_result() {
    let program = lower_ok(
        "extern mark :: Unit -> Unit;\n\
         main :: Unit -> Int32 := () {\n\
           mark();\n\
           value :: Int32 := 7;\n\
           (value);\n\
         };",
    );
    let first_let = top_lambda(&program, "main");
    let ExpressionKind::Let {
        binding: statement,
        body: second_let,
    } = &first_let.kind
    else {
        panic!("expected the expression statement let");
    };
    assert!(matches!(statement.pattern, Pattern::Wildcard { .. }));
    assert!(matches!(statement.value.kind, ExpressionKind::Call { .. }));

    let ExpressionKind::Let {
        binding,
        body: result,
    } = &second_let.kind
    else {
        panic!("expected the local binding let");
    };
    let Pattern::Binding { id, .. } = binding.pattern else {
        panic!("expected a named local binding");
    };
    assert!(matches!(
        result.kind,
        ExpressionKind::Reference(result_id) if result_id == id
    ));
}

#[test]
fn lowers_multiple_parameters_to_product_destructuring() {
    let program = lower_ok(
        "add :: (Int32, Int32) -> Int32 := (left, right) {\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := () { add(20, 22); };",
    );
    let ExpressionKind::Lambda(add) = &program.bindings[0].value.kind else {
        panic!("expected add lambda");
    };
    assert_eq!(
        add.parameter.ty,
        malc::check::ast::Type::Product(
            vec![malc::check::ast::Type::Int32, malc::check::ast::Type::Int32,].into()
        )
    );
    let ExpressionKind::Let { binding, .. } = &add.body.kind else {
        panic!("multiple parameters should be destructured at function entry");
    };
    assert!(matches!(binding.pattern, Pattern::Product { .. }));

    let main = lambda_body(&program.bindings[1].value);
    let ExpressionKind::Call { argument, .. } = &main.kind else {
        panic!("expected add call");
    };
    assert!(matches!(argument.kind, ExpressionKind::Product(_)));
}

#[test]
fn lowers_explicit_returns_to_the_existing_lambda_result_edge() {
    let program = lower_ok(
        "absolute :: Int32 -> Int32 := (x)[return] {\n\
           when (x >= 0) { return(x) };\n\
           return(-x)\n\
         };",
    );
    let body = top_lambda(&program, "absolute");
    let ExpressionKind::Case { arms, .. } = &body.kind else {
        panic!("when should lower to a branch over the remaining continuation");
    };
    assert!(matches!(arms[0].value.kind, ExpressionKind::Goto { .. }));
    let joins = &top_lambda_definition(&program, "absolute").joins;
    assert_eq!(joins.len(), 1);
    let ExpressionKind::Let {
        body: remaining, ..
    } = &joins[0].body.kind
    else {
        panic!("the shared continuation should discard when's Unit");
    };
    assert!(matches!(
        remaining.kind,
        ExpressionKind::PrimitiveUnary { .. }
    ));
    assert!(matches!(arms[1].value.kind, ExpressionKind::Reference(_)));
}

#[test]
fn lowers_long_flat_completion_control_sequences_to_shared_joins() {
    let text = format!(
        "main :: Unit -> Int32 := ()[return] {{ {}return(0i32) }};",
        "when (false) { return(1i32) };".repeat(4_096)
    );

    let program = lower_ok(&text);

    assert_eq!(top_lambda_definition(&program, "main").joins.len(), 4_096);
}

#[test]
fn shares_continuations_across_multiple_normal_branch_exits() {
    let count = 128;
    let item = "if (true) then { when (false) { return(1i32) }; () } else { () };";
    let text = format!(
        "main :: Unit -> Int32 := ()[return] {{ {}return(0i32) }};",
        item.repeat(count)
    );

    let program = lower_ok(&text);

    assert_eq!(
        top_lambda_definition(&program, "main").joins.len(),
        count * 2
    );
}

#[test]
fn lowers_long_flat_prefixes_before_completion_control_iteratively() {
    let text = format!(
        "main :: Unit -> Int32 := ()[return] {{ {}when (false) {{ return(1i32) }}; return(0i32) }};",
        "0i32;".repeat(4_096)
    );

    let program = lower_ok(&text);

    assert!(matches!(
        top_lambda(&program, "main").kind,
        ExpressionKind::Let { .. }
    ));
}

#[test]
fn lowers_empty_elimination_to_a_zero_arm_case() {
    let program = lower_ok("never :: Unit -> [] := ()[] { never()[] };");
    let body = top_lambda(&program, "never");
    let ExpressionKind::Case { arms, .. } = &body.kind else {
        panic!("expected empty case");
    };
    assert!(arms.is_empty());
}
