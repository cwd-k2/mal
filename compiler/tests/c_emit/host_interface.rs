use super::*;

#[test]
fn exposes_aggregate_extern_types_and_executes_the_host_round_trip() {
    let source = "Request :: (Int32, (UInt8, Int32));\n\
         Response :: [Unit, (Int32, Int32)];\n\
         extern exchange :: Request -> Response;\n\
         total :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           left + right - 42i32;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           response := extern exchange(20i32, (2u8, 22i32));\n\
           case (response)\n\
             [0](_) { 1 }\n\
             [1](pair) { total(pair) };\n\
         };";
    let generated = emit(source).expect("emit aggregate ABI");
    assert!(
        generated
            .header
            .contains("typedef MalRepr_Product_1 MalType_Request;")
    );
    assert!(
        generated
            .header
            .contains("typedef MalRepr_Sum_3 MalType_Response;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Response mal_ext_exchange(MalContext *context, MalType_Int32 argument_0, \
         MalRepr_Product_0 argument_1);"
    ));
    assert!(
        generated
            .header
            .contains("#define MAL_DEFINE_exchange(context, argument_0, argument_1) \\")
    );
    assert!(
        generated
            .header
            .contains("MalType_Response mal_ext_exchange( \\")
    );
    assert!(
        generated
            .header
            .contains("MalContext *context MAL_DETAIL_MAYBE_UNUSED, \\")
    );
    assert!(generated.header.contains("MalRepr_Product_0 argument_1 \\"));
    assert!(generated.header.contains(
        "struct MalRepr_Product_0 {\n    MalType_UInt8 field_0;\n    MalType_Int32 field_1;\n};"
    ));
    assert!(generated.header.contains("MalRepr_Product_2 variant_1;"));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_exchange(context, argument_0, argument_1) {
    MAL_TYPE(Request) request = MAL_OPERATION(Request, make)(argument_0, argument_1);
    return MAL_OPERATION(Response, make_1)(
        MAL_OPERATION(Request, get_0)(request),
        MAL_OPERATION(Request, get_1)(request).field_1
    );
}

"#;
    let fixture = NativeFixture::new("aggregate-abi");
    let executable = fixture.compile_generated(generated.clone(), host);
    assert!(fixture.run(executable).status.success());

    let invalid_host = r#"#include "program.mal.h"

MalType_Response mal_ext_exchange(
    MalContext *context,
    MalType_Int32 argument_0,
    MalRepr_Product_0 argument_1
) {
    (void)context;
    (void)argument_0;
    (void)argument_1;
    return (MalType_Response){ .tag = UINT32_C(99) };
}
"#;
    let fixture = NativeFixture::new("aggregate-invalid-tag");
    let executable = fixture.compile_generated(generated, invalid_host);
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("mal trap: invalid sum tag"));
}

#[test]
fn exposes_scalar_alias_names_in_the_host_header() {
    let generated = emit(
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;\n\
         main :: Unit -> Int32 := \\() { Int32(extern increment(41u64) - 42u64); };",
    )
    .expect("emit scalar alias ABI");

    assert!(
        generated
            .header
            .contains("typedef MalType_UInt64 MalType_Count;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Count mal_ext_increment(MalContext *context, MalType_Count value);"
    ));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_increment(context, value) {
    return value + UINT64_C(1);
}
"#;
    let fixture = NativeFixture::new("scalar-alias-abi");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn preserves_the_aliases_spelled_in_an_extern_declaration() {
    let generated = emit(
        "FirstCount :: UInt64;\n\
         SecondCount :: UInt64;\n\
         extern increment :: FirstCount -> SecondCount;\n\
         main :: Unit -> Int32 := \\() { Int32(extern increment(41u64) - 42u64); };",
    )
    .expect("emit explicitly named scalar aliases");

    assert!(
        generated
            .header
            .contains("typedef MalType_UInt64 MalType_FirstCount;")
    );
    assert!(
        generated
            .header
            .contains("typedef MalType_UInt64 MalType_SecondCount;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_SecondCount mal_ext_increment(MalContext *context, MalType_FirstCount value);"
    ));
}

#[test]
fn preserves_aliases_inside_a_flattened_parameter_alias() {
    let generated = emit(
        "Count :: UInt64;\n\
         Payload :: (UInt8, Int32);\n\
         Request :: (Count, Payload);\n\
         extern exchange :: Request -> Count;\n\
         main :: Unit -> Int32 := \\() { 0; };",
    )
    .expect("emit aliases from a flattened parameter");

    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Count mal_ext_exchange(MalContext *context, MalType_Count argument_0, \
         MalType_Payload argument_1);"
    ));
    assert!(generated.header.contains("MalType_Request"));
}

