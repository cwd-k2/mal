use super::*;

#[test]
fn emits_every_fixed_width_scalar_in_the_generated_header() {
    let generated = emit(
        "extern i8 :: Int8 -> Int8;\n\
         extern i16 :: Int16 -> Int16;\n\
         extern i32 :: Int32 -> Int32;\n\
         extern i64 :: Int64 -> Int64;\n\
         extern u8 :: UInt8 -> UInt8;\n\
         extern u16 :: UInt16 -> UInt16;\n\
         extern u32 :: UInt32 -> UInt32;\n\
         extern u64 :: UInt64 -> UInt64;\n\
         main :: Unit -> Int32 := \\() { 0; };",
    )
    .expect("emit C");
    for declaration in [
        "MalType_Int8 mal_ext_i8(MalContext *context, MalType_Int8 value);",
        "MalType_Int16 mal_ext_i16(MalContext *context, MalType_Int16 value);",
        "MalType_Int32 mal_ext_i32(MalContext *context, MalType_Int32 value);",
        "MalType_Int64 mal_ext_i64(MalContext *context, MalType_Int64 value);",
        "MalType_UInt8 mal_ext_u8(MalContext *context, MalType_UInt8 value);",
        "MalType_UInt16 mal_ext_u16(MalContext *context, MalType_UInt16 value);",
        "MalType_UInt32 mal_ext_u32(MalContext *context, MalType_UInt32 value);",
        "MalType_UInt64 mal_ext_u64(MalContext *context, MalType_UInt64 value);",
    ] {
        assert!(
            contains_ignoring_whitespace(&generated.header, declaration),
            "{declaration}"
        );
    }
}

#[test]
fn executes_unaligned_ptr_access_for_every_numeric_scalar() {
    let source = "extern memory :: Unit -> Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
           base := extern memory();\n\
           p0 := base + 1u64; storeInt8(p0, -8i8);\n\
           p1 := base + 3u64; storeInt16(p1, -16i16);\n\
           p2 := base + 6u64; storeInt32(p2, -32i32);\n\
           p3 := base + 11u64; storeInt64(p3, -64i64);\n\
           p4 := base + 20u64; storeUInt8(p4, 8u8);\n\
           p5 := base + 22u64; storeUInt16(p5, 16u16);\n\
           p6 := base + 25u64; storeUInt32(p6, 32u32);\n\
           p7 := base + 30u64; storeUInt64(p7, 64u64);\n\
           p8 := base + 39u64; storeFloat32(p8, 1.5f32);\n\
           end := base + 52u64;\n\
           p9 := end - 8u64; storeFloat64(p9, -2.5f64);\n\
           ok := (loadInt8(p0) == -8i8) && (loadInt16(p1) == -16i16) &&\n\
                 (loadInt32(p2) == -32i32) && (loadInt64(p3) == -64i64) &&\n\
                 (loadUInt8(p4) == 8u8) && (loadUInt16(p5) == 16u16) &&\n\
                 (loadUInt32(p6) == 32u32) && (loadUInt64(p7) == 64u64) &&\n\
                 (loadFloat32(p8) == 1.5f32) && (loadFloat64(p9) == -2.5f64);\n\
           if (ok) then { 0 } else { 1 };\n\
         };";
    let generated = emit(source).expect("emit Ptr operations");
    assert!(
        generated
            .header
            .contains("typedef struct { uint8_t *address; } MalType_Ptr;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Ptr mal_ext_memory(MalContext *context);"
    ));
    let host = r#"#include "program.mal.h"

MalType_Ptr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[52];
    (void)context;
    return (MalType_Ptr){ .address = bytes };
}
"#;
    let fixture = NativeFixture::new("ptr-memory");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn executes_first_class_memory_functions_through_closure_calls() {
    let source = "Reader :: Ptr -> Int64;\n\
         Writer :: (Ptr, Int64) -> Unit;\n\
         extern memory :: Unit -> Ptr;\n\
         readWith :: (Reader, Ptr) -> Int64 := \\(reader :: Reader, pointer :: Ptr) {\n\
           reader(pointer);\n\
         };\n\
         writeWith :: (Writer, Ptr, Int64) -> Unit := \\(writer :: Writer, pointer :: Ptr, value :: Int64) {\n\
           writer(pointer, value);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           pointer := extern memory();\n\
           writeWith(storeInt64, pointer, 42i64);\n\
           Int32(readWith(loadInt64, pointer) - 42i64);\n\
         };";
    let generated = emit(source).expect("emit first-class memory functions");
    assert!(generated.source.contains("mal_memory_function_load_int64"));
    assert!(generated.source.contains("mal_memory_function_store_int64"));
    let host = r#"#include "program.mal.h"

MalType_Ptr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[8];
    (void)context;
    return (MalType_Ptr){ .address = bytes };
}
"#;
    let fixture = NativeFixture::new("first-class-memory");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn executes_unaligned_ptr_value_access() {
    let source = "extern pointerSlot :: Unit -> Ptr;\n\
         extern target :: Unit -> Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
           slot := extern pointerSlot() + 1u64;\n\
           storePtr(slot, extern target());\n\
           stored := loadPtr(slot);\n\
           storeInt32(stored, 42i32);\n\
           loadInt32(extern target()) - 42;\n\
         };";
    let generated = emit(source).expect("emit Ptr value access");
    assert!(generated.source.contains("mal_load_ptr"));
    assert!(generated.source.contains("mal_store_ptr"));
    let host = r#"#include "program.mal.h"

