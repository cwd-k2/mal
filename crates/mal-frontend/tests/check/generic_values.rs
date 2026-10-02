use super::*;

#[test]
fn checks_generic_values_before_specialization() {
    let program = check_ok(
        "identity<A> :: A -> A := (value) -> value;\n\
         pair :: (Int32, UInt8) -> (Int32, UInt8) := (number, byte) ->\n\
           (identity<Int32>(number), identity<UInt8>(byte));\n\
         main :: Unit -> Int32 := () -> {\n\
           (number, byte) := pair(40, 2u8);\n\
           number + byte.i32;\n\
         };",
    );

    assert_eq!(
        program.items.len(),
        3,
        "generic definition and checked bindings"
    );
    let TopItem::GenericBinding(identity) = &program.items[0].kind else {
        panic!("expected checked generic definition");
    };
    assert_eq!(identity.parameters.len(), 1);
    assert!(matches!(identity.value.kind, ExpressionKind::Lambda(_)));

    let specialized = check::specialize(program).expect("specialize from main");
    let program = specialized.program();
    assert_eq!(
        program.items.len(),
        4,
        "two bindings plus two reachable instances"
    );
    for (index, expected) in [(2, Type::Int32), (3, Type::UInt8)] {
        let binding = top_binding(program, index);
        assert_eq!(
            binding.value.ty,
            Type::Function {
                parameter: expected.clone().into(),
                result: expected.into(),
            }
        );
        assert!(matches!(binding.value.kind, ExpressionKind::Lambda(_)));
    }

    let ExpressionKind::Lambda(pair) = &top_binding(program, 0).value.kind else {
        panic!("expected pair lambda");
    };
    let check::ast::Pattern::Product { elements, .. } = pair.parameter.as_deref().unwrap() else {
        panic!("expected product parameter");
    };
    let parameter_ids = elements
        .iter()
        .map(|element| match element {
            check::ast::Pattern::Binding { binding, .. } => binding.id,
            _ => panic!("expected binding parameter"),
        })
        .collect::<Vec<_>>();
    for index in 2..=3 {
        let check::ast::Pattern::Binding { binding, .. } = &top_binding(program, index).pattern
        else {
            panic!("expected specialized binding");
        };
        assert!(!parameter_ids.contains(&binding.id));
    }
}

