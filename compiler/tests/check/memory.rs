use super::*;

#[test]
fn checks_ptr_extern_signatures_and_memory_primitives() {
    let program = check_ok(
        "extern memory :: Unit -> Ptr;\n\
         useMemory :: Ptr -> UInt8 := \\(pointer :: Ptr) {\n\
           slot := pointer + 8u64;\n\
           storeInt64(slot, 42i64);\n\
           value := loadInt64(slot);\n\
           storeUInt8(slot, UInt8(value));\n\
           loadUInt8(slot);\n\
         };",
    );
    let TopItem::ExternalOperation {
        parameter, result, ..
    } = &program.items[0].kind
    else {
        panic!("expected external operation");
    };
    assert_eq!(*parameter, Type::Unit);
    assert_eq!(*result, Type::Ptr);
    let ExpressionKind::Lambda(function) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.result.ty, Type::UInt8);
    assert!(matches!(
        function.body.result.kind,
        ExpressionKind::Memory { .. }
    ));
}

#[test]
fn checks_memory_primitives_for_every_supported_value_type() {
    let program = check_ok(
        "useMemory :: Ptr -> Unit := \\(pointer :: Ptr) {\n\
           storeInt8(pointer, loadInt8(pointer));\n\
           storeInt16(pointer, loadInt16(pointer));\n\
           storeInt32(pointer, loadInt32(pointer));\n\
           storeInt64(pointer, loadInt64(pointer));\n\
           storeUInt8(pointer, loadUInt8(pointer));\n\
           storeUInt16(pointer, loadUInt16(pointer));\n\
           storeUInt32(pointer, loadUInt32(pointer));\n\
           storeUInt64(pointer, loadUInt64(pointer));\n\
           storeFloat32(pointer, loadFloat32(pointer));\n\
           storeFloat64(pointer, loadFloat64(pointer));\n\
           storePtr(pointer, loadPtr(pointer));\n\
           storeSymbol(pointer, loadSymbol(pointer, 4u64));\n\
           ();\n\
         };",
    );
    let ExpressionKind::Lambda(function) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.items.len(), 12);
    assert!(function
        .body
        .items
        .iter()
        .all(|item| matches!(item, malc::check::ast::BodyItem::Expression(expression) if expression.ty == Type::Unit)));
}

#[test]
fn checks_storage_sizes_for_scalar_and_ptr_types() {
    let program = check_ok(
        "Byte :: UInt8;\n\
         byteSize :: UInt64 := @Byte;\n\
         sizes :: Unit -> UInt64 := \\() {\n\
           @Int8 + @Int16 + @Int32 + @Int64 + byteSize\n\
             + @UInt16 + @UInt32 + @UInt64 + @Float32 + @Float64 + @Ptr;\n\
         };",
    );
    assert_eq!(top_binding(&program, 1).value.ty, Type::UInt64);
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::StorageSize(Type::UInt8)
    ));
    let ExpressionKind::Lambda(function) = &top_binding(&program, 2).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.result.ty, Type::UInt64);
}

#[test]
fn rejects_storage_sizes_without_a_memory_representation() {
    for text in [
        "value := @Unit;",
        "value := @Symbol;",
        "value := @(UInt8, Symbol);",
        "value := @[UInt8, Symbol];",
        "extern Resource; value := @Resource;",
        "value := @(Int32 -> Int32);",
    ] {
        let error = check_error(text);
        assert_eq!(
            error.message, "type has no defined memory storage representation",
            "input: {text}"
        );
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn rejects_mistyped_memory_operations() {
    for text in [
        "extern memory :: Unit -> Ptr; bad := \\() { extern memory() + 1i64; };",
        "bad := \\() { loadInt64(0u64); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storeUInt8(extern memory(), 1u64); (); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storePtr(extern memory(), 1u64); (); };",
        "bad := \\() { loadSymbol(0u64, 1u64); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storeSymbol(extern memory(), 1u64); (); };",
        "extern memory :: Unit -> Ptr; bad := \\() { 1u64 + extern memory(); };",
        "extern memory :: Unit -> Ptr; bad := \\() { extern memory() + extern memory(); };",
        "extern memory :: Unit -> Ptr; bad := \\() { 1u64 - extern memory(); };",
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn gives_every_memory_function_a_first_class_function_type() {
    let program = check_ok(
        "li8 :: Ptr -> Int8 := loadInt8; si8 :: (Ptr, Int8) -> Unit := storeInt8;\n\
         li16 :: Ptr -> Int16 := loadInt16; si16 :: (Ptr, Int16) -> Unit := storeInt16;\n\
         li32 :: Ptr -> Int32 := loadInt32; si32 :: (Ptr, Int32) -> Unit := storeInt32;\n\
         li64 :: Ptr -> Int64 := loadInt64; si64 :: (Ptr, Int64) -> Unit := storeInt64;\n\
         lu8 :: Ptr -> UInt8 := loadUInt8; su8 :: (Ptr, UInt8) -> Unit := storeUInt8;\n\
         lu16 :: Ptr -> UInt16 := loadUInt16; su16 :: (Ptr, UInt16) -> Unit := storeUInt16;\n\
         lu32 :: Ptr -> UInt32 := loadUInt32; su32 :: (Ptr, UInt32) -> Unit := storeUInt32;\n\
         lu64 :: Ptr -> UInt64 := loadUInt64; su64 :: (Ptr, UInt64) -> Unit := storeUInt64;\n\
         lf32 :: Ptr -> Float32 := loadFloat32; sf32 :: (Ptr, Float32) -> Unit := storeFloat32;\n\
         lf64 :: Ptr -> Float64 := loadFloat64; sf64 :: (Ptr, Float64) -> Unit := storeFloat64;\n\
         lp :: Ptr -> Ptr := loadPtr; sp :: (Ptr, Ptr) -> Unit := storePtr;\n\
         le :: (Ptr, UInt64) -> Symbol := loadSymbol; se :: (Ptr, Symbol) -> Unit := storeSymbol;",
    );
    assert_eq!(program.items.len(), 24);
    assert!(program.items.iter().all(|item| {
        matches!(
            &item.kind,
            TopItem::Binding(binding)
                if matches!(binding.value.kind, ExpressionKind::MemoryFunction { .. })
                    && matches!(binding.value.ty, Type::Function { .. })
        )
    }));
}
