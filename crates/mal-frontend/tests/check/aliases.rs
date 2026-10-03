use super::*;

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
fn checks_constructor_application_and_recursion_at_the_owning_stage() {
    for (source, message) in [
        (
            "Pair<A, B> :: (A, B); value :: Pair<Int32> := 0;",
            "type constructor used as a value type",
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
            "value :: Buffer := 0;",
            "type constructor used as a value type",
        ),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }

    check_ok("Discard<A> :: USize; Pair<A, B> :: (A, B); value :: Discard<Pair<Int32>> := 0usize;");
    check_ok("Discard<A> :: USize; opaque Box<A> :: A; value :: Discard<Box> := 0usize;");
}

#[test]
fn infers_kinds_and_normalizes_partial_constructor_application() {
    let program = check_ok(
        "Apply<F, A> :: F<A>;\n\
         Pair<A, B> :: (A, B);\n\
         PairWithInt32 :: Pair<Int32>;\n\
         value :: Apply<PairWithInt32, UInt8> := (42i32, 7u8);",
    );

    assert_eq!(
        top_binding(&program, 0).value.ty,
        Type::Product(vec![Type::Int32, Type::UInt8].into())
    );
}

#[test]
fn rejects_invalid_constructor_kinds_at_type_checking() {
    for (source, message) in [
        (
            "Pair<A, B> :: (A, B); value :: Pair<Buffer, Int32> := 0;",
            "type kind mismatch",
        ),
        (
            "Pair<A, B> :: (A, B); value :: Pair<Int32, UInt8, Unit> := 0;",
            "type does not accept arguments",
        ),
        ("Omega<F> :: F<F>;", "infinite kind"),
    ] {
        assert_eq!(check_error(source).message, message, "source: {source}");
    }
}

#[test]
fn generalizes_identity_constructor_kinds_per_use() {
    check_ok(
        "Id<X> :: X;\n\
         Pair<A, B> :: (A, B);\n\
         IntIdentity :: Id<Int32>;\n\
         PairWithInt32 :: Id<Pair<Int32>>;\n\
         left :: IntIdentity := 1;\n\
         right :: PairWithInt32<UInt8> := (1i32, 2u8);",
    );
}

#[test]
fn allows_source_aliases_named_index() {
    check_ok("Index<A> :: USize; value :: Index<Int32> := 0usize;");
}

#[test]
fn forms_buffer_types_only_for_storable_elements() {
    check_ok("Inner :: Buffer<UInt8>; Outer :: Buffer<(Int32, Inner)>;");

    for source in [
        "Callback :: Int32 -> Int32; value :: Buffer<Callback> := 0;",
        "value :: Buffer<[]> := 0;",
        "extern Handle; value :: Buffer<Handle> := 0;",
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
    let mut source = String::from("Pair<A> :: (A, A);\nValue0 :: UInt8;\n");
    for level in 1..=22 {
        source.push_str(&format!("Value{level} :: Pair<Value{}>;\n", level - 1));
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
