use super::*;

const INPUT_HOST: &str = r#"#include "program.mal.h"
#include <string.h>

MAL_DEFINE_input(context) {
    static const uint8_t bytes[] = "abcdefghijklmnopqrstuvwxyz";
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, sizeof(bytes) - 1);
    memcpy(mal_SymbolAdmission_data(&admission), bytes, sizeof(bytes) - 1);
    return mal_SymbolAdmission_finish(context, &admission, sizeof(bytes) - 1);
}
"#;

#[test]
fn exposes_signed_right_shift_as_an_arithmetic_shift_to_clang() {
    let generated = emit(
        r#"extern input :: Unit -> Int64;
extern output :: Int64 -> Unit;
main :: Unit -> Int32 := \() { extern output(extern input() >> 1i64); 0 };"#,
    )
    .expect("emit signed right shift");
    let fixture = NativeFixture::new("signed-right-shift-cost");
    let llvm_ir = fixture.compile_generated_to_llvm_ir(generated);

    assert!(llvm_ir.contains("ashr i64"), "{llvm_ir}");
    assert!(!llvm_ir.contains("lshr i64"), "{llvm_ir}");
}

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
    assert!(
        generated
            .source
            .contains("static inline void mal_symbol_release(")
    );

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
            "-DMAL_TEST_MATERIALIZATION_LIMIT=0",
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
fn compares_rope_symbols_without_materialization() {
    let generated = emit(
        r#"prepend :: Symbol -> Symbol := \(value :: Symbol) { "x" + value };
grow :: (Symbol, Int64) -> Symbol := \(value :: Symbol, remaining :: Int64) {
  if (remaining == 0)
  then { value }
  else { grow(prepend(value), remaining - 1) };
};
main :: Unit -> Int32 := \() {
  value := grow("abcdefghijklmnopqrstuvwxyz", 300i64);
  if (value == value) then { 0 } else { 1 };
};"#,
    )
    .expect("emit rope Symbol equality");
    let fixture = NativeFixture::new("rope-symbol-equality-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_MATERIALIZATION_LIMIT=0",
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
fn keeps_nested_parameter_bindings_in_separate_tail_slots() {
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
    .expect("emit nested parameter tail slots");
    let walk = generated_function(&generated.source, "walk");
    assert!(walk.contains("mal_tail_next_parameter_0"));
    assert!(!walk.contains(" mal_tail_next_parameter ="));

    let fixture = NativeFixture::new("nested-parameter-tail-slots");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    assert!(fixture.run(executable).status.success());
}

#[test]
fn borrows_invariant_nested_tail_fields_through_known_calls() {
    let generated = emit(
        r#"Nested :: (Symbol, UInt64);
byteAt :: (Symbol, UInt64) -> UInt8 := \(value :: Symbol, index :: UInt64) {
  value # index
};
scan :: (Nested, UInt64, UInt64) -> UInt64 := \(state :: Nested, index :: UInt64, total :: UInt64) {
  (value, length) := state;
  if (index == length)
  then { total }
  else { scan(state, index + 1u64, total + UInt64(byteAt(value, index))) };
};
main :: Unit -> Int32 := \() {
  value := "ab" + "cd";
  if (scan((value, 4u64), 0u64, 0u64) == 394u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit borrowed nested tail field");
    let scan = generated_function(&generated.source, "scan");
    assert!(scan.contains("mal_direct_function_"));
    assert!(!scan.contains("mal_symbol_retain("));

    let fixture = NativeFixture::new("borrowed-nested-tail-field");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=0",
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
fn borrows_invariant_managed_tail_slots_through_known_calls() {
    let generated = emit(
        r#"extern input :: Unit -> Symbol;
inputByte :: (Symbol, UInt64) -> UInt8 := \(value :: Symbol, index :: UInt64) {
  value # index
};
scan :: (Symbol, UInt64, UInt64, UInt64) -> UInt64 := \(value :: Symbol, length :: UInt64, index :: UInt64, total :: UInt64) {
  if (index == length)
  then { total }
  else { scan(value, length, index + 1u64, total + UInt64(inputByte(value, index))) };
};
main :: Unit -> Int32 := \() {
  value := extern input();
  if (scan(value, 4u64, 0u64, 0u64) == 394u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit borrowed managed tail slot");
    let scan = generated_function(&generated.source, "scan");
    assert!(scan.contains("mal_direct_function_"));
    assert_eq!(scan.matches("mal_symbol_retain(").count(), 1);

    let fixture = NativeFixture::new("borrowed-managed-tail-slot");
    let executable = fixture.compile_generated_with_options(
        generated,
        INPUT_HOST,
        &[
            "-DMAL_TEST_RETAIN_LIMIT=0",
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
fn copies_a_borrowed_direct_parameter_when_it_escapes() {
    let generated = emit(
        r#"first :: (Symbol, Int64) -> Symbol := \(value :: Symbol, ignored :: Int64) { value };
main :: Unit -> Int32 := \() {
  value := "a" + "b";
  result := first(value, 0i64);
  if ((value == "ab") && (result == "ab")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit escaping borrowed parameter");
    let first = generated_function(&generated.source, "first");
    assert_eq!(first.matches("mal_symbol_retain(").count(), 1);

    let fixture = NativeFixture::new("escaping-borrowed-direct-parameter");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=1",
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=1",
            "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ],
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
