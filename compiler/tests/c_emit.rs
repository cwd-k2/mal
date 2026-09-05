use malc::anf;
use malc::c_emit;
use malc::check;
use malc::closure;
use malc::core;
use malc::parser;
use malc::resolve;
use malc::source::{FileId, SourceFile};

mod support;

use support::NativeFixture;

fn emit(text: &str) -> Result<c_emit::Output, malc::diagnostic::Diagnostic> {
    let source = SourceFile::new(FileId::new(79), "c-emit-test.mal", text.into());
    let parsed = parser::parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let checked =
        check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    c_emit::emit(&closure)
}

fn contains_ignoring_whitespace(haystack: &str, needle: &str) -> bool {
    let compact = |text: &str| {
        text.chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    };
    compact(haystack).contains(&compact(needle))
}

fn compile_and_run(source: &str, host: &str) -> std::process::Output {
    let generated = emit(source).expect("emit C");
    let fixture = NativeFixture::new("c-emit");
    let executable = fixture.compile_generated(generated, host);
    fixture.run(executable)
}

const PRINT_HOST: &str = r#"#include "program.mal.h"
#include <stdio.h>

#if MAL_HAS_EXTERN_printInt32 != 1
#error "missing printInt32 capability"
#endif

MAL_DEFINE_printInt32(context, value) {
    printf("%d\n", value);
}
"#;

#[test]
fn emits_the_basic_host_abi_and_executes_the_host_example() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn represents_bool_as_zero_or_one_across_the_c_abi() {
    let generated = emit(
        "Envelope :: (Bool, (Bool, Int32));\n\
         extern exchange :: Envelope -> Bool;\n\
         main :: Unit -> Int32 := \\() {\n\
           accepted := extern exchange(true, (false, 42));\n\
           if (accepted) then { 0 } else { 1 };\n\
         };",
    )
    .expect("emit scalar Bool ABI");

    assert!(contains_ignoring_whitespace(
        &generated.header,
        "uint8_t mal_ext_exchange(MalContext *context, uint8_t argument_0, \
         MalProduct_0 argument_1);"
    ));
    assert!(
        generated
            .header
            .contains("struct MalProduct_0 {\n    uint8_t field_0;\n    int32_t field_1;\n};")
    );
    assert!(!generated.header.contains("MalSum_"));
    assert!(!generated.source.contains(".tag"));

    let fixture = NativeFixture::new("scalar-bool-abi");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"

uint8_t mal_ext_exchange(
    MalContext *context,
    uint8_t outer,
    MalProduct_0 inner
) {
    (void)context;
    return (outer == UINT8_C(1) && inner.field_0 == UINT8_C(0) &&
            inner.field_1 == INT32_C(42)) ? UINT8_C(1) : UINT8_C(0);
}
"#,
    );
    assert!(fixture.run(executable).status.success());
}

#[test]
fn executes_escaping_capturing_closures() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         makeAdder :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           \\<x>(y :: Int32) { x + y; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           addTen := makeAdder(10);\n\
           extern printInt32(addTen(5));\n\
           0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "15\n");
}

