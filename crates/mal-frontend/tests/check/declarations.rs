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
fn admits_closed_byte_size_arithmetic_at_top_level() {
    let program = check_ok("recordSize :: ByteSize := 1bytes + 8bytes + 4bytes + 8bytes;");

    let value = &top_binding(&program, 0).value;
    assert_eq!(value.ty, Type::ByteSize);
    assert!(matches!(value.kind, ExpressionKind::Binary { .. }));
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
           length(mem).i32;\n\
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
        "external operation `callback` uses a type that is not host mappable"
    );
    assert_eq!(
        check_error("extern wrapped :: [Unit, Int32 -> Int32] -> Unit;").message,
        "external operation `wrapped` uses a type that is not host mappable"
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

#[test]
fn rejects_top_level_arithmetic_over_another_top_level_value() {
    let error = check_error(
        "base :: ByteSize := 1bytes;\n\
         recordSize :: ByteSize := base + 8bytes;",
    );
    assert_eq!(error.message, "invalid top-level initializer");
}

#[test]
fn validates_an_explicit_entry_point_signature() {
    let error = check_error("main :: Unit -> UInt64 := () -> 0u64;");
    assert_eq!(error.message, "invalid entry point type");
    assert!(error.primary.unwrap().message.contains("Buffer<Symbol>"));

    let unit = check_ok("main :: Unit -> Int32 := () -> 0i32;");
    assert_eq!(
        unit.entry.expect("checked entry point").parameter,
        mal_frontend::check::ast::EntryParameter::Unit
    );
    let arguments = check_ok("main :: Buffer<Symbol> -> Int32 := (_) -> { 0i32; };");
    assert_eq!(
        arguments.entry.expect("checked entry point").parameter,
        mal_frontend::check::ast::EntryParameter::ProcessArguments
    );
    let address_pairs =
        check_error("main :: Buffer<(Address, USize)> -> Int32 := (_) -> { 0i32; };");
    assert_eq!(address_pairs.message, "invalid entry point type");

    let product = check_error("(main, other) :: (Unit -> Int32, Int32) := (() -> 0i32, 1i32);");
    assert_eq!(product.message, "invalid entry point binding");

    for text in [
        "main<A> :: A -> Int32 := (_) -> 0i32;",
        "main<A> :: A -> Int32;",
        "extern main :: Unit -> Int32;",
    ] {
        assert_eq!(
            check_error(text).message,
            "invalid entry point declaration",
            "input: {text}"
        );
    }
}

#[test]
fn forms_buffers_of_symbols_and_aggregates_of_symbols() {
    check_ok("build :: Unit -> Buffer<Symbol> := () -> make<Symbol>(0usize);");
    check_ok(
        "Entry :: (Symbol, Int32); build :: Unit -> Buffer<Entry> := () -> make<Entry>(0usize);",
    );
    check_ok(
        "Maybe :: [Unit, Symbol]; build :: Unit -> Buffer<Maybe> := () -> make<Maybe>(0usize);",
    );
}

#[test]
fn keeps_c_host_copy_limited_to_representable_elements() {
    let error = check_error(
        "read :: Address -> Buffer<Symbol> := (address) -> from<Symbol>(address, 0usize, 1usize);",
    );
    assert_eq!(
        error.message,
        "memory intrinsic requires a Representable element type"
    );

    let error = check_error(
        "read<A> :: Address -> Buffer<A> := (address) -> from<A>(address, 0usize, 1usize);",
    );
    assert_eq!(
        error.message,
        "memory intrinsic requires a Representable element type"
    );
}
