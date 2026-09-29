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

#[test]
fn selects_non_overlapping_generic_implementation_patterns() {
    let program = check_ok(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int32> :: (Int32, Int32) -> Bool := (left, right) -> left == right;\n\
         equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool := (left, right) -> #left == #right;\n\
         main :: Unit -> Int32 := () -> {\n\
             values :: Buffer<Int32> := make<Int32>(0usize);\n\
             if (equal(values, values)) then 0 else 1;\n\
         };",
    );

    check::specialize(program).expect("select generic Buffer implementation");
}

#[test]
fn rejects_overlapping_and_non_decreasing_generic_implementations() {
    for (source, message) in [
        (
            "equal<A> :: (A, A) -> Bool;\n\
             equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool := (left, right) -> true;\n\
             equal<Buffer<Int32>> :: (Buffer<Int32>, Buffer<Int32>) -> Bool := (left, right) -> true;",
            "duplicate operation implementation",
        ),
        (
            "equal<A> :: (A, A) -> Bool;\n\
             equal<A> :: (A, A) -> Bool := (left, right) -> true;",
            "generic operation implementation requires structure",
        ),
        (
            "equal<A> :: (A, A) -> Bool;\n\
             other<A> :: (A, A) -> Bool;\n\
             equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool :=\n\
                 (left, right) -> other<Buffer<A>>(left, right);",
            "generic operation requirement does not decrease",
        ),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn generic_patterns_bind_every_family_parameter_and_respect_repeated_variables() {
    assert_eq!(
        check_error(
            "operation<A, B> :: Unit;\n\
             operation<Buffer<A>, Int32> :: Unit := ();"
        )
        .message,
        "generic operation pattern leaves a parameter unbound"
    );

    check_ok(
        "operation<A, B> :: Unit;\n\
         operation<(A, A), Buffer<B>> :: Unit := ();\n\
         operation<(Int32, UInt8), Buffer<(A, B)>> :: Unit := ();",
    );
}

#[test]
fn overlap_check_uses_independent_variables_for_each_pattern() {
    let error = check_error(
        "operation<A, B> :: Unit;\n\
         operation<(A, Int32), Buffer<B>> :: Unit := ();\n\
         operation<(UInt8, A), Buffer<B>> :: Unit := ();",
    );

    assert_eq!(error.message, "duplicate operation implementation");
}

#[test]
fn opaque_identity_is_preserved_in_operation_keys() {
    let program = check_ok(
        "opaque Values<A> :: Buffer<A>;\n\
         equal<A> :: (A, A) -> Bool;\n\
         equal<Buffer<A>> :: (Buffer<A>, Buffer<A>) -> Bool := (left, right) -> #left == #right;\n\
         makeValues<A> :: USize -> Values<A> := (length) -> make<A>(length);\n\
         main :: Unit -> Int32 := () -> {\n\
             values :: Values<Int32> := makeValues<Int32>(0usize);\n\
             if (equal(values, values)) then 0 else 1;\n\
         };",
    );

    assert_eq!(
        check::specialize(program)
            .expect_err("Buffer pattern must not match opaque Values")
            .message,
        "missing operation implementation"
    );
}
