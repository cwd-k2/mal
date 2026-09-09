use super::*;

#[test]
fn checks_the_basic_host_example_end_to_end_through_typed_ast() {
    let program = check_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           printInt32(42);\n\
           0;\n\
         };",
    );
    let TopItem::ExternalOperation {
        parameter, result, ..
    } = &program.items[0].kind
    else {
        panic!("expected external operation");
    };
    assert_eq!(*parameter, Type::Int32);
    assert_eq!(*result, Type::Unit);
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        }
    );
}

#[test]
fn gives_external_operations_first_class_function_types() {
    let program = check_ok(
        "extern inspect :: Int32 -> Int32;\n\
         apply :: (Int32 -> Int32, Int32) -> Int32 := \\(operation, value) { operation(value) };\n\
         main :: Unit -> Int32 := \\() { apply(inspect, 42) };",
    );
    let ExpressionKind::Lambda(main) = &top_binding(&program, 2).value.kind else {
        panic!("expected main lambda");
    };
    let ExpressionKind::Call { argument, .. } = &main.body.result.kind else {
        panic!("expected apply call");
    };
    let ExpressionKind::Product(arguments) = &argument.kind else {
        panic!("expected product argument");
    };
    assert_eq!(
        arguments[0].ty,
        Type::Function {
            parameter: Box::new(Type::Int32),
            result: Box::new(Type::Int32),
        }
    );
}

#[test]
fn admits_external_functions_as_closed_top_level_values() {
    let program = check_ok(
        "extern inspect :: Int32 -> Int32;\n\
         selected :: Int32 -> Int32 := inspect;",
    );

    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Function {
            parameter: Box::new(Type::Int32),
            result: Box::new(Type::Int32),
        }
    );
}

#[test]
fn expands_aliases_and_compares_types_structurally() {
    let program = check_ok(
        "Flag :: [Unit, Unit];\n\
         choose :: Flag -> Int32 := \\(flag) {\n\
           if (flag) then { 1 } else { 0 };\n\
         };",
    );
    let TopItem::TypeAlias { ty, .. } = &program.items[0].kind else {
        panic!("expected alias");
    };
    assert_eq!(*ty, Type::Sum(vec![Type::Unit, Type::Unit]));
}

#[test]
fn rejects_recursive_aliases_even_when_unused() {
    let error = check_error("First :: Second; Second :: First;");
    assert_eq!(error.message, "recursive type alias");
}

#[test]
fn checks_nominal_external_opaque_types() {
    let program = check_ok(
        "extern Mem;\n\
         extern File;\n\
         extern allocate :: UInt64 -> Mem;\n\
         extern length :: Mem -> UInt64;\n\
         main :: Unit -> Int32 := \\() {\n\
           mem := allocate(4u64);\n\
           Int32(length(mem));\n\
         };",
    );
    assert!(matches!(
        program.items[0].kind,
        TopItem::ExternalType { .. }
    ));
    let TopItem::ExternalOperation {
        result: Type::External { name, .. },
        ..
    } = &program.items[2].kind
    else {
        panic!("allocate should return an external type");
    };
    assert_eq!(name, "Mem");

    assert_eq!(
        check_error(
            "extern Mem;\n\
             extern File;\n\
             extern getFile :: Unit -> File;\n\
             extern useMem :: Mem -> Unit;\n\
             main :: Unit -> Int32 := \\() {\n\
               useMem(getFile());\n\
               0;\n\
             };"
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn validates_extern_signatures_recursively() {
    assert_eq!(
        check_error("extern callback :: (Int32 -> Unit) -> Unit;").message,
        "external operation `callback` uses a function value"
    );
    assert_eq!(
        check_error("extern wrapped :: [Unit, Int32 -> Int32] -> Unit;").message,
        "external operation `wrapped` uses a function value"
    );
    assert_eq!(
        check_error("extern constant :: Int32;").message,
        "external operation `constant` must have a function type"
    );
}

#[test]
fn rejects_effectful_top_level_initializers() {
    let error = check_error(
        "extern read :: Unit -> Int32;\n\
         value :: Int32 := read();",
    );
    assert_eq!(error.message, "unsupported top-level initializer");
}
