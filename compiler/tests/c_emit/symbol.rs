use super::*;

#[test]
fn emits_static_symbol_bytes_that_survive_closure_escape() {
    let output = compile_and_run(
        r#"extern inspect :: Symbol -> Unit;
make :: Symbol -> (Unit -> Symbol) := \(value :: Symbol) {
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

void mal_ext_inspect(MalContext *context, MalType_Symbol value) {
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
fn executes_symbol_operators_and_byte_wise_equality() {
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
fn concatenates_symbols_as_immutable_bytes() {
    let output = compile_and_run(
        r#"main :: Unit -> Int32 := \() {
  joined := "あ\0" + "\xffz";
  ok := (joined == "\xe3\x81\x82\x00\xffz") &&
        (#joined == 6u64) &&
        (("" + joined) == joined) &&
        ((joined + "") == joined);
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
fn traps_symbol_concatenation_allocation_failure() {
    let fixture = NativeFixture::new("symbol-concatenation-failure");
    let executable = fixture.compile_generated_with_options(
        emit(r#"main :: Unit -> Int32 := \() { "left" + "right"; 0; };"#)
            .expect("emit Symbol concatenation"),
        "",
        &["-DMAL_TEST_FORCE_ALLOCATION_FAILURE"],
    );
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("mal trap: allocation failed"));
}

#[test]
fn traps_out_of_range_symbol_byte_access() {
    for expression in [r#""" # 0u64"#, r#""a" # 1u64"#] {
        let output = compile_and_run(
            &format!("main :: Unit -> Int32 := \\() {{ {expression}; 0; }};"),
            "",
        );
        assert!(!output.status.success(), "expression: {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Symbol index out of range"),
            "expression: {expression}"
        );
    }
}

#[test]
fn admits_host_bytes_as_symbols() {
    let generated = emit(
        r#"extern fetch :: Unit -> Symbol;
main :: Unit -> Int32 := \() {
  value := extern fetch();
  ok := (value == "host\0\xff") && (value # 5u64 == 255u8);
  if (ok) then { 0 } else { 1 };
};"#,
    )
    .expect("emit Symbol ABI");
    assert!(generated.header.contains(
        "MalType_Symbol mal_Symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);"
    ));
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Symbol mal_ext_fetch(MalContext *context);"
    ));
    let fixture = NativeFixture::new("symbol-copy");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"
#include <stdlib.h>
#include <string.h>

MalType_Symbol mal_ext_fetch(MalContext *context) {
    uint8_t *scratch = (uint8_t *)malloc(6);
    if (scratch == NULL) {
        mal_trap(context, "host allocation failed");
    }
    const uint8_t original[6] = { 'h', 'o', 's', 't', 0, 255 };
    memcpy(scratch, original, 6);
    MalType_Symbol result = mal_Symbol_copy_from_bytes(context, scratch, UINT64_C(6));
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
fn traps_symbol_copy_allocation_failure_and_length_overflow() {
    let source =
        "extern fetch :: Unit -> Symbol; main :: Unit -> Int32 := \\() { extern fetch(); 0; };";

    let failure_fixture = NativeFixture::new("symbol-copy-failure");
    let failure_executable = failure_fixture.compile_generated_with_options(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    const uint8_t value = 1;
    return mal_Symbol_copy_from_bytes(context, &value, UINT64_C(1));
}
"#,
        &["-DMAL_TEST_FORCE_ALLOCATION_FAILURE"],
    );
    let failure = failure_fixture.run(failure_executable);
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("mal trap: allocation failed"));

    let overflow_fixture = NativeFixture::new("symbol-copy-overflow");
    let overflow_executable = overflow_fixture.compile_generated(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    const uint8_t value = 1;
    return mal_Symbol_copy_from_bytes(context, &value, UINT64_MAX);
}
"#,
    );
    let overflow = overflow_fixture.run(overflow_executable);
    assert!(!overflow.status.success());
    assert!(
        String::from_utf8_lossy(&overflow.stderr).contains("mal trap: allocation size overflow")
    );
}
