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
        &[
            "-DMAL_TEST_VALIDATE_SYMBOLS",
            "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ],
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
    assert!(!branch.contains("mal_symbol_retain("));
    let tail = generated_function(&generated.source, "grow");
    assert_eq!(tail.matches("mal_symbol_retain(").count(), 1);
    assert!(!tail.contains("mal_copy_value_"));
    assert!(tail.contains("mal_tail_next_parameter_0"));
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
fn prepends_into_a_consumed_flat_symbol_with_bounded_allocations() {
    let generated = emit(
        r#"grow :: (Int64, Symbol) -> Symbol := \(remaining :: Int64, value :: Symbol) {
  if (remaining == 0)
  then { value }
  else { grow(remaining - 1, "x" + value) };
};
main :: Unit -> Int32 := \() {
  value := grow(5000i64, "");
  ok := (#value == 5000u64) &&
        (value # 0u64 == 'x') &&
        (value # 4999u64 == 'x');
  if (ok) then { 0 } else { 1 };
};"#,
    )
    .expect("emit amortized Symbol prepend");
    assert!(
        generated
            .source
            .contains("mal_symbol_concatenate_consuming_right")
    );

    let fixture = NativeFixture::new("right-growing-symbol-rope");
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
fn transfers_symbol_arguments_through_known_helper_calls() {
    let generated = emit(
        r#"inner :: Symbol -> Symbol := \(value :: Symbol) {
  "x" + value;
};
outer :: Symbol -> Symbol := \(value :: Symbol) {
  inner(value);
};
grow :: (Int64, Symbol) -> Symbol := \(remaining :: Int64, value :: Symbol) {
  if (remaining == 0)
  then { value }
  else { grow(remaining - 1, outer(value)) };
};
main :: Unit -> Int32 := \() {
  value := grow(10000i64, "");
  if (#value == 10000u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit owned direct Symbol calls");
    let inner = generated_function(&generated.source, "inner");
    assert!(inner.contains("mal_owned_function_0("));
    assert!(inner.contains("mal_symbol_concatenate_consuming_right("));
    let outer = generated_function(&generated.source, "outer");
    assert!(outer.contains("mal_owned_function_1("));
    assert!(outer.contains("mal_owned_function_0(mal_context"));

    let fixture = NativeFixture::new("owned-direct-symbol-calls");
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
fn balances_shared_rope_concatenations_before_materialization() {
    let base = "a".repeat(300);
    let source = format!(
        "prepend :: Symbol -> Symbol := \\(value :: Symbol) {{ \"x\" + value }};\n\
         grow :: (Int64, Symbol) -> Symbol := \\(remaining :: Int64, value :: Symbol) {{\n\
           if (remaining == 0)\n\
           then {{ value }}\n\
           else {{\n\
             next := prepend(value);\n\
             if (#value == 0u64)\n\
             then {{ grow(remaining - 1, next) }}\n\
             else {{ grow(remaining - 1, next) }}\n\
           }};\n\
         }};\n\
         main :: Unit -> Int32 := \\() {{\n\
           value := grow(5000i64, \"{base}\");\n\
           if ((#value == 5300u64) && (value # 0u64 == 'x') &&\n\
               (value # 4999u64 == 'x') && (value # 5000u64 == 'a'))\n\
           then {{ 0 }}\n\
           else {{ 1 }};\n\
         }};"
    );
    let generated = emit(&source).expect("emit balanced shared Symbol rope");
    let prepend = generated_function(&generated.source, "prepend");
    assert!(prepend.contains("mal_symbol_concatenate(mal_context"));
    assert!(!prepend.contains("mal_owned_function_0("));
    assert!(generated.source.contains("mal_symbol_rope_balance"));

    let fixture = NativeFixture::new("balanced-shared-symbol-rope");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_VALIDATE_SYMBOLS",
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
fn balances_mixed_shared_rope_growth_and_joins() {
    let base = "a".repeat(300);
    let source = format!(
        "prepend :: Symbol -> Symbol := \\(value :: Symbol) {{ \"l\" + value }};\n\
         append :: Symbol -> Symbol := \\(value :: Symbol) {{ value + \"r\" }};\n\
         step :: Symbol -> Symbol := \\(value :: Symbol) {{\n\
           prefixed := prepend(value);\n\
           if (#value == 0u64) then {{ append(prefixed) }} else {{ append(prefixed) }};\n\
         }};\n\
         grow :: (Int64, Symbol) -> Symbol := \\(remaining :: Int64, value :: Symbol) {{\n\
           if (remaining == 0) then {{ value }} else {{ grow(remaining - 1, step(value)) }};\n\
         }};\n\
         join :: (Symbol, Symbol) -> Symbol := \\(left :: Symbol, right :: Symbol) {{ left + right }};\n\
         main :: Unit -> Int32 := \\() {{\n\
           left := grow(2000i64, \"{base}\");\n\
           right := grow(2000i64, \"{base}\");\n\
           value := join(left, right);\n\
           ok := (#value == 8600u64) && (value # 0u64 == 'l') &&\n\
                 (value # 1999u64 == 'l') && (value # 2000u64 == 'a') &&\n\
                 (value # 4299u64 == 'r') && (value # 4300u64 == 'l') &&\n\
                 (value # 8599u64 == 'r');\n\
           if (ok) then {{ 0 }} else {{ 1 }};\n\
         }};"
    );
    let generated = emit(&source).expect("emit mixed balanced Symbol ropes");
    let fixture = NativeFixture::new("mixed-balanced-symbol-ropes");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
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
fn shares_repeated_subtrees_in_deeply_nested_symbol_calls() {
    let mut expression = "\"a\"".to_owned();
    for _ in 0..18 {
        expression = format!("twice({expression}, \"\")");
    }
    let source = format!(
        "twice :: (Symbol, Symbol) -> Symbol := \\(a :: Symbol, b :: Symbol) {{ a + a + b + b }};\n\
         main :: Unit -> Int32 := \\() {{\n\
           value := {expression};\n\
           if ((#value == 262144u64) && (value # 0u64 == 'a') &&\n\
               (value # 262143u64 == 'a'))\n\
           then {{ 0 }} else {{ 1 }};\n\
         }};"
    );
    let generated = emit(&source).expect("emit deeply nested Symbol calls");
    let fixture = NativeFixture::new("nested-symbol-rope-dag");
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
fn preserves_bytes_when_switching_between_prepend_and_append_reuse() {
    let generated = emit(
        r#"grow :: (Int64, Symbol) -> Symbol := \(remaining :: Int64, value :: Symbol) {
  if (remaining == 0)
  then { value + "tail" }
  else { grow(remaining - 1, "x" + value) };
};
main :: Unit -> Int32 := \() {
  value := grow(1000i64, "center");
  ok := (#value == 1010u64) &&
        (value # 0u64 == 'x') &&
        (value # 999u64 == 'x') &&
        (value # 1000u64 == 'c') &&
        (value # 1009u64 == 'l');
  if (ok) then { 0 } else { 1 };
};"#,
    )
    .expect("emit bidirectional flat Symbol reuse");
    let fixture = NativeFixture::new("bidirectional-symbol-reuse");
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
fn does_not_mutate_a_shared_right_symbol_during_prepend() {
    let output = compile_and_run(
        r#"main :: Unit -> Int32 := \() {
  right := "b" + "c";
  alias := right;
  joined := "a" + right;
  if ((alias == "bc") && (joined == "abc")) then { 0 } else { 1 };
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
fn materializes_rope_bytes_recursively_at_the_host_boundary() {
    let generated = emit(
        r#"Choice :: [Unit, Symbol];
Envelope :: (Symbol, Choice);
extern inspect :: Envelope -> UInt64;
prepend :: Symbol -> Symbol := \(value :: Symbol) {
  "x" + value;
};
grow :: (Int64, Symbol) -> Symbol := \(remaining :: Int64, value :: Symbol) {
  if (remaining == 0)
  then { value }
  else {
    next := prepend(value);
    if (#value == 0u64)
    then { grow(remaining - 1, next) }
    else { grow(remaining - 1, next) };
  };
};
main :: Unit -> Int32 := \() {
  value := grow(1000i64, "");
  choice := Choice[1](value);
  if (extern inspect(value, choice) == 2000u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit recursive host materialization");

    let fixture = NativeFixture::new("rope-host-boundary");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"
#include <stddef.h>

MAL_DEFINE_inspect(context, direct, choice) {
    MalType_Symbol nested = MAL_OPERATION(Choice, expect_1)(context, choice);
    MalType_Symbol held = MAL_CLONE(Symbol)(context, direct);
    if (direct.data == NULL || nested.data == NULL) {
        MAL_DROP(Symbol)(context, &held);
        return UINT64_C(0);
    }
    for (uint64_t index = 0; index < direct.length; ++index) {
        if (direct.data[index] != (uint8_t)'x' || nested.data[index] != (uint8_t)'x') {
            MAL_DROP(Symbol)(context, &held);
            return UINT64_C(0);
        }
    }
    uint64_t result = held.length + nested.length;
    MAL_DROP(Symbol)(context, &held);
    return result;
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
fn reports_materialization_failure_before_calling_the_host() {
    let generated = emit(
        r#"extern inspect :: Symbol -> Unit;
prepend :: Symbol -> Symbol := \(value :: Symbol) { "x" + value };
grow :: (Symbol, Int64) -> Symbol := \(value :: Symbol, remaining :: Int64) {
  if (remaining == 0)
  then { value }
  else {
    next := prepend(value);
    if (#value == 0u64)
    then { grow(next, remaining - 1) }
    else { grow(next, remaining - 1) };
  };
};
main :: Unit -> Int32 := \() {
  extern inspect(grow("abcdefghijklmnopqrstuvwxyz", 300i64));
  0;
};"#,
    )
    .expect("emit host-boundary materialization failure");
    let fixture = NativeFixture::new("rope-host-boundary-materialization-failure");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MAL_DEFINE_inspect(context, value) {
    (void)value;
    mal_trap(context, "host operation must not be called");
}
"#,
        &["-DMAL_TEST_FORCE_MATERIALIZATION_FAILURE"],
    );
    let output = fixture.run(executable);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("mal trap: allocation failed"),
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
fn traps_when_a_consumed_symbol_cannot_be_reallocated_for_prepend() {
    let generated = emit(
        r#"prependTwice :: Symbol -> Symbol := \(value :: Symbol) {
  first := "b" + value;
  "c" + first;
};
main :: Unit -> Int32 := \() { prependTwice("a"); 0; };"#,
    )
    .expect("emit consuming Symbol prepend reallocation");
    let fixture = NativeFixture::new("symbol-prepend-reallocation-failure");
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
fn omits_symbol_precondition_traps_from_generated_c() {
    let generated = emit(
        r#"main :: Unit -> Int32 := \() {
  "a" # 0u64;
  "a" + "b";
  0;
};"#,
    )
    .expect("emit Symbol operations");

    assert!(!generated.source.contains("Symbol index out of range"));
    assert!(!generated.source.contains("Symbol length overflow"));
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
        "MalSymbolAdmission mal_SymbolAdmission_begin(MalContext *context, uint64_t minimum_capacity);"
    ));
    assert!(!generated.header.contains("mal_Symbol_copy_from_bytes"));
    assert!(contains_ignoring_whitespace(
        &generated.header,
        "MalType_Symbol mal_ext_fetch(MalContext *context);"
    ));
    let fixture = NativeFixture::new("symbol-admission");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"
#include <string.h>

MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission dropped = mal_SymbolAdmission_begin(context, UINT64_C(3));
    mal_SymbolAdmission_drop(context, &dropped);
    mal_SymbolAdmission_drop(context, &dropped);

    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(2));
    uint8_t *data = mal_SymbolAdmission_data(&admission);
    data[0] = 'h';
    data[1] = 'o';
    mal_SymbolAdmission_reserve(context, &admission, UINT64_C(6));
    if (mal_SymbolAdmission_capacity(&admission) < UINT64_C(6)) {
        mal_trap(context, "Symbol admission capacity did not grow");
    }
    data = mal_SymbolAdmission_data(&admission);
    const uint8_t rest[4] = { 's', 't', 0, 255 };
    memcpy(data + 2, rest, sizeof(rest));
    MalType_Symbol result = mal_SymbolAdmission_finish(context, &admission, UINT64_C(6));
    mal_SymbolAdmission_drop(context, &admission);
    return result;
}
"#,
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=3",
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
fn traps_symbol_admission_allocation_failure_and_invalid_lengths() {
    let source =
        "extern fetch :: Unit -> Symbol; main :: Unit -> Int32 := \\() { extern fetch(); 0; };";

    let failure_fixture = NativeFixture::new("symbol-admission-failure");
    let failure_executable = failure_fixture.compile_generated_with_options(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(1));
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(1));
}
"#,
        &["-DMAL_TEST_FORCE_ALLOCATION_FAILURE"],
    );
    let failure = failure_fixture.run(failure_executable);
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("mal trap: allocation failed"));

    let overflow_fixture = NativeFixture::new("symbol-admission-overflow");
    let overflow_executable = overflow_fixture.compile_generated(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_MAX);
    return mal_SymbolAdmission_finish(context, &admission, UINT64_MAX);
}
"#,
    );
    let overflow = overflow_fixture.run(overflow_executable);
    assert!(!overflow.status.success());
    assert!(
        String::from_utf8_lossy(&overflow.stderr).contains("mal trap: allocation size overflow")
    );

    let length_fixture = NativeFixture::new("symbol-admission-length");
    let length_executable = length_fixture.compile_generated(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(1));
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(2));
}
"#,
    );
    let length = length_fixture.run(length_executable);
    assert!(!length.status.success());
    assert!(
        String::from_utf8_lossy(&length.stderr)
            .contains("mal trap: Symbol admission length exceeds capacity")
    );

    let growth_fixture = NativeFixture::new("symbol-admission-growth-failure");
    let growth_executable = growth_fixture.compile_generated_with_options(
        emit(source).expect("emit Symbol ABI"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(1));
    mal_SymbolAdmission_data(&admission)[0] = UINT8_C(1);
    mal_SymbolAdmission_reserve(context, &admission, UINT64_C(2));
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(1));
}
"#,
        &["-DMAL_TEST_FORCE_REALLOCATION_FAILURE"],
    );
    let growth = growth_fixture.run(growth_executable);
    assert!(!growth.status.success());
    assert!(String::from_utf8_lossy(&growth.stderr).contains("mal trap: allocation failed"));
}

#[test]
fn finishes_an_empty_symbol_admission_without_leaking_reserved_storage() {
    let source = r#"extern fetch :: Unit -> Symbol;
main :: Unit -> Int32 := \() {
  if (extern fetch() == "") then { 0 } else { 1 };
};"#;
    let fixture = NativeFixture::new("empty-symbol-admission");
    let executable = fixture.compile_generated_with_options(
        emit(source).expect("emit empty Symbol admission"),
        r#"#include "program.mal.h"
MalType_Symbol mal_ext_fetch(MalContext *context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(8));
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(0));
}
"#,
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    assert!(fixture.run(executable).status.success());
}
