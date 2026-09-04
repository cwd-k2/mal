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

fn compile_and_run(source: &str, host: &str) -> std::process::Output {
    let generated = emit(source).expect("emit C");
    let fixture = NativeFixture::new("c-emit");
    let executable = fixture.compile_generated(generated, host);
    fixture.run(executable)
}

const PRINT_HOST: &str = r#"#include "program.mal.h"
#include <stdio.h>

void mal_ext_printInt32(MalContext *context, int32_t value) {
    (void)context;
    printf("%d\n", value);
}
"#;

#[test]
fn emits_the_m0_host_abi_and_executes_the_host_example() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn executes_escaping_capturing_closures() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         makeAdder :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           return \\<x>(y :: Int32) { return x + y; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           addTen := makeAdder(10);\n\
           extern printInt32(addTen(5));\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "15\n");
}

#[test]
fn emits_uint64_literals_and_scalar_extern_abi() {
    let output = compile_and_run(
        "extern printUInt64 :: UInt64 -> Unit;\n\
         capture :: UInt64 -> (Unit -> UInt64) := \\(value :: UInt64) {\n\
           return \\<value>() { return value; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printUInt64(capture(18446744073709551615UInt64)());\n\
           return 0;\n\
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
         main :: Unit -> Int32 := \\() { return 0; };",
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
        assert!(generated.header.contains(declaration), "{declaration}");
    }
}

#[test]
fn traps_when_a_closure_environment_cannot_be_allocated() {
    let generated = emit(
        "makeClosure :: Int32 -> (Unit -> Int32) := \\(value :: Int32) {\n\
           return \\<value>() { return value; };\n\
         };\n\
         main :: Unit -> Int32 := \\() { return makeClosure(7)(); };",
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
           return value == 1;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           false && marked(2);\n\
           true || marked(3);\n\
           marked(4) == marked(5);\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "4\n5\n");
}

#[test]
fn implements_wrapping_int32_arithmetic_without_signed_overflow() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(2147483647 + 1);\n\
           extern printInt32(-2147483648 * -1);\n\
           return 0;\n\
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
             (-128Int8 - 1Int8 == 127Int8) &&\n\
             (32767Int16 * 2Int16 == -2Int16) &&\n\
             (-2Int32 >> 1Int32 == -1Int32) &&\n\
             (-9223372036854775808Int64 / 1Int64 == -9223372036854775808Int64) &&\n\
             (~0UInt8 == 255UInt8) &&\n\
             ((65535UInt16 & 255UInt16) == 255UInt16) &&\n\
             (1UInt32 << 31UInt32 == 2147483648UInt32) &&\
             (18446744073709551615UInt64 + 1UInt64 == 0UInt64);\n\
           return if (ok) then { 0 } else { 1 };\n\
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
             (UInt8(-1Int8) == 255UInt8) &&\n\
             (Int8(255UInt16) == -1Int8) &&\n\
             (UInt16(-1Int8) == 65535UInt16) &&\
             (Int16(255UInt8) == 255Int16) &&\n\
             (UInt64(-1Int8) == 18446744073709551615UInt64);\n\
           return if (ok) then { 0 } else { 1 };\n\
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
fn traps_out_of_range_shift_counts() {
    for expression in [
        "1Int8 << 8Int8",
        "1Int16 << 16Int16",
        "1Int32 << 32Int32",
        "1Int64 << 64Int64",
        "1UInt8 << 8UInt8",
        "1UInt16 << 16UInt16",
        "1UInt32 << 32UInt32",
        "1UInt64 << 64UInt64",
        "1Int8 >> -1Int8",
        "1Int16 >> -1Int16",
        "1Int32 >> -1Int32",
        "1Int64 >> -1Int64",
    ] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; return 0; }};"),
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
           return case value { [0](_) => 0; [1](item) => item; };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(get(Maybe[1](9)));\n\
           return 0;\n\
         };",
        PRINT_HOST,
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "9\n");
}

#[test]
fn traps_invalid_division_and_remainder_at_every_width() {
    let mut cases = Vec::new();
    for (ty, minimum) in [
        ("Int8", Some("-128")),
        ("Int16", Some("-32768")),
        ("Int32", Some("-2147483648")),
        ("Int64", Some("-9223372036854775808")),
        ("UInt8", None),
        ("UInt16", None),
        ("UInt32", None),
        ("UInt64", None),
    ] {
        cases.push((format!("1{ty} / 0{ty}"), "division by zero"));
        cases.push((format!("1{ty} % 0{ty}"), "remainder by zero"));
        if let Some(minimum) = minimum {
            cases.push((
                format!("{minimum}{ty} / -1{ty}"),
                "signed division overflow",
            ));
            cases.push((
                format!("{minimum}{ty} % -1{ty}"),
                "signed remainder overflow",
            ));
        }
    }
    for (expression, message) in cases {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; return 0; }};"),
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
fn rejects_programs_outside_the_m0_c_boundary() {
    assert!(
        emit(
            "extern choose :: [Unit, Unit] -> Int32;\n\
         main :: Unit -> Int32 := \\() { return 0; };"
        )
        .unwrap_err()
        .message
        .contains("outside the M0 C ABI")
    );
    assert!(
        emit("value :: Int32 := 1Int32;")
            .unwrap_err()
            .message
            .contains("has no")
    );
    assert!(
        emit("main :: Int32 -> Int32 := \\(value :: Int32) { return value; };")
            .unwrap_err()
            .message
            .contains("wrong type")
    );
}
