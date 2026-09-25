use super::*;
use check::ast::{AbruptExpressionKind, BodyItem, Completion, SumContinuation};

const RESULT: &str = "Res :: [Int32, Unit];\n";

fn program(body: &str) -> check::ast::Program {
    check_ok(&format!(
        "{RESULT}pick :: (Res, Int32) -> Res := (s, base) -> [ok, fail] => {{ {body} }};"
    ))
}

/// The block of the direct result block that wraps the body of `pick`.
fn result_block(program: &check::ast::Program) -> &check::ast::ExpressionBlock {
    let ExpressionKind::Lambda(lambda) = &top_binding(program, 1).value.kind else {
        panic!("expected lambda");
    };
    let ExpressionKind::ResultBlock { body, .. } = &completion_value(&lambda.body.result).kind
    else {
        panic!("expected a result block");
    };
    body
}

fn first_continuations(program: &check::ast::Program) -> &[SumContinuation] {
    let BodyItem::Binding(binding) = &result_block(program).items[0] else {
        panic!("expected a binding");
    };
    let ExpressionKind::SumElimination { continuations, .. } = &binding.value.kind else {
        panic!("expected a sum elimination");
    };
    continuations
}

#[test]
fn joins_a_value_branch_with_an_abrupt_branch() {
    let program = program("v := s[(n) -> n + base, () -> fail()]; ok(v)");
    let BodyItem::Binding(binding) = &result_block(&program).items[0] else {
        panic!("expected a binding");
    };
    assert_eq!(binding.value.ty, Type::Int32);
    let [first, second] = first_continuations(&program) else {
        panic!("expected two continuations");
    };
    assert!(matches!(first, SumContinuation::Branch(branch) if branch.parameter.is_some()));
    assert!(matches!(second, SumContinuation::Branch(branch)
        if branch.parameter.is_none()
            && matches!(branch.body.result.as_ref(), Completion::Abrupt(_))));
}

#[test]
fn a_sum_elimination_of_abrupt_continuations_is_abrupt() {
    let program = program("s[(n) -> ok(n), () -> fail()]");
    let Completion::Abrupt(abrupt) = result_block(&program).result.as_ref() else {
        panic!("expected an abrupt completion");
    };
    assert!(matches!(
        abrupt.kind,
        AbruptExpressionKind::SumElimination { .. }
    ));
}

#[test]
fn result_binder_names_transfer_the_payload() {
    let program = program("s[ok, fail]");
    let Completion::Abrupt(abrupt) = result_block(&program).result.as_ref() else {
        panic!("expected an abrupt completion");
    };
    let AbruptExpressionKind::SumElimination { continuations, .. } = &abrupt.kind else {
        panic!("expected a sum elimination");
    };
    let variants: Vec<_> = continuations
        .iter()
        .map(|continuation| match continuation {
            SumContinuation::Transfer(transfer) => transfer.variant,
            _ => panic!("expected a transfer"),
        })
        .collect();
    assert_eq!(variants, [Some(0), Some(1)]);
}

#[test]
fn a_binder_name_may_stand_in_any_position_whose_type_matches() {
    check_ok(&format!(
        "{RESULT}swap :: (Res, Int32) -> Int32 := (s, base) -> [k] => {{ s[(n) -> k(n), () -> k(base)] }};\n\
         wrap :: Res -> Res := (s) -> [ok, fail] => {{ s[ok, fail] }};"
    ));
}

#[test]
fn binds_product_payloads_and_ignores_payloads_with_a_wildcard() {
    check_ok(
        "Pair :: [(Int32, Int32), Unit];\n\
         sum :: Pair -> Int32 := (p) -> [k] => { v := p[(a, b) -> a + b, () -> k(0)]; k(v) };\n\
         zero :: Pair -> Int32 := (p) -> [k] => { v := p[(_) -> 1i32, () -> k(0)]; k(v) };",
    );
}

#[test]
fn rejects_continuation_type_and_shape_errors() {
    let cases = [
        ("v := s[(n) -> n, () -> true]; ok(v)", "type mismatch"),
        (
            "v := s[fail, ok]; v",
            "result binder does not accept this payload",
        ),
        (
            "v := s[(n) -> n, (x) -> 0]; ok(v)",
            "lambda parameters do not match",
        ),
        (
            "v := s[() -> 0, () -> 0]; ok(v)",
            "lambda parameters do not match",
        ),
        (
            "v := s[ok, fail]; ok(v)",
            "binding initializer must produce a value",
        ),
    ];
    for (body, message) in cases {
        let source = format!(
            "{RESULT}pick :: (Res, Int32) -> Res := (s, base) -> [ok, fail] => {{ {body} }};"
        );
        assert!(
            check_error(&source).message.contains(message),
            "input: {body}"
        );
    }
}

#[test]
fn explains_why_an_abrupt_elimination_cannot_initialize_a_binding() {
    let error = check_error(&format!(
        "{RESULT}pick :: (Res, Int32) -> Res := (s, base) -> [ok, fail] => {{ v := s[ok, fail]; v }};"
    ));
    assert_eq!(error.message, "binding initializer must produce a value");
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("every continuation leaves the block"),
        "{rendered}"
    );
    assert!(
        rendered.contains("let one branch or continuation produce it"),
        "{rendered}"
    );
}

#[test]
fn names_the_binder_and_position_when_a_payload_does_not_fit() {
    let error = check_error(&format!(
        "{RESULT}pick :: (Res, Int32) -> Res := (s, base) -> [ok, fail] => s[fail, ok];"
    ));
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("`fail` takes `Unit`, but this continuation receives `Int32`"),
        "{rendered}"
    );
    assert!(
        rendered.contains("continuation 0 of the sum carries `Int32`"),
        "{rendered}"
    );
}
