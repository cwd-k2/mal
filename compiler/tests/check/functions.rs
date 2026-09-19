use super::*;

#[test]
fn rejects_binding_and_block_result_type_mismatches() {
    assert_eq!(check_error("value :: Unit := 0;").message, "type mismatch");
    assert_eq!(
        check_error("main :: Unit -> Unit := () -> { 0; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_function_application_and_zero_argument_unit_lowering() {
    let program = check_ok(
        "identity :: Int32 -> Int32 := (x) -> { x; };\n\
         thunk :: Unit -> Int32 := () -> { identity(4); };\n\
         caller :: Unit -> Int32 := () -> { thunk(); };",
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Unit).into(),
            result: Box::new(Type::Int32).into(),
        }
    );

    assert_eq!(
        check_error(
            "identity :: Int32 -> Int32 := (x) -> { x; };\n\
             bad :: Int32 := identity();"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error("value :: Int32 := 1i32();").message,
        "cannot call a non-function value"
    );
}

#[test]
fn rejects_multi_argument_application_with_the_wrong_arity() {
    assert_eq!(
        check_error(
            "takePair :: (Int32, Int32) -> Int32 := (left, _) -> { left; };\n\
             bad :: Int32 := takePair(1, 2, 3);"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "observe :: (Int32, Int32) -> Int32 := (_, value) -> { value; };\n\
             bad :: Int32 := 1.observe(2, 3);"
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_return_as_an_ordinary_local_name() {
    check_ok("value :: Unit -> Int64 := () -> { return := 1; return; };");
}

#[test]
fn checks_products_destructuring_and_multiple_parameters() {
    let program = check_ok(
        "Pair :: (Int32, UInt8);\n\
         pair :: Pair := (1, 2);\n\
         add :: (Int32, Int32) -> Int32 := (left, right) -> {\n\
           left + right;\n\
         };\n\
         firstValue :: Pair -> Int32 := (pair) -> {\n\
           (value, _) := pair;\n\
           value;\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           (first, _) := pair;\n\
           nested := ((first, 2i32), 39i32);\n\
           ((left, right), extra) := nested;\n\
           add(left + right, extra);\n\
         };",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Product(vec![Type::Int32, Type::UInt8].into())
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Product(vec![Type::Int32, Type::Int32].into())).into(),
            result: Box::new(Type::Int32).into(),
        }
    );
    let ExpressionKind::Lambda(first_value) = &top_binding(&program, 3).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(
        first_value.parameter_type,
        Type::Product(vec![Type::Int32, Type::UInt8].into())
    );

    assert_eq!(
        check_error("pair := (1, 2); (first, second, third) := pair;").message,
        "product pattern has the wrong arity"
    );
    assert_eq!(
        check_error("value := 1; (first, second) := value;").message,
        "product pattern requires a product value"
    );
}

#[test]
fn checks_top_level_and_local_self_recursion_against_the_annotation() {
    let program = check_ok(
        "count :: Int64 -> Int64 := (n) -> {\n\
           if (n == 0) then { 0 } else { count(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           local :: Int32 -> Int32 := (n) -> {\n\
             if (n == 0) then { 0 } else { local(n - 1) };\n\
           };\n\
           local(10);\n\
         };",
    );

    for index in [0, 1] {
        let ExpressionKind::Lambda(lambda) = &top_binding(&program, index).value.kind else {
            panic!("expected lambda");
        };
        assert!(lambda.self_binding.is_some());
    }
}

#[test]
fn requires_an_expected_function_type_and_matching_parameter_shape() {
    assert_eq!(
        check_error("identity := (value) -> { value; };").message,
        "lambda requires an expected function type"
    );
    for text in [
        "bad :: Unit -> Unit := (value) -> { (); };",
        "bad :: Int32 -> Int32 := () -> { 0; };",
    ] {
        assert_eq!(
            check_error(text).message,
            "lambda parameters do not match the expected function type",
            "input: {text}"
        );
    }
    assert_eq!(
        check_error("bad :: (Int32, Int32) -> Int32 := (left, middle, right) -> { left; };")
            .message,
        "product pattern has the wrong arity"
    );
}

#[test]
fn checks_a_lambda_from_an_application_context() {
    check_ok(
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> {
         function(value);
         };
         main :: Unit -> Int32 := () -> {
           apply((value) -> { value + 1; }, 41);
           ((value) -> { value + 1; }, 41)[apply];
         };",
    );
}

#[test]
fn checks_postfix_application_and_sum_continuations() {
    check_ok(
        "Choice :: [Int32, Symbol];
         first :: Int32 -> Choice := (value) -> [first, second] => { first(value) };
         second :: Symbol -> Choice := (value) -> [first, second] => { second(value) };
         identity :: Int32 -> Int32 := (value) -> { value };
         choose :: Choice -> Int32 := (choice) -> {
           choice[(value) -> { value }, (symbol) -> { (#symbol).i32 }]
         };
         initialize :: Unit -> Int32 := () -> { 42 };
         main :: Unit -> Int32 := () -> {
           forward := identity(42);
           backward := 42[identity];
           initialized := [initialize];
           forward + backward - initialized
         };",
    );
}

#[test]
fn checks_receiver_first_calls_with_ordinary_function_bindings() {
    check_ok(
        "add :: (Int32, Int32) -> Int32 := (left, right) -> { left + right };\n\
         apply :: (((Int32, Int32) -> Int32), Int32) -> Int32 := (operation, value) -> {\n\
           value.operation(1)\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           40i32.add(1).add(1) - apply(add, 41)\n\
         };",
    );
}

#[test]
fn rejects_invalid_sum_continuations_and_type_applications() {
    assert_eq!(
        check_error(
            "Choice :: [Unit, Int32]; bad :: Choice -> Int32 := (choice) -> { choice[() -> { 0 }] };"
        )
        .message,
        "lambda parameters do not match the expected function type"
    );
    assert_eq!(
        check_error(
            "Choice :: [Unit, Int32]; bad :: Choice -> Int32 := (choice) -> { choice[() -> { 0 }, (value) -> { value }, (value) -> { value }] };"
        )
        .message,
        "sum continuation count does not match its type"
    );
}

#[test]
fn propagates_types_through_capture_bindings() {
    let program = check_ok(
        "make :: Int32 -> (Int32 -> Int32) := (x) -> {\n\
           (y) -> { x + y; };\n\
         };",
    );
    let ExpressionKind::Lambda(outer) = &top_binding(&program, 0).value.kind else {
        panic!("expected outer lambda");
    };
    let ExpressionKind::Lambda(inner) = &completion_value(&outer.body.result).kind else {
        panic!("expected inner lambda");
    };
    assert_eq!(inner.captures[0].ty, Type::Int32);
    assert_eq!(inner.parameter_type, Type::Int32);
}

#[test]
fn type_errors_keep_a_renderable_source_span() {
    let source = source("value :: Unit := 0;");
    let parsed = parse(&source).expect("parse");
    let resolved = resolve::resolve(&parsed).expect("resolve");
    let error = check::check(&resolved).expect_err("type mismatch");

    assert!(error.render(&source).contains("check-test.mal:1:18"));
}

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
            "generic value requires type arguments",
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