#[test]
fn executes_top_level_and_local_recursive_closures() {
    let output = compile_and_run(
        "factorial :: Int32 -> Int32 := \\(n :: Int32) {\n\
           if (n == 0) then { 1 } else { n * factorial(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           base :: Int32 := 120;\n\
           local :: Int32 -> Int32 := \\<base>(n :: Int32) {\n\
             if (n == 0) then { base } else { local(n - 1) };\n\
           };\n\
           local(3) - factorial(5);\n\
         };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emits_direct_calls_for_known_top_level_and_self_functions() {
    let generated = emit(
        "square :: Int64 -> Int64 := \\(value :: Int64) { value * value; };\n\
         factorial :: Int64 -> Int64 := \\(value :: Int64) {\n\
           if (value == 0i64) then { 1i64 } else { value * factorial(value - 1i64) };\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(square(factorial(3i64)) - 36i64); };",
    )
    .expect("emit direct calls");
    assert!(
        generated
            .source
            .contains("mal_function_0(mal_context, NULL,")
    );
    assert!(
        generated
            .source
            .contains("mal_function_1(mal_context, mal_environment,")
    );
    let fixture = NativeFixture::new("direct-known-calls");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn labels_generated_functions_with_top_level_source_bindings() {
    let generated = emit(
        "double :: Int64 -> Int64 := \\(value :: Int64) { value * 2i64; };\n\
         main :: Unit -> Int32 := \\() { Int32(double(21i64) - 42i64); };",
    )
    .expect("emit labeled functions");

    assert!(
        generated
            .source
            .contains("/* mal source binding: double */\nstatic int64_t mal_function_0(")
    );
    assert!(
        generated
            .source
            .contains("/* mal source binding: main */\nstatic int32_t mal_function_1(")
    );
}

#[test]
fn destructures_product_atoms_without_copying_the_product() {
    let generated = emit(
        "Pair :: (Int64, Int64);\n\
         sum :: Pair -> Int64 := \\(pair :: Pair) {\n\
           (left, right) := pair;\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(sum((20i64, 22i64)) - 42i64); };",
    )
    .expect("emit product destructuring");
    assert!(!generated.source.contains("mal_discard_"));

    let fixture = NativeFixture::new("direct-product-destructure");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn passes_known_product_arguments_through_a_direct_entry() {
    let generated = emit(
        "combine :: (Int64, Int64) -> Int64 := \\(left :: Int64, right :: Int64) {\n\
           left + right;\n\
         };\n\
         apply :: ((Int64, Int64) -> Int64) -> Int64 := \\(operation :: (Int64, Int64) -> Int64) {\n\
           operation(20i64, 22i64);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           Int32(combine(20i64, 22i64) + apply(combine) - 84i64);\n\
         };",
    )
    .expect("emit a direct product entry");

    assert!(generated.source.contains(
        "static int64_t mal_direct_function_0(MalContext *mal_context, \
         const void *mal_environment, int64_t mal_direct_parameter_0, \
         int64_t mal_direct_parameter_1)"
    ));
    assert!(generated.source.contains(
        "return mal_direct_function_0(mal_context, mal_environment, \
         mal_value_core_0.field_0, mal_value_core_0.field_1);"
    ));
    assert!(
        generated
            .source
            .contains("mal_direct_function_0(mal_context, NULL,")
    );

    let fixture = NativeFixture::new("known-product-entry");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn flattens_nested_products_only_at_known_call_entries() {
    let generated = emit(
        "combine :: ((Int64, Int64), Int64) -> Int64 := \\(pair :: (Int64, Int64), extra :: Int64) {\n\
           (left, right) := pair;\n\
           left + right + extra;\n\
         };\n\
         apply :: (((Int64, Int64), Int64) -> Int64) -> Int64 := \\(operation :: ((Int64, Int64), Int64) -> Int64) {\n\
           operation((20i64, 21i64), 1i64);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           Int32(combine((20i64, 21i64), 1i64) + apply(combine) - 84i64);\n\
         };",
    )
    .expect("emit nested direct product fields");

    assert!(generated.source.contains(
        "int64_t mal_direct_parameter_0, int64_t mal_direct_parameter_1, \
         int64_t mal_direct_parameter_2)"
    ));
    assert!(generated.source.contains(".field_0.field_0"));
    assert!(generated.source.contains(".field_0.field_1"));

    let fixture = NativeFixture::new("nested-known-product-entry");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn keeps_large_product_calls_on_the_aggregate_fallback() {
    let types = std::iter::repeat_n("Int64", 17)
        .collect::<Vec<_>>()
        .join(", ");
    let parameters = (0..17)
        .map(|index| format!("value{index} :: Int64"))
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = std::iter::repeat_n("0i64", 17)
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "select :: ({types}) -> Int64 := \\({parameters}) {{ value0; }};\n\
         main :: Unit -> Int32 := \\() {{ Int32(select({arguments})); }};"
    );
    let generated = emit(&source).expect("emit aggregate fallback");

    assert!(!generated.source.contains("mal_direct_function_0"));
    let fixture = NativeFixture::new("large-product-fallback");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn lowers_direct_tail_recursion_without_growing_the_c_stack() {
    let source = "count :: (Int64, Int64) -> Int64 := \\(remaining :: Int64, total :: Int64) {\n\
           if (remaining == 0) then { total } else {\n\
             count(remaining - 1, total + 1)\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           if (count(1000000i64, 0) == 1000000i64) then { 0 } else { 1 };\n\
         };";
    let generated = emit(source).expect("emit tail-recursive C");
    assert!(generated.source.contains("goto mal_tail_entry;"));

    let fixture = NativeFixture::new("direct-tail-recursion");
    let executable = fixture.compile_generated(generated, "");
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn preserves_effect_order_before_a_direct_tail_call() {
    let output = compile_and_run(
        "extern step :: Int32 -> Int32;\n\
         walk :: (Int32, Int32) -> Int32 := \\(remaining :: Int32, total :: Int32) {\n\
           if (remaining == 0) then { total } else {\n\
             walk(extern step(remaining), total + 1)\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { walk(4, 0) - 4; };",
        r#"#include "program.mal.h"

static int32_t expected = INT32_C(4);

int32_t mal_ext_step(MalContext *context, int32_t value) {
    if (value != expected) {
        mal_trap(context, "tail-call effect order changed");
    }
    expected -= INT32_C(1);
    return value - INT32_C(1);
}
"#,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emits_uint64_literals_and_scalar_extern_abi() {
    let output = compile_and_run(
        "extern printUInt64 :: UInt64 -> Unit;\n\
         capture :: UInt64 -> (Unit -> UInt64) := \\(value :: UInt64) {\n\
           \\<value>() { value; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printUInt64(capture(18446744073709551615u64)());\n\
           0;\n\
         };",
        r#"#include "program.mal.h"
#include <inttypes.h>
#include <stdio.h>

void mal_ext_printUInt64(MalContext *context, uint64_t value) {
    (void)context;
    printf("%" PRIu64 "\n", value);
}
"#,
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "18446744073709551615\n"
    );
}

#[test]
fn emits_exact_float_bits_and_scalar_extern_abi() {
    let generated = emit(
        "extern inspect :: (Float32, Float64) -> Int32;\n\
         main :: Unit -> Int32 := \\() { extern inspect(0.1f32, -0.0f64); };",
    )
    .expect("emit Float ABI");
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "int32_t mal_ext_inspect(MalContext *context, float argument_0, double argument_1);"
    ));
    assert!(generated.source.contains("#pragma STDC FP_CONTRACT OFF"));
    let fixture = NativeFixture::new("float-bits");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"
#include <string.h>

int32_t mal_ext_inspect(MalContext *context, float single, double negative_zero) {
    (void)context;
    uint32_t single_bits;
    uint64_t double_bits;
    memcpy(&single_bits, &single, sizeof(single_bits));
    memcpy(&double_bits, &negative_zero, sizeof(double_bits));
    return single_bits == UINT32_C(0x3dcccccd) &&
           double_bits == UINT64_C(0x8000000000000000) ? INT32_C(0) : INT32_C(1);
}
"#,
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn executes_strict_float_arithmetic_and_ieee_comparisons() {
    let output = compile_and_run(
        "main :: Unit -> Int32 := \\() {\n\
           infinity := 1.0f32 / 0.0f32;\n\
           nan := 0.0f32 / 0.0f32;\n\
           rounded := (16777216.0f32 + 1.0f32) - 16777216.0f32;\n\
           subnormal := 1.40129846e-45f32;\n\
           valid := infinity > 1.0f32 &&\n\
                    nan != nan && !(nan == nan) &&\n\
                    0.0f32 == -0.0f32 &&\n\
                    rounded == 0.0f32 && subnormal > 0.0f32;\n\
           if (valid) then { 0 } else { 1 };\n\
         };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn executes_ties_to_even_numeric_float_conversions() {
    let output = compile_and_run(
        "main :: Unit -> Int32 := \\() {\n\
           lowerEven := Float32(16777217u64);\n\
           upperEven := Float32(16777219u64);\n\
           widened := Float64(0.1f32);\n\
           narrowed := Float32(widened);\n\
           valid := lowerEven == 16777216.0f32 &&\n\
                    upperEven == 16777220.0f32 &&\n\
                    narrowed == 0.1f32 &&\n\
                    Int8(-128.75f64) == -128i8 &&\n\
                    UInt8(-0.5f32) == 0u8;\n\
           if (valid) then { 0 } else { 1 };\n\
         };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn traps_invalid_float_to_integer_conversions_before_the_c_cast() {
    for expression in [
        "UInt8(0.0f32 / 0.0f32)",
        "Int64(1.0f64 / 0.0f64)",
        "UInt8(-1.0f32)",
        "Int8(128.0f64)",
    ] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("float-to-integer conversion out of range"),
            "expression: {expression}"
        );
    }
}

#[test]
fn emits_static_string_bytes_that_survive_closure_escape() {
    let output = compile_and_run(
        r#"extern inspect :: String -> Unit;
make :: String -> (Unit -> String) := \(value :: String) {
  \<value>() { value; };
};
main :: Unit -> Int32 := \() {
  extern inspect("あ\0\xff");
  held := make("scope");
  extern inspect(held());
  extern inspect("");
  0;
};"#,
        r#"#include "program.mal.h"
#include <inttypes.h>
#include <stdio.h>

void mal_ext_inspect(MalContext *context, MalString value) {
    (void)context;
    printf("%" PRIu64 ":", value.length);
    for (uint64_t index = 0; index < value.length; index += UINT64_C(1)) {
        printf("%02x", value.data[index]);
    }
    putchar('\n');
}
"#,
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "5:e3818200ff\n5:73636f7065\n0:\n"
    );
}

#[test]
fn executes_string_operators_and_byte_wise_equality() {
    let output = compile_and_run(
        r#"main :: Unit -> Int32 := \() {
  value := "あ\0\xff";
  ok := (#value == 5u64) &&
        (value # 0u64 == 227u8) &&
        (value # 4u64 == 255u8) &&
        (value == "\xe3\x81\x82\x00\xff") &&
        (value != "あ\0\xfe") &&
        ("" == "");
  if (ok) then { 0 } else { 1 };
};"#,
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn traps_out_of_range_string_byte_access() {
    for expression in [r#""" # 0u64"#, r#""a" # 1u64"#] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("string index out of range"),
            "expression: {expression}"
        );
    }
}

#[test]
fn copies_host_string_results_into_program_lifetime_storage() {
    let generated = emit(
        r#"extern fetch :: Unit -> String;
main :: Unit -> Int32 := \() {
  value := extern fetch();
  ok := (value == "host\0\xff") && (value # 5u64 == 255u8);
  if (ok) then { 0 } else { 1 };
};"#,
    )
    .expect("emit String ABI");
    assert!(generated.header.contains(
        "MalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);"
    ));
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalString mal_ext_fetch(MalContext *context);"
    ));
    let fixture = NativeFixture::new("string-copy");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"
#include <stdlib.h>
#include <string.h>

MalString mal_ext_fetch(MalContext *context) {
    uint8_t *scratch = (uint8_t *)malloc(6);
    if (scratch == NULL) {
        mal_trap(context, "host allocation failed");
    }
    const uint8_t original[6] = { 'h', 'o', 's', 't', 0, 255 };
    memcpy(scratch, original, 6);
    MalString result = mal_string_copy(context, scratch, UINT64_C(6));
    memset(scratch, 0, 6);
    free(scratch);
    return result;
}
"#,
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn traps_string_copy_allocation_failure_and_length_overflow() {
    let source =
        "extern fetch :: Unit -> String; main :: Unit -> Int32 := \\() { extern fetch(); 0; };";

    let failure_fixture = NativeFixture::new("string-copy-failure");
    let failure_executable = failure_fixture.compile_generated_with_options(
        emit(source).expect("emit String ABI"),
        r#"#include "program.mal.h"
MalString mal_ext_fetch(MalContext *context) {
    const uint8_t value = 1;
    return mal_string_copy(context, &value, UINT64_C(1));
}
"#,
        &["-DMAL_TEST_FORCE_ALLOCATION_FAILURE"],
    );
    let failure = failure_fixture.run(failure_executable);
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("mal trap: allocation failed"));

    let overflow_fixture = NativeFixture::new("string-copy-overflow");
    let overflow_executable = overflow_fixture.compile_generated(
        emit(source).expect("emit String ABI"),
        r#"#include "program.mal.h"
MalString mal_ext_fetch(MalContext *context) {
    const uint8_t value = 1;
    return mal_string_copy(context, &value, UINT64_MAX);
}
"#,
    );
    let overflow = overflow_fixture.run(overflow_executable);
    assert!(!overflow.status.success());
    assert!(
        String::from_utf8_lossy(&overflow.stderr).contains("mal trap: allocation size overflow")
    );
}

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
        "int8_t mal_ext_i8(MalContext *context, int8_t value);",
        "int16_t mal_ext_i16(MalContext *context, int16_t value);",
        "int32_t mal_ext_i32(MalContext *context, int32_t value);",
        "int64_t mal_ext_i64(MalContext *context, int64_t value);",
        "uint8_t mal_ext_u8(MalContext *context, uint8_t value);",
        "uint16_t mal_ext_u16(MalContext *context, uint16_t value);",
        "uint32_t mal_ext_u32(MalContext *context, uint32_t value);",
        "uint64_t mal_ext_u64(MalContext *context, uint64_t value);",
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
           p0 := offset(base, 1u64); storeInt8(p0, -8i8);\n\
           p1 := offset(base, 3u64); storeInt16(p1, -16i16);\n\
           p2 := offset(base, 6u64); storeInt32(p2, -32i32);\n\
           p3 := offset(base, 11u64); storeInt64(p3, -64i64);\n\
           p4 := offset(base, 20u64); storeUInt8(p4, 8u8);\n\
           p5 := offset(base, 22u64); storeUInt16(p5, 16u16);\n\
           p6 := offset(base, 25u64); storeUInt32(p6, 32u32);\n\
           p7 := offset(base, 30u64); storeUInt64(p7, 64u64);\n\
           p8 := offset(base, 39u64); storeFloat32(p8, 1.5f32);\n\
           p9 := offset(base, 44u64); storeFloat64(p9, -2.5f64);\n\
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
            .contains("typedef struct { uint8_t *address; } MalPtr;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalPtr mal_ext_memory(MalContext *context);"
    ));
    let host = r#"#include "program.mal.h"

MalPtr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[52];
    (void)context;
    return (MalPtr){ .address = bytes };
}
"#;
    let fixture = NativeFixture::new("ptr-memory");
    let executable = fixture.compile_generated(generated, host);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn executes_unaligned_ptr_value_access() {
    let source = "extern pointerSlot :: Unit -> Ptr;\n\
         extern target :: Unit -> Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
           slot := offset(extern pointerSlot(), 1u64);\n\
           storePtr(slot, extern target());\n\
           stored := loadPtr(slot);\n\
           storeInt32(stored, 42i32);\n\
           loadInt32(extern target()) - 42;\n\
         };";
    let generated = emit(source).expect("emit Ptr value access");
    assert!(generated.source.contains("mal_load_ptr"));
    assert!(generated.source.contains("mal_store_ptr"));
    let host = r#"#include "program.mal.h"

MalPtr mal_ext_pointerSlot(MalContext *context) {
    static uint8_t bytes[sizeof(MalPtr) + 1];
    (void)context;
    return mal_ptr_from_address(bytes);
}

MalPtr mal_ext_target(MalContext *context) {
    static uint8_t bytes[sizeof(int32_t)];
    (void)context;
    return mal_ptr_from_address(bytes);
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
             + @Float32 + @Float64 + pointerSize + @String;\n\
           if (actual == extern expectedSize()) then { 0 } else { 1 };\n\
         };",
        r#"#include "program.mal.h"

uint64_t mal_ext_expectedSize(MalContext *context) {
    (void)context;
    return UINT64_C(50) + (uint64_t)(sizeof(MalPtr) * 2);
}
"#,
    );
    assert!(output.status.success());
}

#[test]
fn executes_unaligned_string_descriptor_access() {
    let source = "extern stringSlot :: Unit -> Ptr;\n\
         extern inspectStringSlot :: Unit -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           slot := offset(extern stringSlot(), 1u64);\n\
           initial := loadString(slot);\n\
           storeString(slot, \"held\\0\\xff\");\n\
           extern inspectStringSlot();\n\
           stored := loadString(slot);\n\
           if ((initial == \"seed\") && (stored == \"held\\0\\xff\") && (stored # 5u64 == 255u8)) then {\n\
             0\n\
           } else {\n\
             1\n\
           };\n\
         };";
    let generated = emit(source).expect("emit String descriptor access");
    assert!(generated.source.contains("mal_load_string"));
    assert!(generated.source.contains("mal_store_string"));
    let host = r#"#include "program.mal.h"
#include <string.h>

static uint8_t slot[sizeof(MalPtr) + sizeof(uint64_t) + 1];

MalPtr mal_ext_stringSlot(MalContext *context) {
    static const uint8_t seed[] = { 's', 'e', 'e', 'd' };
    MalString initial = mal_string_copy(context, seed, UINT64_C(4));
    MalPtr data = mal_ptr_from_address((uint8_t *)initial.data);
    uint64_t length = initial.length;
    memcpy(slot + 1, &data, sizeof(data));
    memcpy(slot + 1 + sizeof(data), &length, sizeof(length));
    return mal_ptr_from_address(slot);
}

void mal_ext_inspectStringSlot(MalContext *context) {
    MalPtr data;
    uint64_t length;
    static const uint8_t expected[] = { 'h', 'e', 'l', 'd', 0, 255 };
    memcpy(&data, slot + 1, sizeof(data));
    memcpy(&length, slot + 1 + sizeof(data), sizeof(length));
    if (length != UINT64_C(6)
        || memcmp(mal_ptr_address(data), expected, sizeof(expected)) != 0) {
        mal_trap(context, "unexpected stored String descriptor");
    }
}
"#;
    let fixture = NativeFixture::new("string-value-memory");
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
    assert!(!generated.source.contains("mal_load_string"));
    assert!(!generated.source.contains("mal_store_string"));

    let fixture = NativeFixture::new("selective-memory-runtime");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MalPtr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[8];
    (void)context;
    return (MalPtr){ .address = bytes };
}
"#,
        &["-O2"],
    );
    assert!(fixture.run(executable).status.success());
}

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
    assert!(!generated.header.contains("MalType_Request"));
    assert!(
        generated
            .header
            .contains("typedef MalSum_2 MalType_Response;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Response mal_ext_exchange(MalContext *context, int32_t argument_0, \
         MalProduct_0 argument_1);"
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
            .contains("MalContext *context MAL_MAYBE_UNUSED, \\")
    );
    assert!(generated.header.contains("MalProduct_0 argument_1 \\"));
    assert!(
        generated
            .header
            .contains("struct MalProduct_0 {\n    uint8_t field_0;\n    int32_t field_1;\n};")
    );
    assert!(generated.header.contains("MalProduct_1 variant_1;"));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_exchange(context, argument_0, argument_1) {
    return mal_make_Response_1(argument_0, argument_1.field_1);
}

"#;
    let fixture = NativeFixture::new("aggregate-abi");
    let executable = fixture.compile_generated(generated.clone(), host);
    assert!(fixture.run(executable).status.success());

    let invalid_host = r#"#include "program.mal.h"

MalType_Response mal_ext_exchange(
    MalContext *context,
    int32_t argument_0,
    MalProduct_0 argument_1
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

    assert!(generated.header.contains("typedef uint64_t MalType_Count;"));
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
            .contains("typedef uint64_t MalType_FirstCount;")
    );
    assert!(
        generated
            .header
            .contains("typedef uint64_t MalType_SecondCount;")
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
    assert!(!generated.header.contains("MalType_Request"));
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
        "static inline MalType_Choice mal_make_Choice_1(int32_t value_0, int32_t value_1)"
    ));
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "static inline int32_t mal_get_Choice_1_0(MalContext *context, MalType_Choice value)"
    ));

    let host = r#"#include "program.mal.h"

