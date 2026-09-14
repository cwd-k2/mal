use super::*;

#[test]
fn checks_ptr_extern_signatures_and_memory_primitives() {
    let program = check_ok(
        "extern memory :: Unit -> Ptr;\n\
         useMemory :: Ptr -> UInt8 := (pointer) -> {\n\
           slot := pointer + 8u64;\n\
           Int64.store(slot, 42i64);\n\
           value := Int64.load(slot);\n\
           UInt8.store(slot, UInt8(value));\n\
           UInt8.load(slot);\n\
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
    assert_eq!(completion_value(&function.body.result).ty, Type::UInt8);
    assert!(matches!(
        completion_value(&function.body.result).kind,
        ExpressionKind::Memory { .. }
    ));
}

#[test]
fn checks_memory_primitives_for_every_supported_value_type() {
    let program = check_ok(
        "useMemory :: Ptr -> Unit := (pointer) -> {\n\
           Int8.store(pointer, Int8.load(pointer));\n\
           Int16.store(pointer, Int16.load(pointer));\n\
           Int32.store(pointer, Int32.load(pointer));\n\
           Int64.store(pointer, Int64.load(pointer));\n\
           UInt8.store(pointer, UInt8.load(pointer));\n\
           UInt16.store(pointer, UInt16.load(pointer));\n\
           UInt32.store(pointer, UInt32.load(pointer));\n\
           UInt64.store(pointer, UInt64.load(pointer));\n\
           Float32.store(pointer, Float32.load(pointer));\n\
           Float64.store(pointer, Float64.load(pointer));\n\
           Ptr.store(pointer, Ptr.load(pointer));\n\
           Symbol.write(pointer, Symbol.read(pointer, 4u64));\n\
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
         byteSize :: UInt64 := Byte.size;\n\
         sizes :: Unit -> UInt64 := () -> {\n\
           Int8.size + Int16.size + Int32.size + Int64.size + byteSize\n\
             + UInt16.size + UInt32.size + UInt64.size + Float32.size + Float64.size + Ptr.size;\n\
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
    assert_eq!(completion_value(&function.body.result).ty, Type::UInt64);
}

#[test]
fn rejects_storage_sizes_without_a_memory_representation() {
    for text in [
        "value := Unit.size;",
        "value := Symbol.size;",
        "Pair :: (UInt8, Symbol); value := Pair.size;",
        "Choice :: [UInt8, Symbol]; value := Choice.size;",
        "extern Resource; value := Resource.size;",
        "Callback :: Int32 -> Int32; value := Callback.size;",
    ] {
        let error = check_error(text);
        assert!(
            error
                .message
                .contains("has no predefined memory primitive `size`"),
            "input: {text}"
        );
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn expands_transparent_aliases_for_every_memory_primitive() {
    let program = check_ok(
        "Byte :: UInt8;\n\
         size :: UInt64 := Byte.size;\n\
         reader :: Ptr -> Byte := Byte.load;\n\
         writer :: (Ptr, Byte) -> Unit := Byte.store;",
    );
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::StorageSize(Type::UInt8)
    ));
    assert!(program.items[2..].iter().all(|item| matches!(
        &item.kind,
        TopItem::Binding(binding)
            if matches!(binding.value.kind, ExpressionKind::MemoryFunction { .. })
    )));
}

#[test]
fn rejects_unknown_or_mismatched_memory_primitive_members() {
    for text in [
        "value := UInt8.read;",
        "value := Symbol.load;",
        "value := Ptr.read;",
        "value := UInt8.unknown;",
    ] {
        let error = check_error(text);
        assert!(
            error.message.contains("has no predefined memory primitive"),
            "input: {text}"
        );
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn rejects_calling_size_as_a_function() {
    let error = check_error("value :: UInt64 := UInt8.size();");
    assert_eq!(error.message, "cannot call a non-function value");
    assert!(error.primary.is_some());
}

#[test]
fn rejects_mistyped_memory_operations() {
    for text in [
        "extern memory :: Unit -> Ptr; bad := memory() + 1i64;",
        "bad := Int64.load(0u64);",
        "extern memory :: Unit -> Ptr; bad := UInt8.store(memory(), 1u64);",
        "extern memory :: Unit -> Ptr; bad := Ptr.store(memory(), 1u64);",
        "bad := Symbol.read(0u64, 1u64);",
        "extern memory :: Unit -> Ptr; bad := Symbol.write(memory(), 1u64);",
        "extern memory :: Unit -> Ptr; bad := 1u64 + memory();",
        "extern memory :: Unit -> Ptr; bad := memory() + memory();",
        "extern memory :: Unit -> Ptr; bad := 1u64 - memory();",
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn gives_every_memory_function_a_first_class_function_type() {
    let program = check_ok(
        "li8 :: Ptr -> Int8 := Int8.load; si8 :: (Ptr, Int8) -> Unit := Int8.store;\n\
         li16 :: Ptr -> Int16 := Int16.load; si16 :: (Ptr, Int16) -> Unit := Int16.store;\n\
         li32 :: Ptr -> Int32 := Int32.load; si32 :: (Ptr, Int32) -> Unit := Int32.store;\n\
         li64 :: Ptr -> Int64 := Int64.load; si64 :: (Ptr, Int64) -> Unit := Int64.store;\n\
         lu8 :: Ptr -> UInt8 := UInt8.load; su8 :: (Ptr, UInt8) -> Unit := UInt8.store;\n\
         lu16 :: Ptr -> UInt16 := UInt16.load; su16 :: (Ptr, UInt16) -> Unit := UInt16.store;\n\
         lu32 :: Ptr -> UInt32 := UInt32.load; su32 :: (Ptr, UInt32) -> Unit := UInt32.store;\n\
         lu64 :: Ptr -> UInt64 := UInt64.load; su64 :: (Ptr, UInt64) -> Unit := UInt64.store;\n\
         lf32 :: Ptr -> Float32 := Float32.load; sf32 :: (Ptr, Float32) -> Unit := Float32.store;\n\
         lf64 :: Ptr -> Float64 := Float64.load; sf64 :: (Ptr, Float64) -> Unit := Float64.store;\n\
         lp :: Ptr -> Ptr := Ptr.load; sp :: (Ptr, Ptr) -> Unit := Ptr.store;\n\
         le :: (Ptr, UInt64) -> Symbol := Symbol.read; se :: (Ptr, Symbol) -> Unit := Symbol.write;",
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
