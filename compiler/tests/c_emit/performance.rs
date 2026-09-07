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

    let executable = fixture.compile_generated_with_options(
        generated,
        INPUT_HOST,
        &[
            "-DMAL_TEST_RETAIN_LIMIT=26",
            "-DMAL_TEST_RELEASE_LIMIT=27",
            "-DMAL_TEST_MATERIALIZATION_LIMIT=26",
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
    assert!(generated.source.contains("mal_tail_next_parameter"));

    let fixture = NativeFixture::new("managed-aggregate-tail-cost");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_RETAIN_LIMIT=256",
            "-DMAL_TEST_RELEASE_LIMIT=256",
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
