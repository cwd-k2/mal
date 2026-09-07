use super::*;

const INPUT_HOST: &str = r#"#include "program.mal.h"

MAL_DEFINE_input(context) {
    static const uint8_t bytes[] = "abcdefghijklmnopqrstuvwxyz";
    return mal_Symbol_copy_from_bytes(context, bytes, sizeof(bytes) - 1);
}
"#;

#[test]
fn tracks_flat_symbol_scan_cost_and_keeps_an_optimized_ir_fixture() {
    let generated = emit(
        r#"extern input :: Unit -> Symbol;
scan :: (Symbol, UInt64, UInt64) -> UInt64 := \(value :: Symbol, index :: UInt64, total :: UInt64) {
  if (index == #value)
  then { total }
  else { scan(value, index + 1u64, total + UInt64(value # index)) };
};
main :: Unit -> Int32 := \() {
  if (scan(extern input(), 0u64, 0u64) == 2847u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit flat Symbol scan");

    let fixture = NativeFixture::new("flat-symbol-scan-cost");
    let llvm_ir = fixture.compile_generated_to_llvm_ir(generated.clone());
    assert!(llvm_ir.contains("mal_ext_input"), "{llvm_ir}");
    assert!(llvm_ir.contains("icmp eq i64"), "{llvm_ir}");
    assert!(!llvm_ir.contains("@mal_symbol_at("), "{llvm_ir}");
    assert!(llvm_ir.contains("@mal_symbol_at_slow("), "{llvm_ir}");
    assert!(llvm_ir.contains("load i8"), "{llvm_ir}");

    let executable = fixture.compile_generated_with_options(
        generated,
        INPUT_HOST,
        &[
            "-DMAL_TEST_RETAIN_LIMIT=0",
            "-DMAL_TEST_RELEASE_LIMIT=1",
            "-DMAL_TEST_MATERIALIZATION_LIMIT=0",
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=1",
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
fn keeps_flat_symbol_equality_off_the_materialization_path() {
    let generated = emit(
        r#"extern input :: Unit -> Symbol;
main :: Unit -> Int32 := \() {
  value := extern input();
  if (value == "abcdefghijklmnopqrstuvwxyz") then { 0 } else { 1 };
};"#,
    )
    .expect("emit flat Symbol equality");
    let fixture = NativeFixture::new("flat-symbol-equality-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        INPUT_HOST,
        &[
            "-DMAL_TEST_MATERIALIZATION_LIMIT=0",
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=1",
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
fn borrows_a_fresh_symbol_through_an_ephemeral_access_product() {
    let generated = emit(
        r#"main :: Unit -> Int32 := \() {
  value := "a" + "b";
  byte := value # 0u64;
  if ((byte == 'a') && (value == "ab")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit a fresh Symbol byte access");
    let fixture = NativeFixture::new("fresh-symbol-ephemeral-access");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=0",
            "-DMAL_TEST_MATERIALIZATION_LIMIT=0",
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=1",
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
fn passes_unmanaged_ephemeral_products_directly_to_known_calls() {
    let generated = emit(
        r#"add :: (Int64, Int64) -> Int64 := \(left :: Int64, right :: Int64) { left + right };
main :: Unit -> Int32 := \() { Int32(add(20i64, 22i64) - 42i64) };"#,
    )
    .expect("emit an ephemeral known-call product");
    let main = generated_function(&generated.source, "main");
    assert!(!main.contains("MalRepr_Product_"), "{main}");

    let fixture = NativeFixture::new("ephemeral-known-call-product");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn projects_ephemeral_product_fields_without_storing_the_product() {
    let generated = emit(
        r#"main :: Unit -> Int32 := \() {
  value := "a" + "b";
  (left, right) := (value, value);
  if ((value == "ab") && (left == "ab") && (right == "ab")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit an ephemeral product projection");
    let main = generated_function(&generated.source, "main");
    assert!(!main.contains("MalRepr_Product_"), "{main}");

    let fixture = NativeFixture::new("ephemeral-product-projection");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    assert!(fixture.run(executable).status.success());
}

#[test]
fn tracks_rope_symbol_scan_cost() {
    let generated = emit(
        r#"prepend :: Symbol -> Symbol := \(value :: Symbol) { "x" + value };
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
scan :: (Symbol, UInt64, UInt64) -> UInt64 := \(value :: Symbol, index :: UInt64, total :: UInt64) {
  if (index == #value)
  then { total }
  else { scan(value, index + 1u64, total + UInt64(value # index)) };
};
main :: Unit -> Int32 := \() {
  value := grow("abcdefghijklmnopqrstuvwxyz", 32i64);
  if (scan(value, 0u64, 0u64) == 6687u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit rope Symbol scan");
    let fixture = NativeFixture::new("rope-symbol-scan-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=512",
            "-DMAL_TEST_RELEASE_LIMIT=512",
            "-DMAL_TEST_MATERIALIZATION_LIMIT=128",
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=128",
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
fn tracks_transient_host_symbol_admission_cost() {
    let generated = emit(
        r#"extern input :: Unit -> Symbol;
read :: (Int64, UInt64) -> UInt64 := \(remaining :: Int64, total :: UInt64) {
  if (remaining == 0)
  then { total }
  else {
    value := extern input();
    read(remaining - 1, total + #value);
  };
};
main :: Unit -> Int32 := \() {
  if (read(32i64, 0u64) == 832u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit transient Symbol admission");
    let fixture = NativeFixture::new("transient-symbol-admission-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        INPUT_HOST,
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
fn tracks_managed_aggregate_tail_state_cost() {
    let generated = emit(
        r#"State :: (Symbol, Symbol, Int64);
walk :: State -> Symbol := \(left :: Symbol, right :: Symbol, remaining :: Int64) {
  if (remaining == 0)
  then { left + right }
  else { walk(left, right, remaining - 1) };
};
main :: Unit -> Int32 := \() {
  left := "a" + "b";
  right := "c" + "d";
  if (walk(left, right, 64i64) == "abcd") then { 0 } else { 1 };
};"#,
    )
    .expect("emit managed aggregate tail state");
    assert!(
        generated
            .source
            .contains("mal_tail_next_parameter_0 = mal_value_source_")
    );
    assert!(
        !generated
            .source
            .contains("MalRepr_Product_0 mal_tail_next_parameter =")
    );

    let fixture = NativeFixture::new("managed-aggregate-tail-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=0",
            "-DMAL_TEST_RELEASE_LIMIT=16",
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
fn keeps_aggregate_tail_state_when_a_parameter_leaf_is_a_product() {
    let generated = emit(
        r#"Nested :: (Symbol, Int64);
walk :: (Nested, Int64) -> Symbol := \(state :: Nested, remaining :: Int64) {
  (value, _) := state;
  if (remaining == 0)
  then { value }
  else { walk(state, remaining - 1) };
};
main :: Unit -> Int32 := \() {
  value := "a" + "b";
  if (walk((value, 0i64), 4i64) == "ab") then { 0 } else { 1 };
};"#,
    )
    .expect("emit nested aggregate tail state");
    assert!(generated.source.contains(" mal_tail_next_parameter ="));

    let fixture = NativeFixture::new("nested-aggregate-tail-fallback");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    assert!(fixture.run(executable).status.success());
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