#[test]
fn generated_sum_helpers_construct_and_inspect_named_variants() {
    let generated = emit(
        "Pair :: (Int32, Int32);\n\
         Choice :: [Unit, Pair];\n\
         extern inspect :: Choice -> Int32;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern inspect(Choice[1]((20i32, 22i32))) - 42i32;\n\
         };",
    )
    .expect("emit named sum helpers");

    assert!(contains_ignoring_whitespace(
        &generated.header,
        "static inline MalType_Choice mal_Choice_make_1(MalType_Int32 value_0, \
         MalType_Int32 value_1)"
    ));
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "static inline MalType_Int32 mal_Choice_expect_1_0(MalContext *context, \
         MalType_Choice value)"
    ));
    assert!(
        generated
            .header
            .contains("#define MAL_TYPE(name) MalType_##name")
    );
    assert!(
        generated
            .header
            .contains("#define MAL_OPERATION(type, operation) mal_##type##_##operation")
    );
    assert!(
        generated
            .header
            .contains("#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant")
    );
    assert!(
        generated
            .header
            .contains("#define MAL_EXTERN(name) mal_ext_##name")
    );

    let host = r#"#include "program.mal.h"

MAL_DEFINE_inspect(context, value) {
    MAL_TYPE(Choice) copy = value;
    if (!MAL_OPERATION(Choice, is_1)(copy) ||
        MAL_OPERATION(Choice, tag)(copy) != MAL_TAG(Choice, 1)) {
        mal_trap(context, "expected pair");
    }
    return MAL_OPERATION(Choice, expect_1_0)(context, copy) +
           MAL_OPERATION(Choice, expect_1_1)(context, copy);
}
"#;
    let fixture = NativeFixture::new("named-sum-helpers");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn exposes_copyable_opaque_handles_to_the_host() {
    let source = "extern Mem;\n\
         extern allocate :: UInt64 -> Mem;\n\
         extern combinedLength :: (Mem, Mem) -> UInt64;\n\
         main :: Unit -> Int32 := \\() {\n\
           mem := extern allocate(21u64);\n\
           Int32(extern combinedLength(mem, mem) - 42u64);\n\
         };";
    let generated = emit(source).expect("emit opaque ABI");
    assert!(
        generated
            .header
            .contains("typedef struct { uintptr_t bits; } MalType_Mem;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_UInt64 mal_ext_combinedLength(MalContext *context, \
         MalType_Mem argument_0, MalType_Mem argument_1);"
    ));
    let host = r#"#include "program.mal.h"

MalType_Mem MAL_EXTERN(allocate)(MalContext *context, MalType_UInt64 value) {
    (void)context;
    return (MalType_Mem){ .bits = (uintptr_t)value };
}

MalType_UInt64 mal_ext_combinedLength(
    MalContext *context,
    MalType_Mem first,
    MalType_Mem second
) {
    (void)context;
    return (uint64_t)first.bits + (uint64_t)second.bits;
}
"#;
    let fixture = NativeFixture::new("opaque-abi");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn preserves_duplicate_sum_members_by_tag() {
    let output = compile_and_run(
        "Pair :: (Int32, Int32);\n\
         Choice :: [Pair, Pair];\n\
         extern choose :: Unit -> Choice;\n\
         difference :: Pair -> Int32 := \\(pair :: Pair) {\n\
           (left, right) := pair;\n\
           left - right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           case (extern choose())\n\
             [0](pair) { difference(pair) + 1i32 }\n\
             [1](pair) { difference(pair) };\n\
         };",
        r#"#include "program.mal.h"

MalRepr_Sum_1 mal_ext_choose(MalContext *context) {
    (void)context;
    return (MalRepr_Sum_1){
        .tag = UINT32_C(1),
        .payload.variant_1 = {
            .field_0 = INT32_C(42),
            .field_1 = INT32_C(42),
        },
    };
}
"#,
    );
    assert!(output.status.success());
}
