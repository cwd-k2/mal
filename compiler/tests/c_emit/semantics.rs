use super::*;

#[test]
fn traps_when_a_closure_environment_cannot_be_allocated() {
    let generated = emit(
        "makeClosure :: Int32 -> (Unit -> Int32) := \\(value :: Int32) {\n\
           \\() { value; };\n\
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
           \\() {\n\
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
fn executes_valid_shift_counts_at_every_width() {
    for expression in [
        "1i8 << 7i8",
        "1i16 << 15i16",
        "1i32 << 31i32",
        "1i64 << 63i64",
        "1u8 << 7u8",
        "1u16 << 15u16",
        "1u32 << 31u32",
        "1u64 << 63u64",
        "-1i8 >> 7i8",
        "-1i16 >> 15i16",
        "-1i32 >> 31i32",
        "-1i64 >> 63i64",
    ] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(
            output.status.success(),
            "expression: {expression}: {}",
            String::from_utf8_lossy(&output.stderr)
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
fn executes_valid_division_and_remainder_at_every_width() {
    let mut expressions = Vec::new();
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
        expressions.push(format!("7{suffix} / 2{suffix}"));
        expressions.push(format!("7{suffix} % 2{suffix}"));
        if let Some(minimum) = minimum {
            expressions.push(format!("{minimum}{suffix} / 1{suffix}"));
            expressions.push(format!("{minimum}{suffix} % 1{suffix}"));
        }
    }
    for expression in expressions {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(
            output.status.success(),
            "expression: {expression}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn omits_numeric_precondition_traps_from_generated_c() {
    let generated = emit(
        "main :: Unit -> Int32 := \\() {\n\
           1u64 << 1u64;\n\
           4i64 / 2i64;\n\
           5i64 % 2i64;\n\
           Int64(1.0f64);\n\
           0;\n\
         };",
    )
    .expect("emit numeric operations");

    for obsolete in [
        "shift count out of range",
        "division by zero",
        "remainder by zero",
        "float-to-integer conversion out of range",
    ] {
        assert!(!generated.source.contains(obsolete), "{obsolete}");
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

#[test]
fn admits_process_arguments_as_symbols() {
    let generated = emit(
        "Arguments :: (UInt64, Ptr);\n\
         argumentAt :: (Ptr, UInt64) -> Symbol := \\(arguments :: Ptr, index :: UInt64) {\n\
           slot := arguments + index * (@Ptr + @UInt64);\n\
           loadSymbol(loadPtr(slot), loadUInt64(slot + @Ptr));\n\
         };\n\
         main :: Arguments -> Int32 := \\(count :: UInt64, arguments :: Ptr) {\n\
           first := argumentAt(arguments, 0u64);\n\
           second := argumentAt(arguments, 1u64);\n\
           if (count == 2u64 && first == \"alpha\" && second == \"\")\n\
             then { 0 }\n\
             else { 1 };\n\
         };",
    )
    .expect("emit an argument-aware entry point");
    assert!(
        generated
            .source
            .contains("int main(int mal_argc, char **mal_argv)")
    );

    let fixture = NativeFixture::new("process-arguments");
    let executable = fixture.compile_generated(generated, "");
    let output = std::process::Command::new(executable)
        .args(["alpha", ""])
        .output()
        .expect("run argument-aware executable");
    assert!(output.status.success());
}
