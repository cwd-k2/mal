use super::*;

#[test]
fn exposes_aggregate_extern_types_and_executes_the_host_round_trip() {
    let source = "Request :: (Int32, (UInt8, Int32));\n\
         Pair :: (Int32, Int32);\n\
         Response :: [Unit, Pair];\n\
         extern exchange :: Request -> Response;\n\
         total :: (Int32, Int32) -> Int32 := \\(left, right) {\n\
           left + right - 42i32;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           response := exchange(20i32, (2u8, 22i32));\n\
           case (response)\n\
             [0](_) { 1 }\n\
             [1](pair) { total(pair) };\n\
         };";
    let generated = emit(source).expect("emit aggregate ABI");
    assert!(
        generated
            .header
            .contains("typedef mal_repr_product_1_t mal_Request_t;")
    );
    assert!(
        generated
            .header
            .contains("typedef mal_repr_sum_3_t mal_Response_t;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Response mal_ext_exchange(MalContext *context, MalType_Int32 argument_0, \
         MalRepr_Product_0 argument_1);"
    ));
    assert!(
        generated
            .header
            .contains("#define MAL_DEFINE_exchange(call, value) \\")
    );
    assert!(generated.header.contains(
        "static MalType_Response mal_detail_exchange(mal_call_t *call, mal_Request_t value);"
    ));
    assert!(generated.header.contains("mal_repr_product_0_t field_1;"));
    assert!(
        generated.header.contains("mal_Pair_t variant_1;")
            || generated.header.contains("mal_repr_product_2_t variant_1;")
    );

    let host = r#"#include "program.mal.h"

MAL_DEFINE_exchange(call, value) {
    return mal_Response_return_1(
        call,
        (mal_Pair_t){
            .field_0 = value.field_0,
            .field_1 = value.field_1.field_1,
        }
    );
}

"#;
    let fixture = NativeFixture::new("aggregate-abi");
    let executable = fixture.compile_generated(generated.clone(), host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn returns_borrowed_managed_values_through_typed_helpers() {
    let generated = emit(
        r#"Pair :: (Symbol, Symbol);
Response :: [Unit, Pair];
extern duplicate :: Symbol -> Response;
main :: Unit -> Int32 := \() {
  response := duplicate("host");
  case (response)
    [0](_) { 1 }
    [1](pair) {
      (left, right) := pair;
      if ((left == "host") && (right == "host")) then { 0 } else { 2 };
    };
};"#,
    )
    .expect("emit managed host return helpers");

    assert!(generated.header.contains("mal_Symbol_return"));
    assert!(generated.header.contains("mal_Response_return_1"));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_duplicate(call, value) {
    return mal_Response_return_1(
        call,
        (mal_Pair_t){ .field_0 = value, .field_1 = value }
    );
}
"#;
    let fixture = NativeFixture::new("managed-host-ownership");
    let executable = fixture.compile_generated_with_options(
        generated,
        host,
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "status: {:?}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn exposes_scalar_alias_names_in_the_host_header() {
    let generated = emit(
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;\n\
         main :: Unit -> Int32 := \\() { Int32(increment(41u64) - 42u64); };",
    )
    .expect("emit scalar alias ABI");

    assert!(
        generated
            .header
            .contains("typedef mal_UInt64_t mal_Count_t;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Count mal_ext_increment(MalContext *context, MalType_Count value);"
    ));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_increment(call, value) {
    return mal_Count_return(call, value + UINT64_C(1));
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
         main :: Unit -> Int32 := \\() { Int32(increment(41u64) - 42u64); };",
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
fn generated_sum_helpers_construct_and_return_named_variants() {
    let generated = emit(
        "Pair :: (Int32, Int32);\n\
         Choice :: [Unit, Pair];\n\
         extern inspect :: Choice -> Int32;\n\
         main :: Unit -> Int32 := \\() {\n\
           inspect(Choice[1]((20i32, 22i32))) - 42i32;\n\
         };",
    )
    .expect("emit named sum helpers");

    assert!(contains_ignoring_whitespace(
        &generated.header,
        "static inline MalType_Choice mal_Choice_return_1(mal_call_t *call, mal_Pair_t value)"
    ));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_inspect(call, value) {
    if (value.tag != mal_Choice_tag_1) {
        mal_call_trap(call, "expected pair");
    }
    return mal_Int32_return(
        call,
        value.payload.variant_1.field_0 + value.payload.variant_1.field_1
    );
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
           mem := allocate(21u64);\n\
           Int32(combinedLength(mem, mem) - 42u64);\n\
         };";
    let generated = emit(source).expect("emit opaque ABI");
    assert!(
        generated
            .header
            .contains("typedef struct { uintptr_t mal_detail_bits; } mal_Mem_t;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_UInt64 mal_ext_combinedLength(MalContext *context, \
         MalType_Mem argument_0, MalType_Mem argument_1);"
    ));
    let host = r#"#include "program.mal.h"

MAL_DEFINE_allocate(call, value) {
    return mal_Mem_return(call, mal_Mem_from_bits((uintptr_t)value));
}

MAL_DEFINE_combinedLength(call, value) {
    return mal_UInt64_return(
        call,
        (uint64_t)mal_Mem_to_bits(value.field_0) +
        (uint64_t)mal_Mem_to_bits(value.field_1)
    );
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
         difference :: Pair -> Int32 := \\(pair) {\n\
           (left, right) := pair;\n\
           left - right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           case (choose())\n\
             [0](pair) { difference(pair) + 1i32 }\n\
             [1](pair) { difference(pair) };\n\
         };",
        r#"#include "program.mal.h"

MAL_DEFINE_choose(call) {
    return mal_Choice_return_1(
        call,
        (mal_Pair_t){
            .field_0 = INT32_C(42),
            .field_1 = INT32_C(42),
        }
    );
}

"#,
    );
    assert!(output.status.success());
}

#[test]
fn traps_invalid_nested_sum_tags_before_reading_the_payload() {
    let generated = emit(
        "Choice :: [Unit, Int32];\n\
         Envelope :: (Choice, Int32);\n\
         extern invalid :: Unit -> Envelope;\n\
         main :: Unit -> Int32 := \\() { invalid(); 0; };",
    )
    .expect("emit nested sum validation");
    let fixture = NativeFixture::new("invalid-nested-sum-tag");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"

MAL_DEFINE_invalid(call) {
    mal_Envelope_t value = {
        .field_0 = { .tag = UINT32_C(99) },
        .field_1 = INT32_C(0),
    };
    return mal_Envelope_return(call, value);
}
"#,
    );
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid sum tag"));
}

#[test]
fn traps_invalid_bool_results() {
    let generated = emit(
        "extern invalid :: Unit -> Bool;\n\
         main :: Unit -> Int32 := \\() { if (invalid()) then { 0 } else { 1 }; };",
    )
    .expect("emit Bool validation");
    let fixture = NativeFixture::new("invalid-bool-result");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"

MAL_DEFINE_invalid(call) {
    return mal_Bool_return(call, (mal_Bool_t)UINT8_C(2));
}
"#,
    );
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid Bool"));
}

#[test]
fn reports_reference_count_overflow_as_an_implementation_resource_failure() {
    let generated = emit(
        r#"extern inspect :: Symbol -> Symbol;
main :: Unit -> Int32 := \() {
  inspect("left" + "right");
  0;
};"#,
    )
    .expect("emit reference-count overflow fixture");
    let fixture = NativeFixture::new("reference-count-resource-failure");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MAL_DEFINE_inspect(call, value) {
    return mal_Symbol_return(call, value);
}
"#,
        &["-DMAL_TEST_FORCE_REFERENCE_COUNT_OVERFLOW"],
    );
    let output = fixture.run(executable);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("mal implementation resource failure: reference count overflow"),
        "{stderr}"
    );
    assert!(!stderr.contains("mal trap:"), "{stderr}");
}
