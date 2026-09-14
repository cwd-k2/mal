use super::*;

#[test]
fn checks_the_basic_host_example_end_to_end_through_typed_ast() {
    let program = check_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := () -> {\n\
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
            parameter: Box::new(Type::Unit).into(),
            result: Box::new(Type::Int32).into(),
        }
    );
}

#[test]
fn gives_external_operations_first_class_function_types() {
    let program = check_ok(
        "extern inspect :: Int32 -> Int32;\n\
         apply :: (Int32 -> Int32, Int32) -> Int32 := (operation, value) -> { operation(value) };\n\
         main :: Unit -> Int32 := () -> { apply(inspect, 42) };",
    );
    let ExpressionKind::Lambda(main) = &top_binding(&program, 2).value.kind else {
        panic!("expected main lambda");
    };
    let ExpressionKind::Call { argument, .. } = &completion_value(&main.body.result).kind else {
        panic!("expected apply call");
    };
    let ExpressionKind::Product(arguments) = &argument.kind else {
        panic!("expected product argument");
    };
    assert_eq!(
        arguments[0].ty,
        Type::Function {
            parameter: Box::new(Type::Int32).into(),
            result: Box::new(Type::Int32).into(),
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
            parameter: Box::new(Type::Int32).into(),
            result: Box::new(Type::Int32).into(),
        }
    );
}

#[test]
fn expands_aliases_and_compares_types_structurally() {
    let program = check_ok(
        "Flag :: [Unit, Unit];\n\
         choose :: Flag -> Int32 := (flag) -> {\n\
           if (flag) then { 1 } else { 0 };\n\
         };",
    );
    let TopItem::TypeAlias { ty, .. } = &program.items[0].kind else {
        panic!("expected alias");
    };
    assert_eq!(*ty, Type::Sum(vec![Type::Unit, Type::Unit].into()));
}

#[test]
fn shares_repeated_alias_structure_without_exponential_expansion() {
    let mut source = String::from("Left0 :: Unit;\nRight0 :: Unit;\n");
    for level in 1..=64 {
        source.push_str(&format!(
            "Left{level} :: [Left{}, Left{}];\nRight{level} :: [Right{}, Right{}];\n",
            level - 1,
            level - 1,
            level - 1,
            level - 1
        ));
    }
    source.push_str(
        "extern inspect :: Left64 -> Unit;\nidentity :: Left64 -> Right64 := (value) -> { value; };",
    );

    check_ok(&source);
}

#[test]
fn rejects_types_whose_physical_product_representation_is_too_large() {
    let mut source = String::from("Value0 :: UInt8;\n");
    for level in 1..=17 {
        source.push_str(&format!(
            "Value{level} :: (Value{}, Value{});\n",
            level - 1,
            level - 1
        ));
    }

    let error = check_error(&source);

    assert_eq!(error.message, "type representation is too large");
    assert!(
        error
            .primary
            .expect("representation diagnostic")
            .message
            .contains("64 nested levels and 65536 storage components")
    );
}

#[test]
fn rejects_deep_representations_built_by_flat_alias_declarations() {
    let mut source = String::from("Value0 :: UInt8;\n");
    for level in 1..=65 {
        source.push_str(&format!("Value{level} :: (Value{}, UInt8);\n", level - 1));
    }

    let error = check_error(&source);

    assert_eq!(error.message, "type representation is too large");
    assert!(
        error
            .primary
            .expect("representation diagnostic")
            .message
            .contains("64 nested levels")
    );
}

#[test]
fn expands_long_alias_dependency_chains_without_host_recursion() {
    let mut source = String::new();
    for index in 0..4096 {
        source.push_str(&format!("Alias{index} :: Alias{};\n", index + 1));
    }
    source.push_str(
        "Alias4096 :: (UInt8, UInt8);\n\
         Operation :: Alias0 -> Alias0;\n\
         extern exchange :: Operation;\n\
         value :: Alias0 := (1u8, 2u8);",
    );

    check_ok(&source);
}

#[test]
fn bounds_names_of_shared_types_in_diagnostics() {
    let mut ty = Type::UInt8;
    for _ in 0..64 {
        ty = Type::Product(vec![ty.clone(), ty].into());
    }

    let name = check::type_name(&ty);

    assert!(name.len() <= 4099);
    assert!(name.ends_with('…'));
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
         main :: Unit -> Int32 := () -> {\n\
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
             main :: Unit -> Int32 := () -> {\n\
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
    assert_eq!(error.message, "invalid top-level initializer");
}
