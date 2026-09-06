use super::*;

#[test]
fn lowers_lambda_statements_and_a_block_result_to_lets_and_a_result() {
    let program = lower_ok(
        "extern mark :: Unit -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern mark();\n\
           value :: Int32 := 7;\n\
           (value);\n\
         };",
    );
    let first_let = lambda_body(&program.bindings[0].value);
    let ExpressionKind::Let {
        binding: statement,
        body: second_let,
    } = &first_let.kind
    else {
        panic!("expected the expression statement let");
    };
    assert!(matches!(statement.pattern, Pattern::Wildcard { .. }));
    assert!(matches!(
        statement.value.kind,
        ExpressionKind::ExternalCall { .. }
    ));

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
        "add :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() { add(20, 22); };",
    );
    let ExpressionKind::Lambda(add) = &program.bindings[0].value.kind else {
        panic!("expected add lambda");
    };
    assert_eq!(
        add.parameter.ty,
        malc::check::ast::Type::Product(vec![
            malc::check::ast::Type::Int32,
            malc::check::ast::Type::Int32,
        ])
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
