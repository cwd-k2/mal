use super::*;

#[test]
fn rejects_binding_and_block_result_type_mismatches() {
    assert_eq!(check_error("value :: Unit := 0;").message, "type mismatch");
    assert_eq!(
        check_error("main :: Unit -> Unit := \\() { 0; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_function_application_and_zero_argument_unit_lowering() {
    let program = check_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         thunk :: Unit -> Int32 := \\() { identity(4); };\n\
         caller :: Unit -> Int32 := \\() { thunk(); };",
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        }
    );

    assert_eq!(
        check_error(
            "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
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
fn checks_return_as_an_ordinary_local_name() {
    check_ok("value :: Unit -> Int64 := \\() { return := 1; return; };");
}

#[test]
fn checks_products_destructuring_and_multiple_parameters() {
    let program = check_ok(
        "Pair :: (Int32, UInt8);\n\
         pair :: Pair := (1, 2);\n\
         add :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           (first, _) := pair;\n\
           nested := ((first, 2i32), 39i32);\n\
           ((left, right), extra) := nested;\n\
           add(left + right, extra);\n\
         };",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Product(vec![Type::Int32, Type::UInt8])
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Product(vec![Type::Int32, Type::Int32])),
            result: Box::new(Type::Int32),
        }
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
        "count :: Int64 -> Int64 := \\(n :: Int64) {\n\
           if (n == 0) then { 0 } else { count(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           local :: Int32 -> Int32 := \\(n :: Int32) {\n\
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
fn rejects_recursive_lambda_that_disagrees_with_its_annotation() {
    let error = check_error("looping :: Int32 -> Int32 := \\(n :: Int64) { looping(n); };");
    assert_eq!(error.message, "type mismatch");
}

#[test]
fn propagates_types_through_capture_bindings() {
    let program = check_ok(
        "make :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           \\<x>(y :: Int32) { x + y; };\n\
         };",
    );
    let ExpressionKind::Lambda(outer) = &top_binding(&program, 0).value.kind else {
        panic!("expected outer lambda");
    };
    let ExpressionKind::Lambda(inner) = &outer.body.result.kind else {
        panic!("expected inner lambda");
    };
    assert_eq!(inner.captures[0].ty, Type::Int32);
    assert_eq!(inner.parameters[0].ty, Type::Int32);
}

#[test]
fn type_errors_keep_a_renderable_source_span() {
    let source = source("value :: Unit := 0;");
    let parsed = parse(&source).expect("parse");
    let resolved = resolve::resolve(&parsed).expect("resolve");
    let error = check::check(&resolved).expect_err("type mismatch");

    assert!(error.render(&source).contains("check-test.mal:1:18"));
}
