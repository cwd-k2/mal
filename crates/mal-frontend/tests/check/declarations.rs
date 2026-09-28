use super::*;

#[test]
fn opaque_types_use_their_representation_only_in_the_declaring_file() {
    let program = check_ok(
        "opaque Pair<A> :: (A, A);\n\
         makePair<A> :: (A, A) -> Pair<A> := (pair) -> pair;\n\
         first<A> :: Pair<A> -> A := ((first, _)) -> first;\n\
         main :: Unit -> Int32 := () -> first(makePair((40i32, 2i32)));",
    );

    let TopItem::OpaqueType { .. } = &program.items[0].kind else {
        panic!("expected opaque declaration");
    };
    let specialized = check::specialize(program).expect("erase opaque boundaries");
    assert!(
        specialized
            .program()
            .items
            .iter()
            .all(|item| { !matches!(item.kind, TopItem::OpaqueType { .. }) })
    );
}

#[test]
fn opaque_sum_uses_existing_construction_and_elimination_syntax() {
    let program = check_ok(
        "opaque Option<A> :: [Unit, A];\n\
         none<A> :: Unit -> Option<A> := () -> [none, some] => none();\n\
         isNone<A> :: Option<A> -> Bool := (value) -> value[\n\
             () -> true,\n\
             (_) -> false\n\
         ];\n\
         main :: Unit -> Int32 := () -> {\n\
             value :: Option<Int32> := none<Int32>();\n\
             if (isNone(value)) then 0 else 1;\n\
         };",
    );

    check::specialize(program).expect("erase opaque sum boundary");
}

#[test]
fn opaque_declarations_with_the_same_representation_remain_distinct() {
    let error = check_error(
        "opaque Left :: Int32;\n\
         opaque Right :: Int32;\n\
         wrong :: Left -> Right := (value) -> value;",
    );

    assert_eq!(error.message, "type mismatch");
}

#[test]
fn rejects_recursive_opaque_representations_even_when_unused() {
    assert_eq!(
        check_error("opaque Loop<A> :: (A, Loop<A>);").message,
        "recursive opaque representation"
    );
}

#[test]
fn opaque_buffer_uses_memory_operations_only_in_its_declaring_file() {
    let program = check_ok(
        "opaque Values<A> :: Buffer<A>;\n\
         values<A> :: USize -> Values<A> := (capacity) -> make<A>(capacity);\n\
         append<A> :: (Values<A>, A) -> USize := (items, value) -> items.new(value);\n\
         length<A> :: Values<A> -> USize := (items) -> #items;\n\
         main :: Unit -> Int32 := () -> {\n\
             items :: Values<Int32> := values<Int32>(1usize);\n\
             items.append(42i32);\n\
             if (items.length() == 1usize && items.get(0usize) == 42i32) then 0 else 1;\n\
         };",
    );

    check::specialize(program).expect("erase opaque Buffer boundary");
}

#[test]
fn source_opaque_types_do_not_cross_the_extern_boundary() {
    assert_eq!(
        check_error("opaque Counter :: Int32; extern inspect :: Counter -> Unit;").message,
        "external operation `inspect` uses a type that is not host mappable"
    );
}

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
fn expands_generic_aliases_with_canonical_concrete_arguments() {
    let program = check_ok(
        "Pair<A> :: (A, A);\n\
         Nested<A> :: Pair<Pair<A>>;\n\
         value :: Nested<Int32> := ((1, 2), (3, 4));",
    );

    assert_eq!(
        program.items.len(),
        1,
        "generic aliases are compile-time declarations"
    );
    assert_eq!(
        top_binding(&program, 0).value.ty,
        Type::Product(
            vec![
                Type::Product(vec![Type::Int32, Type::Int32].into()),
                Type::Product(vec![Type::Int32, Type::Int32].into()),
            ]
            .into()
        )
    );
}

