use super::*;

#[test]
fn checks_typed_cursor_region_and_stride_operations() {
    let program = check_ok(
        "extern memory :: Unit -> Address;\n\
         useMemory :: Unit -> UInt64 := () -> {\n\
           cursor := memory()@u64;\n\
           next := cursor <- 41u64;\n\
           value := <-cursor;\n\
           region := next@3usize;\n\
           indexed :: Cursor<UInt64> := region # 1usize;\n\
           projected :: Address := ?region;\n\
           shifted := projected + #u64;\n\
           _ := shifted@u8;\n\
           value + (<-indexed) + (#(u8, u64)).u64;\n\
         };",
    );
    let ExpressionKind::Lambda(function) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(completion_value(&function.body.result).ty, Type::UInt64);
}

#[test]
fn permits_discarding_a_store_result_as_an_expression_statement() {
    check_ok(
        "write :: (Cursor<UInt8>, UInt8) -> Unit := (cursor, value) -> {\n\
           cursor <- value;\n\
           ();\n\
         };",
    );
}

#[test]
fn rejects_mismatched_typed_memory_operations() {
    for text in [
        "bad :: Address -> Unit := (address) -> { address@u64 <- 1u8; (); };",
        "bad :: Address -> Address := (address) -> ?address;",
        "bad :: Address -> UInt64 := (address) -> <-(address@u64@1usize);",
        "bad := 1u64@u8;",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_region_packed_transfer_views_and_symbol_conversion() {
    check_ok(
        "admit :: Region<UInt8> -> Packed<UInt8> := (region) -> <-region;\n\
         store :: (Region<UInt8>, Packed<UInt8>) -> Region<UInt8> :=\n\
           (region, packed) -> region <- packed;\n\
         inspect :: (Packed<UInt8>, USize) -> (UInt8, USize, Symbol) :=\n\
           (packed, count) -> {\n\
             prefix := packed / count;\n\
             _ := packed % count;\n\
             (packed # 0usize, #prefix, *prefix);\n\
           };\n\
         bytes :: Symbol -> Packed<UInt8> := (symbol) -> *symbol;",
    );
}

#[test]
fn checks_view_slices_in_an_expected_view_context() {
    check_ok(
        "split :: (Region<UInt8>, Packed<UInt8>, USize) -> (Region<UInt8>, Packed<UInt8>) :=\n\
           (region, packed, count) -> {\n\
             prefix :: Region<UInt8> := region / count;\n\
             remainder :: Packed<UInt8> := packed % count;\n\
             (prefix, remainder);\n\
           };",
    );
}

#[test]
fn rejects_packed_operations_for_wrong_element_or_operand_types() {
    for text in [
        "bad :: Packed<UInt16> -> Symbol := (packed) -> *packed;",
        "bad :: Packed<UInt8> -> UInt8 := (packed) -> packed # 0bytes;",
        "bad :: Region<UInt8> -> Cursor<UInt8> := (region) -> region # 0bytes;",
        "bad :: Region<UInt8> -> Region<UInt8> := (region) -> region / 1bytes;",
        "bad :: (Region<UInt8>, Packed<UInt16>) -> Region<UInt8> := (region, packed) -> region <- packed;",
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
