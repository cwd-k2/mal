use super::*;

#[test]
fn emits_static_symbol_bytes_that_survive_closure_escape() {
    let output = compile_and_run(
        r#"extern inspect :: Symbol -> Unit;
make :: Symbol -> (Unit -> Symbol) := \(value :: Symbol) {
  \() { value; };
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
fn releases_function_local_symbols_before_the_next_call() {
    let generated = emit(
        r#"measure :: Symbol -> UInt64 := \(suffix :: Symbol) {
  value := "prefix" + suffix;
  #value;
};
main :: Unit -> Int32 := \() {
  measure("a");
  measure("b");
  measure("c");
  0;
};"#,
    )
    .expect("emit owned Symbol cleanup");
    let fixture = NativeFixture::new("symbol-local-ownership");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_LIVE_ALLOCATION_LIMIT=1",
            "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ],
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn retains_returned_and_captured_symbols_until_their_owners_are_destroyed() {
    let generated = emit(
        r#"make :: Symbol -> Symbol := \(suffix :: Symbol) {
  "prefix" + suffix;
};
hold :: Symbol -> (Unit -> Symbol) := \(suffix :: Symbol) {
  value := make(suffix);
  \() { value; };
};
main :: Unit -> Int32 := \() {
  first := make("a");
  alias := first;
  pair := (first, alias);
  (left, right) := pair;
  held := hold(left);
  returned := held();
  ok := (left == "prefixa") &&
        (right == "prefixa") &&
        (returned == "prefixprefixa");
  if (ok) then { 0 } else { 1 };
};"#,
    )
    .expect("emit shared and captured Symbol ownership");
    let fixture = NativeFixture::new("symbol-shared-ownership");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
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
fn transfers_only_last_owned_uses_through_aliases_branches_and_tail_edges() {
    let generated = emit(
        r#"extern inspect :: Symbol -> Unit;
keepAlias :: Symbol -> Symbol := \(suffix :: Symbol) {
  owned := "prefix" + suffix;
  alias := owned;
  extern inspect(owned);
  alias;
};
keepShared :: Symbol -> (Symbol, Symbol) := \(suffix :: Symbol) {
  owned := "shared" + suffix;
  alias := owned;
  extended := owned + "x";
  (alias, extended);
};
duplicate :: Symbol -> Symbol := \(value :: Symbol) { value + value };
choose :: (Bool, Symbol) -> Symbol := \(condition :: Bool, suffix :: Symbol) {
  owned := "branch" + suffix;
  if (condition)
  then { selected := owned; selected }
  else { selected := owned; selected };
};
grow :: (Symbol, Int64) -> Symbol := \(value :: Symbol, remaining :: Int64) {
  if (remaining == 0)
  then { value }
  else { grow(value + "x", remaining - 1) };
};
main :: Unit -> Int32 := \() {
  alias := keepAlias("a");
  (shared, extended) := keepShared("b");
  duplicated := duplicate("c");
  selected := choose(true, "b");
  grown := grow("", 3i64);
  if ((alias == "prefixa") &&
      (shared == "sharedb") &&
      (extended == "sharedbx") &&
      (duplicated == "cc") &&
      (selected == "branchb") &&
      (grown == "xxx"))
  then { 0 }
  else { 1 };
};"#,
    )
    .expect("emit last-use ownership transfers");

    let alias = generated_function(&generated.source, "keepAlias");
    assert_eq!(alias.matches("mal_symbol_retain(").count(), 1);
    let shared = generated_function(&generated.source, "keepShared");
    assert!(shared.contains("mal_symbol_concatenate_consuming_left("));
    let duplicated = generated_function(&generated.source, "duplicate");
    assert!(duplicated.contains("mal_symbol_concatenate("));
    assert!(!duplicated.contains("mal_symbol_concatenate_consuming_left("));
    let branch = generated_function(&generated.source, "choose");
    assert_eq!(branch.matches("mal_symbol_retain(").count(), 1);
    let tail = generated_function(&generated.source, "grow");
    assert!(!tail.contains("mal_symbol_retain("));
    assert_eq!(tail.matches("mal_copy_value_").count(), 1);
    assert!(tail.contains("mal_symbol_concatenate_consuming_left("));

    let fixture = NativeFixture::new("last-use-ownership-transfer");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"
#include <string.h>

void mal_ext_inspect(MalContext *context, MalType_Symbol value) {
    static const uint8_t expected[] = "prefixa";
    if (value.length != UINT64_C(7)
        || memcmp(value.data, expected, sizeof(expected) - 1) != 0) {
        mal_trap(context, "last-use transfer changed a live alias");
    }
}
"#,
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn retains_only_the_active_managed_sum_payload() {
    let generated = emit(
        r#"Choice :: [Unit, Symbol];
make :: Symbol -> Choice := \(suffix :: Symbol) {
  Choice[1]("prefix" + suffix);
};
read :: Choice -> Symbol := \(choice :: Choice) {
  case (choice)
    [0](_) { "empty" }
    [1](value) { value };
};
main :: Unit -> Int32 := \() {
  choice := make("a");
  value := read(choice);
  if (value == "prefixa") then { 0 } else { 1 };
};"#,
    )
    .expect("emit managed sum ownership");
    let fixture = NativeFixture::new("symbol-sum-ownership");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
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
fn destroys_discarded_and_destructured_managed_branch_results() {
    let generated = emit(
        r#"main :: Unit -> Int32 := \() {
  if (true) then { "a" + "b" } else { "c" + "d" };
  (left, right) := if (true)
    then { ("e" + "f", "g" + "h") }
    else { ("i" + "j", "k" + "l") };
  if ((left == "ef") && (right == "gh")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit managed branch result cleanup");
    let fixture = NativeFixture::new("symbol-branch-ownership");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
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
fn grows_a_consumed_symbol_with_bounded_allocation_operations() {
    let generated = emit(
        r#"grow :: (Symbol, Int64) -> Symbol := \(value :: Symbol, remaining :: Int64) {
  if (remaining == 0)
  then { value }
  else { grow(value + "x", remaining - 1) };
};
main :: Unit -> Int32 := \() {
  value := grow("", 10000i64);
  if (#value == 10000u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit amortized Symbol growth");
    let fixture = NativeFixture::new("bounded-symbol-growth-allocations");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=32",
            "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ],
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn traps_when_a_consumed_symbol_cannot_be_reallocated() {
    let generated = emit(
        r#"appendTwice :: Symbol -> Symbol := \(value :: Symbol) {
  first := value + "b";
  first + "c";
};
main :: Unit -> Int32 := \() { appendTwice("a"); 0; };"#,
    )
    .expect("emit consuming Symbol reallocation");
    let fixture = NativeFixture::new("symbol-reallocation-failure");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_FORCE_REALLOCATION_FAILURE"],
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

fn generated_function<'a>(source: &'a str, binding: &str) -> &'a str {
    let marker = format!("/* mal source binding: {binding} */");
    let start = source
        .rfind(&marker)
        .unwrap_or_else(|| panic!("missing generated function for {binding}"));
    let remainder = &source[start + marker.len()..];
    let end = remainder
        .find("/* mal source binding:")
        .unwrap_or(remainder.len());
    &remainder[..end]
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