MalType_Ptr mal_ext_pointerSlot(MalContext *context) {
    static uint8_t bytes[sizeof(MalType_Ptr) + 1];
    (void)context;
    return mal_Ptr_from_address(bytes);
}

MalType_Ptr mal_ext_target(MalContext *context) {
    static uint8_t bytes[sizeof(int32_t)];
    (void)context;
    return mal_Ptr_from_address(bytes);
}
"#;
    let fixture = NativeFixture::new("ptr-value-memory");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn executes_target_storage_size_expressions() {
    let output = compile_and_run(
        "extern expectedSize :: Unit -> UInt64;\n\
         pointerSize :: UInt64 := @Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
           actual := @Int8 + @Int16 + @Int32 + @Int64\n\
             + @UInt8 + @UInt16 + @UInt32 + @UInt64\n\
             + @Float32 + @Float64 + pointerSize;\n\
           if (actual == extern expectedSize()) then { 0 } else { 1 };\n\
         };",
        r#"#include "program.mal.h"

uint64_t mal_ext_expectedSize(MalContext *context) {
    (void)context;
    return UINT64_C(42) + (uint64_t)sizeof(MalType_Ptr);
}
"#,
    );
    assert!(output.status.success());
}

#[test]
fn copies_symbols_between_mal_and_external_memory() {
    let source = "extern symbolSlot :: Unit -> Ptr;\n\
         extern inspectSymbolSlot :: Unit -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           slot := extern symbolSlot() + 1u64;\n\
           initial := loadSymbol(slot, 4u64);\n\
           storeSymbol(slot, \"held\\0\\xff\");\n\
           extern inspectSymbolSlot();\n\
           stored := loadSymbol(slot, 6u64);\n\
           if ((initial == \"seed\") && (stored == \"held\\0\\xff\") && (stored # 5u64 == 255u8)) then {\n\
             0\n\
           } else {\n\
             1\n\
           };\n\
         };";
    let generated = emit(source).expect("emit Symbol byte copies");
    assert!(generated.source.contains("mal_load_symbol"));
    assert!(generated.source.contains("mal_store_symbol"));
    let host = r#"#include "program.mal.h"
#include <string.h>

static uint8_t slot[7];

MalType_Ptr mal_ext_symbolSlot(MalContext *context) {
    static const uint8_t seed[] = { 's', 'e', 'e', 'd' };
    (void)context;
    memcpy(slot + 1, seed, sizeof(seed));
    return mal_Ptr_from_address(slot);
}

void mal_ext_inspectSymbolSlot(MalContext *context) {
    static const uint8_t expected[] = { 'h', 'e', 'l', 'd', 0, 255 };
    if (memcmp(slot + 1, expected, sizeof(expected)) != 0) {
        mal_trap(context, "unexpected stored Symbol bytes");
    }
}
"#;
    let fixture = NativeFixture::new("symbol-value-memory");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn emits_only_required_memory_helpers_and_compiles_with_optimization() {
    let generated = emit(
        "extern memory :: Unit -> Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
           pointer := extern memory();\n\
           storeInt64(pointer, 42i64);\n\
           Int32(loadInt64(pointer) - 42i64);\n\
         };",
    )
    .expect("emit selective memory helpers");

    assert!(generated.source.contains("mal_load_int64"));
    assert!(generated.source.contains("mal_store_int64"));
    assert!(!generated.source.contains("mal_ptr_offset"));
    assert!(!generated.source.contains("mal_load_int8"));
    assert!(!generated.source.contains("mal_store_float64"));
    assert!(!generated.source.contains("mal_load_symbol"));
    assert!(!generated.source.contains("mal_store_symbol"));

    let fixture = NativeFixture::new("selective-memory-runtime");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MalType_Ptr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[8];
    (void)context;
    return (MalType_Ptr){ .address = bytes };
}
"#,
        &["-O2"],
    );
    assert!(fixture.run(executable).status.success());
}
