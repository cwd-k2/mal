use super::*;

fn outer_lambda(program: &resolved::Program) -> &resolved::Lambda {
    let resolved::Expression::Lambda(outer) = &top_binding(&program.items[0]).value.kind else {
        panic!("expected the outer lambda");
    };
    outer
}

/// The body items of a lambda, looking through a direct result block that wraps them.
fn body_items(lambda: &resolved::Lambda) -> &[resolved::BodyItem] {
    match &lambda.body.result.kind {
        resolved::Expression::ResultBlock { body, .. } => &body.items,
        _ => &lambda.body.items,
    }
}

fn first_continuations(lambda: &resolved::Lambda) -> &[resolved::Continuation] {
    let resolved::BodyItem::Binding(binding) = &body_items(lambda)[0] else {
        panic!("expected the continuation binding");
    };
    let resolved::Expression::ContinuationApplication { continuations, .. } =
        &binding.kind.value.kind
    else {
        panic!("expected a continuation application");
    };
    continuations
}

#[test]
fn sum_continuation_lambda_literal_belongs_to_the_enclosing_lambda() {
    let program =
        resolve_ok("f := (s, base) -> [k] => { v := s[(n) -> n + base, () -> k(base)]; k(v) };");
    let outer = outer_lambda(&program);
    assert!(outer.captures.is_empty());
    let [first, second] = first_continuations(outer) else {
        panic!("expected two continuations");
    };
    let resolved::Continuation::Branch(first) = first else {
        panic!("expected a branch");
    };
    assert!(first.parameter.is_some());
    let resolved::Continuation::Branch(second) = second else {
        panic!("expected a branch");
    };
    assert!(second.parameter.is_none());
}

#[test]
fn parenthesized_lambda_literal_is_still_a_branch() {
    let program = resolve_ok("f := (s) -> [k] => { v := s[((n) -> k(n)), () -> k(0)]; v };");
    let [first, _] = first_continuations(outer_lambda(&program)) else {
        panic!("expected two continuations");
    };
    assert!(matches!(first, resolved::Continuation::Branch(_)));
}

#[test]
fn only_a_sum_elimination_has_branches() {
    let program = resolve_ok("f := (s) -> { v := s[(n) -> n]; v };");
    let [only] = first_continuations(outer_lambda(&program)) else {
        panic!("expected one continuation");
    };
    assert!(matches!(only, resolved::Continuation::Function(_)));
}

#[test]
fn result_binder_names_stay_references_in_continuation_position() {
    let program = resolve_ok("f := (s) -> [a, b] => { v := s[a, b]; v };");
    let continuations = first_continuations(outer_lambda(&program));
    assert!(
        continuations
            .iter()
            .all(|continuation| matches!(continuation, resolved::Continuation::Function(_)))
    );
}

#[test]
fn branch_parameter_is_scoped_to_its_branch() {
    let error = resolve_error("f := (s) -> [k] => { v := s[(n) -> n, () -> n]; k(v) };");
    assert!(error.message.starts_with("unknown value"));
}

#[test]
fn lambdas_inside_a_branch_still_cannot_capture_result_binders() {
    assert_eq!(
        resolve_error("f := (s) -> [k] => { v := s[(n) -> () -> k(n), () -> k(0)]; k(v) };")
            .message,
        "result binder cannot be captured"
    );
    assert_eq!(
        resolve_error("f := (s) -> [k] => { h := (n) -> k(n); v := s[h, () -> k(0)]; k(v) };")
            .message,
        "result binder cannot be captured"
    );
}