#[test]
fn erases_phantom_alias_arguments_without_forming_recursive_value_types() {
    let program = check_ok(
        "Coordinate<A> :: USize;\n\
         Node :: (Int32, Coordinate<Node>);\n\
         Tree :: Buffer<Node>;\n\
         root :: Node := (1i32, 0usize);\n\
         next :: Coordinate<Node> -> Coordinate<Node> := (index) -> index + 1usize;",
    );

    let TopItem::TypeAlias { ty, .. } = &program.items[0].kind else {
        panic!("expected Node alias");
    };
    assert_eq!(*ty, Type::Product(vec![Type::Int32, Type::USize].into()));
    assert_eq!(top_binding(&program, 2).value.ty, *ty);
}

#[test]
fn keeps_phantom_detection_local_to_each_alias_declaration() {
    check_ok(
        "Discard<A> :: USize; Node :: (Int32, Discard<Node>); value :: Node := (1i32, 0usize);",
    );

    let error = check_error(
        "Discard<A> :: USize;\n\
         Forward<A> :: Discard<A>;\n\
         Node :: (Int32, Forward<Node>);",
    );
    assert_eq!(error.message, "recursive type alias");
}

#[test]
fn checks_generic_alias_arity_and_recursion_at_the_owning_stage() {
    for (source, message) in [
        (
            "Pair<A, B> :: (A, B); value :: Pair<Int32> := 0;",
            "generic type argument arity mismatch",
        ),
        (
            "Value :: Int32; value :: Value<Int32> := 0;",
            "type does not accept arguments",
        ),
        (
            "Loop<A> :: Loop<A>; value := 0;",
            "recursive generic type alias",
        ),
        (
            "Discard<A> :: USize; Pair<A, B> :: (A, B); value :: Discard<Pair<Int32>> := 0usize;",
            "generic type argument arity mismatch",
        ),
        ("value :: Buffer := 0;", "generic type requires arguments"),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn allows_source_aliases_named_index() {
    check_ok("Index<A> :: USize; value :: Index<Int32> := 0usize;");
}

#[test]
fn forms_buffer_types_only_for_storable_elements() {
    for source in [
        "Callback :: Int32 -> Int32; value :: Buffer<Callback> := 0;",
        "value :: Buffer<[]> := 0;",
        "value :: Buffer<Buffer<UInt8>> := 0;",
        "extern Handle; value :: Buffer<Handle> := 0;",
        "value :: Buffer<(Int32, Buffer<UInt8>)> := 0;",
        "value :: Buffer<[Unit, Unit -> Unit]> := 0;",
    ] {
        assert_eq!(
            check_error(source).message,
            "buffer element type is not storable",
            "source: {source}"
        );
    }
}

#[test]
fn rejects_non_host_mappable_types_at_the_host_boundary() {
    for source in [
        "extern inspect :: Symbol -> Unit;",
        "extern inspect :: Buffer<UInt8> -> Unit;",
        "extern inspect :: Unit -> Buffer<UInt8>;",
        "extern inspect :: (Int32, Buffer<UInt8>) -> Unit;",
        "Payload :: [UInt8, Symbol]; extern inspect :: Payload -> Unit;",
    ] {
        assert_eq!(
            check_error(source).message,
            "external operation `inspect` uses a type that is not host mappable",
            "source: {source}"
        );
    }
}

#[test]
fn admits_address_and_length_descriptors_at_the_host_boundary() {
    check_ok(
        "Bytes :: (Address, USize);\n\
         Transfer :: [Bytes, UInt32];\n\
         extern exchange :: Bytes -> Transfer;",
    );
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
fn reports_the_first_invalid_alias_in_source_order() {
    let text = "Pair :: (Int32, Pair); Loop :: [Loop, Int32]; Other :: (Other, Int32);";
    for _ in 0..8 {
        let error = check_error(text);
        assert_eq!(error.message, "recursive type alias");
        assert_eq!(
            error.primary.expect("alias cycle location").span.start(),
            text.find("Pair)").expect("first cycle reference"),
        );
    }
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
    // The former argument form is no longer an entry type.
    let former = check_error("main :: Buffer<(Address, USize)> -> Int32 := (_) -> { 0i32; };");
    assert_eq!(former.message, "invalid entry point type");

    let product = check_error("(main, other) :: (Unit -> Int32, Int32) := (() -> 0i32, 1i32);");
    assert_eq!(product.message, "invalid entry point binding");
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
