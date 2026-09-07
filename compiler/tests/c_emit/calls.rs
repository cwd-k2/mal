use super::*;

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
        "MalType_Bool mal_ext_exchange(MalContext *context, MalType_Bool argument_0, \
         MalRepr_Product_0 argument_1);"
    ));
    assert!(generated.header.contains(
        "struct MalRepr_Product_0 {\n    MalType_Bool field_0;\n    MalType_Int32 field_1;\n};"
    ));
    assert!(!generated.header.contains("MalRepr_Sum_"));
    assert!(!generated.source.contains(".tag"));

    let fixture = NativeFixture::new("scalar-bool-abi");
    let executable = fixture.compile_generated(
        generated,
        r#"#include "program.mal.h"

MalType_Bool mal_ext_exchange(
    MalContext *context,
    MalType_Bool outer,
    MalRepr_Product_0 inner
) {
    (void)context;
    return (outer == MAL_TRUE && inner.field_0 == MAL_FALSE &&
            inner.field_1 == INT32_C(42)) ? MAL_TRUE : MAL_FALSE;
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
            .contains("/* mal source binding: double */\nstatic MalType_Int64 mal_function_0(")
    );
    assert!(
        generated
            .source
            .contains("/* mal source binding: main */\nstatic MalType_Int32 mal_function_1(")
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
        "static MalType_Int64 mal_direct_function_0(MalContext *mal_context, \
         const void *mal_environment, MalType_Int64 mal_direct_parameter_0, \
         MalType_Int64 mal_direct_parameter_1)"
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
        "MalType_Int64 mal_direct_parameter_0, MalType_Int64 mal_direct_parameter_1, \
         MalType_Int64 mal_direct_parameter_2)"
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
fn lowers_managed_direct_tail_recursion_with_constant_stack() {
    let source = "extern input :: Unit -> Symbol;\n\
         count :: (Symbol, Int64) -> UInt64 := \\(value :: Symbol, remaining :: Int64) {\n\
           if (remaining == 0) then { #value } else {\n\
             count(value, remaining - 1)\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           if (count(extern input(), 1000000i64) == 1u64) then { 0 } else { 1 };\n\
         };";
    let generated = emit(source).expect("emit managed tail-recursive C");
    assert!(generated.source.contains("goto mal_tail_entry;"));

    let fixture = NativeFixture::new("managed-direct-tail-recursion");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MalType_Symbol mal_ext_input(MalContext *context) {
    static const uint8_t bytes[] = { UINT8_C(120) };
    return mal_Symbol_copy_from_bytes(context, bytes, UINT64_C(1));
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
fn cleans_managed_case_bindings_on_direct_tail_edges() {
    let source = r#"Choice :: [Unit, Symbol];
extern input :: Unit -> Symbol;
walk :: (Choice, Int64) -> Symbol := \(choice :: Choice, remaining :: Int64) {
  case (choice)
    [0](_) { "empty" }
    [1](value) {
      if (remaining == 0)
      then { value }
      else { walk(choice, remaining - 1) };
    };
};
main :: Unit -> Int32 := \() {
  result := walk(Choice[1](extern input()), 100000i64);
  if (result == "x") then { 0 } else { 1 };
};"#;
    let generated = emit(source).expect("emit managed case tail-recursive C");
    assert!(generated.source.contains("goto mal_tail_entry;"));

    let fixture = NativeFixture::new("managed-case-tail-recursion");
    let executable = fixture.compile_generated_with_options(
        generated,
        r#"#include "program.mal.h"

MalType_Symbol mal_ext_input(MalContext *context) {
    static const uint8_t bytes[] = { UINT8_C(120) };
    return mal_Symbol_copy_from_bytes(context, bytes, UINT64_C(1));
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
