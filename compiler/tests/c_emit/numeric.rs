use super::*;

#[test]
fn emits_uint64_literals_and_scalar_extern_abi() {
    let output = compile_and_run(
        "extern printUInt64 :: UInt64 -> Unit;\n\
         capture :: UInt64 -> (Unit -> UInt64) := \\(value :: UInt64) {\n\
           \\() { value; };\n\
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
        "MalType_Int32 mal_ext_inspect(MalContext *context, MalType_Float32 argument_0, \
         MalType_Float64 argument_1);"
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