#[test]
fn specializes_only_bindings_reachable_from_main() {
    let program = check_ok(
        "identity<A> :: A -> A := (value) -> value;\n\
         used :: Unit -> Int32 := () -> identity<Int32>(42);\n\
         unused :: Unit -> UInt8 := () -> identity<UInt8>(7u8);\n\
         main :: Unit -> Int32 := () -> used();",
    );
    let specialized = check::specialize(program).expect("specialize reachable graph");
    let names = specialized
        .program()
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            TopItem::Binding(binding) => match &binding.pattern {
                check::ast::Pattern::Binding { binding, .. } => Some(binding.name.text.as_str()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(names, ["used", "main", "identity"]);
}

#[test]
fn specialization_reachability_does_not_depend_on_generic_declarations() {
    for generic in ["", "identity<A> :: A -> A := (value) -> value;\n"] {
        let program = check_ok(&format!(
            "{generic}\
             unused :: Unit -> UInt8 := () -> 7u8;\n\
             used :: Unit -> Int32 := () -> 42;\n\
             main :: Unit -> Int32 := () -> used();"
        ));
        let specialized = check::specialize(program).expect("specialize reachable graph");
        let names = specialized
            .program()
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                TopItem::Binding(binding) => match &binding.pattern {
                    check::ast::Pattern::Binding { binding, .. } => {
                        Some(binding.name.text.as_str())
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(names, ["used", "main"]);
    }
}

#[test]
fn rejects_invalid_generic_value_use_before_specialization() {
    for (source, message) in [
        (
            "identity<A> :: A -> A := (value) -> value; value := identity;",
            "generic type arguments cannot be inferred",
        ),
        (
            "identity<A> :: A -> A := (value) -> value; value := identity<Int32, UInt8>;",
            "generic value argument arity mismatch",
        ),
        (
            "value :: Int32 := 1; other := value<Int32>;",
            "value does not accept type arguments",
        ),
        (
            "double<A> :: A -> A := (value) -> value + value;",
            "numeric operator requires numeric operands",
        ),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn infers_generic_arguments_from_operands_results_and_function_contexts() {
    check_ok(
        "identity<A> :: A -> A := (value) -> value;\n\
         apply<A, B> :: (A -> B, A) -> B := (function, value) -> function(value);\n\
         map<A, B> :: (A, A -> B) -> B := (value, function) -> function(value);\n\
         fromResult :: Unit -> UInt8 := () -> identity(7);\n\
         fromValue :: Unit -> Int32 := () -> apply(identity, 42);\n\
         fromLambda :: Unit -> UInt64 := () -> map(1i32, (value) -> value.u64);\n\
         main :: Unit -> Int32 := () -> identity(42);",
    );
}

#[test]
fn accepts_explicit_constructor_arguments_and_infers_value_types() {
    let program = check_ok(
        "Pair<A> :: (A, A);\n\
         preserve<F, A> :: F<A> -> F<A> := (value) -> value;\n\
         main :: Unit -> Int32 := () -> {\n\
             pair :: Pair<Int32> := (20, 22);\n\
             (left, right) := preserve<Pair>(pair);\n\
             left + right;\n\
         };",
    );

    check::specialize(program).expect("specialize a constructor argument");

    check_ok(
        "Pair<A> :: (A, A);\n\
         preserve<F, A> :: F<A> -> F<A> := (value) -> value;\n\
         main :: Unit -> Int32 := () -> {\n\
             keep :: Pair<Int32> -> Pair<Int32> := preserve<Pair>;\n\
             (left, right) := keep((20i32, 22i32));\n\
             left + right;\n\
         };",
    );

    assert_eq!(
        check_error(
            "Pair<A> :: (A, A);\n\
             preserve<F, A> :: F<A> -> F<A> := (value) -> value;\n\
             main :: Unit -> Pair<Int32> := () -> preserve((20i32, 22i32));"
        )
        .message,
        "generic type arguments cannot be inferred"
    );
}

#[test]
fn carries_applied_constructor_storable_requirements() {
    let program = check_ok(
        "Pair<A> :: (A, A);\n\
         store<F, A> :: (F<A>, USize) -> Buffer<F<A>> := (_, length) -> make<F<A>>(length);\n\
         main :: Unit -> Int32 := () -> { values := store<Pair>((1i32, 2i32), 1usize); (#values).i32; };",
    );
    check::specialize(program).expect("specialize an applied constructor requirement");

    assert_eq!(
        check_error(
            "Callback<A> :: A -> A;\n\
             store<F, A> :: (F<A>, USize) -> Buffer<F<A>> := (_, length) -> make<F<A>>(length);\n\
             main :: Unit -> Int32 := () -> { store<Callback, Int32>((value) -> value, 1usize); 0; };"
        )
        .message,
        "generic application lacks a Storable requirement"
    );
}

#[test]
fn infers_a_generic_continuation_result_parameter_from_the_sum_payload() {
    check_ok(
        "const<A, B> :: A -> B -> A := (value) -> (_) -> value;\n\
         identity<A> :: A -> A := (value) -> value;\n\
         choose :: Bool -> [Unit, UInt8] := (condition) ->\n\
           if (condition) then [none, some] => none() else [none, some] => some(1u8);\n\
         chooseByte :: Bool -> [UInt8, UInt8] := (condition) ->\n\
           if (condition) then [left, right] => left(1u8) else [left, right] => right(2u8);\n\
         unwrap :: [UInt8, UInt8] -> UInt8 := (choice) -> [escape] => {\n\
           value := choice[escape, (identity)];\n\
           escape(value);\n\
         };\n\
         main :: Unit -> Int32 := () ->\n\
           if (choose(true)[const(false), const(true)]) then unwrap(chooseByte(false)).i32 else 1;",
    );
}

#[test]
fn infers_a_generic_lambda_result_from_direct_result_payloads() {
    check_ok(
        "route<A, B> :: (A, A -> [A, B]) -> [A, B] := (value, step) -> step(value);\n\
         main :: Unit -> Int32 := () -> {\n\
           choice := route(1u8, (value) -> [again, done] => { done(42i32) });\n\
           choice[(again) -> 0, (done) -> done];\n\
         };",
    );
}

#[test]
fn inferred_and_explicit_references_share_a_specialization_key() {
    let program = check_ok(
        "identity<A> :: A -> A := (value) -> value;\n\
         main :: Unit -> Int32 := () -> identity(20) + identity<Int32>(22);",
    );
    let specialized = check::specialize(program).expect("specialize shared application");

    assert_eq!(specialized.program().items.len(), 2);
}

#[test]
fn kind_polymorphic_constructors_share_a_canonical_specialization_key() {
    let program = check_ok(
        "Id<X> :: X;\n\
         preserve<F, A> :: F<A> -> F<A> := (value) -> value;\n\
         main :: Unit -> Int32 := () -> preserve<Id, Int32>(20i32) + preserve<Id, Int32>(22i32);",
    );
    let specialized = check::specialize(program).expect("specialize the canonical constructor");

    assert_eq!(specialized.program().items.len(), 2);

    let phantom = check_ok(
        "Id<X> :: X;\n\
         constant<F> :: Unit -> Int32 := () -> 21;\n\
         main :: Unit -> Int32 := () -> constant<Id>() + constant<Id>();",
    );
    let specialized =
        check::specialize(phantom).expect("canonicalize generalized constructor kinds");
    assert_eq!(specialized.program().items.len(), 2);
}

#[test]
fn rejects_missing_and_conflicting_generic_inference() {
    for (source, message) in [
        (
            "hidden<A> :: Unit -> Unit := () -> (); main :: Unit -> Unit := () -> hidden();",
            "generic type arguments cannot be inferred",
        ),
        (
            "same<A> :: (A, A) -> A := (left, _) -> left; main :: Unit -> Int32 := () -> same(1i32, 2u8);",
            "conflicting generic type inference",
        ),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn permits_same_key_recursion_and_rejects_polymorphic_recursion() {
    check_ok(
        "repeat<A> :: (Bool, A) -> A := (again, value) ->\n\
           if (again) then repeat<A>(false, value) else value;\n\
         main :: Unit -> Int32 := () -> repeat<Int32>(true, 7);",
    );

    let error = check_error(
        "recurse<A> :: A -> A := (value) -> {\n\
           recurse<(A, A)>((value, value));\n\
           value;\n\
         };",
    );
    assert_eq!(error.message, "polymorphic recursion is not supported");
}

#[test]
fn gives_each_generic_instance_binders_no_other_instance_shares() {
    let program = check_ok(
        "twice<A> :: (A, A -> [A, A]) -> A := (state, step) -> step(state)[\
           (next) -> next, (done) -> done];\n\
         main :: Unit -> Int32 := () -> {\n\
           number := twice<Int32>(1i32, (value) -> [again, stop] => { stop(value) });\n\
           byte := twice<UInt8>(2u8, (value) -> [again, stop] => { stop(value) });\n\
           number + byte.i32;\n\
         };",
    );
    let specialized = check::specialize(program).expect("specialize both instances");
    let program = specialized.program();
    let parameters = program
        .items
        .iter()
        .filter_map(|item| {
            let TopItem::Binding(binding) = &item.kind else {
                return None;
            };
            let ExpressionKind::Lambda(lambda) = &binding.value.kind else {
                return None;
            };
            let check::ast::Pattern::Product { elements, .. } = lambda.parameter.as_deref()? else {
                return None;
            };
            (elements.len() == 2).then(|| {
                elements
                    .iter()
                    .map(|element| match element {
                        check::ast::Pattern::Binding { binding, .. } => binding.id,
                        _ => panic!("expected binding parameter"),
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(parameters.len(), 2, "one instance per type argument");
    assert!(
        parameters[0].iter().all(|id| !parameters[1].contains(id)),
        "instances must not share parameter binder identities"
    );
}

#[test]
fn nested_inferred_generic_calls_check_each_argument_once() {
    // Each level used to re-check its argument once per inference round, so 20 levels took hours.
    let depth = 20;
    let identity = format!(
        "id<A> :: A -> A := (x) -> x;\nmain :: Unit -> Int32 := () -> {}1i32{};",
        "id(".repeat(depth),
        ")".repeat(depth)
    );
    check_ok(&identity);

    let lambdas = format!(
        "apply<A, B> :: (A -> B, A) -> B := (f, x) -> f(x);\nmain :: Unit -> Int32 := () -> {}x{};",
        (0..depth).map(|_| "apply((x) -> ").collect::<String>(),
        ", 1i32)".repeat(depth)
    );
    check_ok(&lambdas);
}

#[test]
fn nested_inferred_generic_calls_fail_without_rechecking_each_level() {
    // A failed argument probe used to be re-checked on the next inference round, so each level tripled the work.
    let depth = 20;
    let program = format!(
        "apply<A, B> :: (A -> B, A) -> B := (f, x) -> f(x);\nmain :: Unit -> Int32 := () -> {}apply(() -> 1i32, 1i32){};",
        (0..depth).map(|_| "apply((x) -> ").collect::<String>(),
        ", 1i32)".repeat(depth)
    );
    assert_eq!(
        check_error(&program).message,
        "lambda parameters do not match the expected function type"
    );
}
