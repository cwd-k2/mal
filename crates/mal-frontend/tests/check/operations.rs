use super::*;

#[test]
fn selects_exact_function_and_value_implementations() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;\n\
         zero<A> :: A;\n\
         zero<Int32> :: Int32 := 0;\n\
         main :: Unit -> Int32 := () -> if (equal(20i32, 22i32)) then 0 else zero;",
    );
    let specialized = check::specialize(program).expect("select exact implementations");

    assert_eq!(specialized.program().items.len(), 3);
    assert!(specialized.program().items.iter().all(|item| {
        !matches!(
            item.kind,
            TopItem::GenericBinding(_)
                | TopItem::OperationFamily(_)
                | TopItem::OperationImplementation(_)
        )
    }));
}

#[test]
fn carries_symbolic_operation_requirements_through_generic_specialization() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;\n\
         same<A> :: (A, A) -> Bool := (left, right) -> equal(left, right);\n\
         main :: Unit -> Int32 := () -> if (same(20i32, 20i32)) then 42 else 0;",
    );
    let TopItem::GenericBinding(same) = &program.items[2].kind else {
        panic!("expected generic binding");
    };
    assert_eq!(same.operations.len(), 1);

    check::specialize(program).expect("resolve symbolic operation requirement");
}

#[test]
fn propagates_operation_requirements_through_generic_references() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         inner<A> :: (A, A) -> Bool := (left, right) -> equal(left, right);\n\
         outer<A> :: (A, A) -> Bool := (left, right) -> inner(left, right);",
    );
    let TopItem::GenericBinding(outer) = &program.items[2].kind else {
        panic!("expected outer generic binding");
    };

    assert_eq!(outer.operations.len(), 1);
}

#[test]
fn rejects_duplicate_keys_and_mismatched_implementation_signatures() {
    for (source, message) in [
        (
            "Number :: Int32; equal<A> :: (A, A) -> Bool;\n\
             equal<Number> :: (Number, Number) -> Bool := (left, right) -> left == right;\n\
             equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;",
            "duplicate operation implementation",
        ),
        (
            "equal<A> :: (A, A) -> Bool;\n\
             equal<Int32> :: (Int32, UInt8) -> Bool := (left, right) -> left == right;",
            "type mismatch",
        ),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn reports_a_missing_exact_implementation_when_the_goal_is_reached() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         main :: Unit -> Int32 := () -> if (equal(1u8, 1u8)) then 1 else 0;",
    );
    let error = check::specialize(program).expect_err("missing implementation");

    assert_eq!(error.message, "missing operation implementation");
}

#[test]
fn resolves_nested_exact_operation_dependencies() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;\n\
         different<A> :: (A, A) -> Bool;\n\
         different<Int32> :: (Int32, Int32) -> Bool := (left, right) -> !equal(left, right);\n\
         main :: Unit -> Int32 := () -> if (different(1i32, 2i32)) then 0 else 1;",
    );

    check::specialize(program).expect("resolve nested exact operations");
}