MAL_DEFINE_inspect(context, value) {
    if (!mal_is_Choice_1(value)) {
        mal_trap(context, "expected pair");
    }
    return mal_get_Choice_1_0(context, value) +
           mal_get_Choice_1_1(context, value);
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
            .contains("typedef struct { uintptr_t bits; } MalOpaque_Mem;")
    );
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "uint64_t mal_ext_combinedLength(MalContext *context, \
         MalOpaque_Mem argument_0, MalOpaque_Mem argument_1);"
    ));
    let host = r#"#include "program.mal.h"

MalOpaque_Mem mal_ext_allocate(MalContext *context, uint64_t value) {
    (void)context;
    return (MalOpaque_Mem){ .bits = (uintptr_t)value };
}

uint64_t mal_ext_combinedLength(
    MalContext *context,
    MalOpaque_Mem first,
    MalOpaque_Mem second
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

MalSum_1 mal_ext_choose(MalContext *context) {
    (void)context;
    return (MalSum_1){
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

#[test]
fn traps_when_a_closure_environment_cannot_be_allocated() {
    let generated = emit(
        "makeClosure :: Int32 -> (Unit -> Int32) := \\(value :: Int32) {\n\
           \\<value>() { value; };\n\
         };\n\
         main :: Unit -> Int32 := \\() { makeClosure(7)(); };",
    )
    .expect("emit C");
    let fixture = NativeFixture::new("allocation-failure");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_FORCE_ALLOCATION_FAILURE"],
    );
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("mal trap: allocation failed"));
}

#[test]
fn preserves_short_circuit_and_eager_bool_equality_order() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         marked :: Int32 -> Bool := \\(value :: Int32) {\n\
           extern printInt32(value);\n\
           value == 1;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           false && marked(2);\n\
           true || marked(3);\n\
           marked(4) == marked(5);\n\
           0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "4\n5\n");
}

