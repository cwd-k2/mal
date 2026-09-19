use super::*;

#[test]
fn checks_typed_region_and_stride_operations() {
    let program = check_ok(
        "extern memory :: Unit -> Address;\n\
         useMemory :: Unit -> UInt64 := () ->\n\
           view<UInt64>(memory(), 0usize, 3usize, (region) -> {\n\
             region.put(0usize, 41u64);\n\
             prefix := region / 2usize;\n\
             remainder := region % 1usize;\n\
             shifted := memory() + #u64;\n\
             _ := shifted;\n\
             prefix.get(0usize) + remainder.get(0usize) + (#(u8, u64)).u64;\n\
           });",
    );
    let ExpressionKind::Lambda(function) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(completion_value(&function.body.result).ty, Type::UInt64);
}

#[test]
fn permits_discarding_a_put_result_as_an_expression_statement() {
    check_ok(
        "write :: (Region<UInt8>, UInt8) -> Unit := (region, value) -> {\n\
           region.put(0usize, value);\n\
           ();\n\
         };",
    );
}

#[test]
fn rejects_mismatched_typed_memory_operations() {
    for text in [
        "bad :: Region<UInt64> -> Unit := (region) -> region.put(0usize, 1u8);",
        "bad :: Region<UInt64> -> UInt64 := (region) -> region.get(0bytes);",
        "bad :: Region<UInt64> -> Region<UInt64> := (region) -> region;",
        "bad :: Address -> Packed<UInt64> := (address) -> pack<UInt64>(address, 0bytes, 1usize);",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_region_packed_transfer_views_and_symbol_conversion() {
    check_ok(
        "admit :: (Address, USize) -> Packed<UInt8> := (address, count) ->\n\
           pack<UInt8>(address, 0usize, count);\n\
         store :: (Region<UInt8>, Packed<UInt8>) -> Unit :=\n\
           (region, packed) -> { _ := region.set(packed); (); };\n\
         inspect :: (Packed<UInt8>, USize) -> (UInt8, USize, Symbol) :=\n\
           (packed, count) -> {\n\
             prefix := packed / count;\n\
             remainder := packed % count;\n\
             joined := prefix + remainder;\n\
             (joined # 0usize, #prefix, *prefix);\n\
           };\n\
         bytes :: Symbol -> Packed<UInt8> := (symbol) -> *symbol;",
    );
}

#[test]
fn keeps_region_and_buffer_authority_inside_their_invocation() {
    for text in [
        "bad :: Region<UInt8> -> Region<UInt8> := (region) -> region;",
        "bad :: Region<UInt8> -> (Region<UInt8>, USize) := (region) -> (region, #region);",
        "bad :: Region<UInt8> -> UInt8 := (region) -> { read :: Unit -> UInt8 := () -> region.get(0usize); read(); };",
        "bad := make<UInt8>(0usize, (buffer) -> { read :: Unit -> UInt8 := () -> buffer.get(0usize); _ := read(); (); });",
        "identity<A> :: A -> A := (value) -> value; bad :: Region<UInt8> -> Unit := (region) -> { _ := identity<Region<UInt8>>(region); (); };",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_scoped_packed_construction_and_editing() {
    check_ok(
        "create :: Unit -> Packed<Int32> := () ->
           make<Int32>(0usize, (buffer) -> {
             index := buffer.new(10i32);
             buffer.put(index, buffer.get(index) + 1i32);
             ();
           });
         change :: Packed<Int32> -> Packed<Int32> := (source) ->
           source.edit<Int32>((buffer) -> {
             buffer.put(0usize, buffer.get(0usize) + 1i32);
             _ := buffer.new(20i32);
             ();
           });
         makeBulk :: Unit -> Packed<Int32> := () ->
           make<Int32>(16usize, (buffer) -> {
             _ := buffer.new(30i32);
             ();
           });",
    );
}

#[test]
fn passes_one_buffer_through_helpers_and_supports_ufcs_operations() {
    check_ok(
        "update :: (Buffer<Int32>, USize) -> Unit := (buffer, index) -> {
           put(buffer, index, get(buffer, index) + 1i32);
           ();
         };
         get :: Int32 -> Int32 := (value) -> value;
         create :: Unit -> Packed<Int32> := () -> make<Int32>(0usize, (buffer) -> {
           index := buffer.new(get(10i32));
           update(buffer, index);
           ();
         });",
    );
}

#[test]
fn rejects_invalid_packed_intrinsic_applications() {
    for text in [
        "bad := make<Symbol>(0usize, (_) -> ());",
        "bad := make<Int32>(0usize);",
        "bad := make<Int32>((_) -> ());",
        "bad := make<Int32>(1i32, (_) -> ());",
        "bad := make<Symbol>(1usize, (_) -> ());",
        "bad :: Packed<Int32> -> Packed<Int32> := (source) -> edit<Int32>(source);",
        "bad := make<Int32>(0usize, (buffer) -> { _ := buffer.new(1u32); (); });",
        "bad := make<Int32>(0usize, (buffer) -> buffer);",
        "bad := make<Int32>(0usize, (buffer) -> { buffer.get(); (); });",
        "bad := make<Int32>(0usize, (buffer) -> { buffer.put(0usize, 1u32); (); });",
        "bad :: Buffer<Symbol> -> Unit := (_) -> ();",
        "extern bad :: Buffer<Int32> -> Unit;",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_view_slices_in_an_expected_view_context() {
    check_ok(
        "split :: (Region<UInt8>, Packed<UInt8>, USize) -> (UInt8, Packed<UInt8>) :=\n\
           (region, packed, count) -> {\n\
             prefix :: Region<UInt8> := region / count;\n\
             remainder :: Packed<UInt8> := packed % count;\n\
             (prefix.get(0usize), remainder);\n\
           };",
    );
}

#[test]
fn rejects_packed_operations_for_wrong_element_or_operand_types() {
    for text in [
        "bad :: Packed<UInt16> -> Symbol := (packed) -> *packed;",
        "bad :: Packed<UInt8> -> UInt8 := (packed) -> packed # 0bytes;",
        "bad :: Region<UInt8> -> UInt8 := (region) -> region.get(0bytes);",
        "bad :: Region<UInt8> -> Region<UInt8> := (region) -> region / 1bytes;",
        "bad :: (Region<UInt8>, Packed<UInt16>) -> Unit := (region, packed) -> { _ := region.set(packed); (); };",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_closed_layout_shapes_for_every_representable_scalar() {
    let program = check_ok(
        "sizes :: Unit -> ByteSize := () -> {\n\
           #unit + #i8 + #i16 + #i32 + #i64 + #u8 + #u16 + #u32 + #u64\n\
             + #f32 + #f64 + #address + #bytesize + #usize + #bool;\n\
         };",
    );
    let ExpressionKind::Lambda(function) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(completion_value(&function.body.result).ty, Type::ByteSize);
}