#[test]
fn emits_direct_comparison_conditions_without_tagged_bool_values() {
    let generated = emit(
        "choose :: Int32 -> Int32 := \\(value :: Int32) {\n\
           if (value < 10) then { 42 } else { value };\n\
         };\n\
         main :: Unit -> Int32 := \\() { choose(9) - 42; };",
    )
    .expect("emit primitive branch");

    assert!(!generated.source.contains("switch ("));
    assert!(!generated.source.contains(".tag ="));
    assert!(generated.source.contains("if (mal_value_source_"));
    let fixture = NativeFixture::new("primitive-branch");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn implements_wrapping_int32_arithmetic_without_signed_overflow() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(2147483647 + 1);\n\
           extern printInt32(-2147483648 * -1);\n\
           0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "-2147483648\n-2147483648\n"
    );
}

#[test]
fn executes_all_fixed_width_integer_operator_families() {
    let output = compile_and_run(
        "main :: Unit -> Int32 := \\() {\n\
           ok :: Bool :=\n\
             (-128i8 - 1i8 == 127i8) &&\n\
             (32767i16 * 2i16 == -2i16) &&\n\
             (2147483647i32 + 1i32 == -2147483648i32) &&\n\
             (9223372036854775807i64 + 1i64 == -9223372036854775808i64) &&\n\
             (-1u8 == 255u8) &&\n\
             ((65535u16 & 255u16) == 255u16) &&\n\
             (1u32 << 31u32 == 2147483648u32) &&\
             (18446744073709551615u64 + 1u64 == 0u64) &&\n\
             (-2i32 >> 1i32 == -1i32) &&\n\
             (-9223372036854775808i64 / 1i64 == -9223372036854775808i64);\n\
           if (ok) then { 0 } else { 1 };\n\
         };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn executes_modulo_integer_conversions() {
    let output = compile_and_run(
        "main :: Unit -> Int32 := \\() {\n\
           ok :: Bool :=\n\
             (UInt8(-1i8) == 255u8) &&\n\
             (Int8(255u16) == -1i8) &&\n\
             (UInt16(-1i8) == 65535u16) &&\
             (Int16(255u8) == 255i16) &&\n\
             (UInt32(-1i8) == 4294967295u32) &&\n\
             (Int32(4294967295u32) == -1i32) &&\n\
             (Int64(18446744073709551615u64) == -1i64) &&\n\
             (UInt64(-1i8) == 18446744073709551615u64);\n\
           if (ok) then { 0 } else { 1 };\n\
         };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn executes_nested_products_destructuring_and_multiple_arguments() {
    let output = compile_and_run(
        "extern mark :: Int32 -> Int32;\n\
         pair :: (Int32, Int32) := (20i32, 22i32);\n\
         add :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           (first, second) := pair;\n\
           nested := ((extern mark(first), ()), extern mark(second));\n\
           ((value, _), extra) := nested;\n\
           add(value, extra) - 42i32;\n\
         };",
        r#"#include "program.mal.h"
#include <stdio.h>

int32_t mal_ext_mark(MalContext *context, int32_t value) {
    (void)context;
    printf("%d\n", value);
    return value;
}
"#,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "20\n22\n");
}

#[test]
fn executes_a_product_captured_by_an_escaping_closure() {
    let output = compile_and_run(
        "make :: Unit -> (Unit -> Int32) := \\() {\n\
           pair := (20i32, 22i32);\n\
           \\<pair>() {\n\
             (left, right) := pair;\n\
             left + right;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { make()() - 42i32; };",
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn traps_out_of_range_shift_counts() {
    for expression in [
        "1i8 << 8i8",
        "1i16 << 16i16",
        "1i32 << 32i32",
        "1i64 << 64i64",
        "1u8 << 8u8",
        "1u16 << 16u16",
        "1u32 << 32u32",
        "1u64 << 64u64",
        "1i8 >> -1i8",
        "1i16 >> -1i16",
        "1i32 >> -1i32",
        "1i64 >> -1i64",
    ] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("shift count out of range"),
            "expression: {expression}"
        );
    }
}

#[test]
fn executes_sum_injection_and_case() {
    let output = compile_and_run(
        "Maybe :: [Unit, Int32];\n\
         extern printInt32 :: Int32 -> Unit;\n\
         get :: Maybe -> Int32 := \\(value :: Maybe) {\n\
           case (value)\n\
             [0](_) { extern printInt32(100); 0 }\n\
             [1](item) { doubled := item + item; extern printInt32(doubled); doubled };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(get(Maybe[1](9)));\n\
           0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "18\n18\n");
}

#[test]
fn traps_invalid_division_and_remainder_at_every_width() {
    let mut cases = Vec::new();
    for (suffix, minimum) in [
        ("i8", Some("-128")),
        ("i16", Some("-32768")),
        ("i32", Some("-2147483648")),
        ("i64", Some("-9223372036854775808")),
        ("u8", None),
        ("u16", None),
        ("u32", None),
        ("u64", None),
    ] {
        cases.push((format!("1{suffix} / 0{suffix}"), "division by zero"));
        cases.push((format!("1{suffix} % 0{suffix}"), "remainder by zero"));
        if let Some(minimum) = minimum {
            cases.push((
                format!("{minimum}{suffix} / -1{suffix}"),
                "signed division overflow",
            ));
            cases.push((
                format!("{minimum}{suffix} % -1{suffix}"),
                "signed remainder overflow",
            ));
        }
    }
    for (expression, message) in cases {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "expression: {expression}"
        );
    }
}

#[test]
fn rejects_invalid_executable_programs() {
    assert!(
        emit("value :: Int32 := 1i32;")
            .unwrap_err()
            .message
            .contains("has no")
    );
    assert!(
        emit("main :: Int32 -> Int32 := \\(value :: Int32) { value; };")
            .unwrap_err()
            .message
            .contains("wrong type")
    );
}
