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
    let result = fixture.run(executable);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn executes_escaping_capturing_closures() {
    let output = compile_and_run(
        "extern printInt32 :: Int32 -> Unit;\n\
         makeAdder :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           \\(y :: Int32) { x + y; };\n\
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
fn stack_allocates_a_capturing_closure_used_only_as_a_local_callee() {
    let generated = emit(
        "main :: Unit -> Int32 := \\() {\n\
           base :: Int32 := 40;\n\
           add := \\(value :: Int32) { base + value; };\n\
           add(2) - 42;\n\
         };",
    )
    .expect("emit a non-escaping closure");

    assert!(
        generated
            .source
            .contains("MalEnvironment_1 mal_stack_environment_"),
        "{}",
        generated.source
    );
    assert!(
        generated
            .source
            .contains("mal_function_1(mal_context, &mal_stack_environment_")
    );
    assert!(!generated.source.contains("mal_new_environment_"));

    let fixture = NativeFixture::new("stack-closure-environment");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=0"],
    );
    let output = fixture.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn heap_allocates_a_local_closure_live_across_control_suspension() {
    let generated = emit(
        "walk :: (Int32, Int32) -> Int32 := \\(base :: Int32, depth :: Int32) {\n\
           if (depth == 0i32)\n\
             then { 0i32 }\n\
             else {\n\
               add := \\(value :: Int32) { base + value; };\n\
               child := walk(base, depth - 1i32);\n\
               add(child);\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() { walk(1i32, 4i32) - 4i32; };",
    )
    .expect("emit a closure crossing a recursive suspension");

    assert!(generated.source.contains("mal_control_environment_"));
    assert!(generated.source.contains("mal_control_push("));
    assert!(
        !generated
            .source
            .contains("MalEnvironment_1 mal_stack_environment_")
    );

    let fixture = NativeFixture::new("suspended-closure-environment");
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
fn preserves_a_managed_capture_across_control_suspension() {
    let generated = emit(
        "walk :: (Int32, Symbol) -> Symbol := \\(depth :: Int32, prefix :: Symbol) {\n\
           if (depth == 0i32)\n\
             then { prefix }\n\
             else {\n\
               append := \\(suffix :: Symbol) { prefix + suffix; };\n\
               child := walk(depth - 1i32, prefix);\n\
               append(child);\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(8i32, seed);\n\
           Int32(#result) - 18i32;\n\
         };",
    )
    .expect("emit a managed capture crossing suspension");
    assert!(generated.source.contains("mal_control_environment_"));
    assert!(generated.source.contains("MalRepr_Closure_"));

    let fixture = NativeFixture::new("suspended-managed-capture");
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
fn emits_control_storage_only_for_programs_with_dispatch_edges() {
    let recursive = emit(
        "depth :: Int32 -> Int32 := \\(value :: Int32) {\n\
           if (value == 0i32)\n\
             then { 0i32 }\n\
             else { 1i32 + depth(value - 1i32) };\n\
         };\n\
         main :: Unit -> Int32 := \\() { depth(4i32) - 4i32; };",
    )
    .expect("emit a recursive control stack");
    assert!(recursive.source.contains("MalControlStack"));
    assert!(recursive.source.contains("mal_control_push("));
    assert!(
        recursive
            .source
            .contains("mal implementation resource failure: ")
    );
    let fixture = NativeFixture::new("control-stack-runtime");
    let executable = fixture.compile_generated(recursive, "");
    assert!(fixture.run(executable).status.success());

    let acyclic = emit(
        "identity :: Int32 -> Int32 := \\(value :: Int32) { value; };\n\
         main :: Unit -> Int32 := \\() { identity(0i32); };",
    )
    .expect("emit an acyclic direct call");
    assert!(!acyclic.source.contains("MalControlStack"));
    assert!(!acyclic.source.contains("mal_control_push"));
}

#[test]
fn isolates_nested_recursive_regions_in_separate_cached_arenas() {
    let generated = emit(
        "inner :: Int32 -> Int32 := \\(depth :: Int32) {\n\
           if (depth == 0i32) then { 0i32 } else { 1i32 + inner(depth - 1i32) };\n\
         };\n\
         outer :: Int32 -> Int32 := \\(depth :: Int32) {\n\
           if (depth == 0i32) then { 0i32 } else {\n\
             child := outer(depth - 1i32);\n\
             child + inner(depth);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { outer(4i32) - 10i32; };",
    )
    .expect("emit nested recursive regions");

    assert!(
        generated
            .source
            .contains("MalControlStack control_region_0;")
    );
    assert!(
        generated
            .source
            .contains("MalControlStack control_region_1;")
    );
    let fixture = NativeFixture::new("nested-control-regions");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn emits_typed_control_frame_fields_for_live_symbols() {
    let generated = emit(
        "recurse :: (Int32, Symbol) -> Symbol := \\(depth :: Int32, prefix :: Symbol) {\n\
           if (depth == 0i32)\n\
             then { prefix }\n\
             else {\n\
               suffix := recurse(depth - 1i32, prefix);\n\
               prefix + suffix;\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           value := recurse(2i32, \"x\");\n\
           Int32(value # 0u64) - 120i32;\n\
         };",
    )
    .expect("emit a managed recursive frame");

    assert!(generated.source.contains("MalControlFrameHeader header;"));
    assert!(generated.source.contains("MalType_Symbol field_"));
    assert!(generated.source.contains("mal_symbol_retain("));
    assert!(generated.source.contains("mal_symbol_release("));
    assert!(
        !generated
            .source
            .lines()
            .any(|line| line.contains("->field_") && line.contains("mal_symbol_retain("))
    );
    let fixture = NativeFixture::new("typed-symbol-control-frame");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    assert!(fixture.run(executable).status.success());
}

#[test]
fn lowers_deep_symbol_recursion_with_balanced_frame_ownership() {
    let generated = emit(
        "walk :: (Int32, Symbol) -> Symbol := \\(depth :: Int32, value :: Symbol) {\n\
           if (depth == 0i32)\n\
             then { value }\n\
             else {\n\
               resumed := walk(depth - 1i32, value);\n\
               first := resumed # 0u64;\n\
               if (first == 120u8) then { resumed } else { \"bad\" };\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(50000i32, seed);\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    )
    .expect("emit deep managed control frames");
    assert!(generated.source.contains("mal_control_push("));

    let fixture = NativeFixture::new("deep-symbol-control");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-O2", "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    let result = fixture.run(executable);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn lowers_deep_non_tail_self_recursion_without_growing_the_c_stack() {
    let generated = emit(
        "unwind :: Int32 -> Int32 := \\(depth :: Int32) {\n\
           if (depth == 0i32)\n\
             then { 0i32 }\n\
             else { 1i32 + unwind(depth - 1i32) };\n\
         };\n\
         main :: Unit -> Int32 := \\() { unwind(300000i32) - 300000i32; };",
    )
    .expect("emit deep non-tail recursion");
    assert!(generated.source.contains("goto mal_control_state_"));
    assert!(generated.source.contains("mal_control_push("));

    let fixture = NativeFixture::new("deep-non-tail-control");
    let executable = fixture.compile_generated_with_options(generated, "", &["-O2"]);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn lowers_a_deep_indirect_tail_forwarder_without_growing_the_c_stack() {
    let generated = emit(
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := \\(function :: Int32 -> Int32, value :: Int32) {\n\
           function(value);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           recurse :: Int32 -> Int32 := \\(value :: Int32) {\n\
             if (value == 0i32) then { 0i32 } else { apply(recurse, value - 1i32) };\n\
           };\n\
           recurse(300000i32);\n\
         };",
    )
    .expect("emit a deep indirect tail forwarder");
    assert!(generated.source.contains("goto mal_control_state_"));

    let fixture = NativeFixture::new("deep-indirect-tail-forwarder");
    let executable = fixture.compile_generated_with_options(generated, "", &["-O2"]);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn preserves_managed_arguments_through_a_deep_indirect_tail_forwarder() {
    let generated = emit(
        "apply :: (((Int32, Symbol) -> Symbol), (Int32, Symbol)) -> Symbol := \\(function :: (Int32, Symbol) -> Symbol, argument :: (Int32, Symbol)) {\n\
           function(argument);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           recurse :: (Int32, Symbol) -> Symbol := \\(depth :: Int32, value :: Symbol) {\n\
             if (depth == 0i32) then { value } else { apply(recurse, (depth - 1i32, value)) };\n\
           };\n\
           result := recurse(50000i32, \"x\" + \"y\");\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    )
    .expect("emit managed arguments through an indirect tail forwarder");

    let fixture = NativeFixture::new("managed-indirect-tail-forwarder");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-O2", "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"],
    );
    let result = fixture.run(executable);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn lowers_a_deep_first_class_call_cycle_without_growing_the_c_stack() {
    let generated = emit(
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := \\(function :: Int32 -> Int32, value :: Int32) {\n\
           function(value);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           recurse :: Int32 -> Int32 := \\(value :: Int32) {\n\
             if (value == 0i32) then { 0i32 } else {\n\
               child := apply(recurse, value - 1i32);\n\
               child + 1i32;\n\
             };\n\
           };\n\
           recurse(300000i32) - 300000i32;\n\
         };",
    )
    .expect("emit a deep first-class call cycle");
    assert!(generated.source.contains("static void mal_run_control_"));
    assert!(generated.source.contains("mal_control_push("));

    let fixture = NativeFixture::new("deep-first-class-call-cycle");
    let executable = fixture.compile_generated_with_options(generated, "", &["-O2"]);
    let result = fixture.run(executable);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn emits_independent_first_class_cycles_as_separate_regions() {
    let generated = emit(
        "apply32 :: ((Int32 -> Int32), Int32) -> Int32 := \\(function :: Int32 -> Int32, value :: Int32) { function(value); };\n\
         apply64 :: ((Int64 -> Int64), Int64) -> Int64 := \\(function :: Int64 -> Int64, value :: Int64) { function(value); };\n\
         main :: Unit -> Int32 := \\() {\n\
           recurse32 :: Int32 -> Int32 := \\(value :: Int32) {\n\
             if (value == 0i32) then { 0i32 } else { child := apply32(recurse32, value - 1i32); child + 1i32; };\n\
           };\n\
           recurse64 :: Int64 -> Int64 := \\(value :: Int64) {\n\
             if (value == 0i64) then { 0i64 } else { child := apply64(recurse64, value - 1i64); child + 1i64; };\n\
           };\n\
           recurse32(4i32) + Int32(recurse64(5i64)) - 9i32;\n\
         };",
    )
    .expect("emit independent first-class control regions");

    assert!(generated.source.contains("static void mal_run_control_0("));
    assert!(generated.source.contains("static void mal_run_control_1("));
    let fixture = NativeFixture::new("independent-first-class-control-regions");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn preserves_a_managed_capture_through_a_first_class_call_cycle() {
    let generated = emit(
        "apply :: ((Int32 -> Symbol), Int32) -> Symbol := \\(function :: Int32 -> Symbol, value :: Int32) {\n\
           function(value);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           prefix := \"x\" + \"y\";\n\
           recurse :: Int32 -> Symbol := \\(value :: Int32) {\n\
             if (value == 0i32) then { prefix } else {\n\
               child := apply(recurse, value - 1i32);\n\
               if (child == prefix) then { child } else { \"bad\" };\n\
             };\n\
           };\n\
           result := recurse(50000i32);\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    )
    .expect("emit a managed capture through a first-class call cycle");
    assert!(generated.source.contains("destroy_environment;"));

    let fixture = NativeFixture::new("managed-first-class-call-cycle");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-O1",
            "-fsanitize=address,undefined",
            "-fno-omit-frame-pointer",
            "-DMAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ],
    );
    let result = std::process::Command::new(executable)
        .env("ASAN_OPTIONS", "detect_leaks=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .expect("run sanitized first-class call cycle");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn reports_control_storage_failure_as_an_implementation_resource_limit() {
    let generated = emit(
        "unwind :: Int32 -> Int32 := \\(depth :: Int32) {\n\
           if (depth == 0i32)\n\
             then { 0i32 }\n\
             else { 1i32 + unwind(depth - 1i32) };\n\
         };\n\
         main :: Unit -> Int32 := \\() { unwind(1i32); };",
    )
    .expect("emit control storage failure fixture");
    let fixture = NativeFixture::new("control-resource-failure");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_FORCE_CONTROL_ALLOCATION_FAILURE"],
    );
    let result = fixture.run(executable);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("mal implementation resource failure: control stack allocation failed"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn resumes_multiple_non_tail_self_calls_with_product_parameters() {
    let generated = emit(
        "tree :: (Int32, Int32) -> Int32 := \\(depth :: Int32, seed :: Int32) {\n\
           if (depth == 0i32)\n\
             then { seed }\n\
             else {\n\
               left := tree(depth - 1i32, seed + 1i32);\n\
               right := tree(depth - 1i32, seed + 2i32);\n\
               left + right;\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() { tree(3i32, 0i32) - 36i32; };",
    )
    .expect("emit multiple recursive continuations");
    assert!(
        generated
            .source
            .matches("MalControlFrameHeader header;")
            .count()
            >= 2
    );

    let fixture = NativeFixture::new("multiple-non-tail-control");
    let executable = fixture.compile_generated_with_options(generated, "", &["-O2"]);
    assert!(fixture.run(executable).status.success());
}

#[test]
fn stack_closure_calls_use_flattened_product_entries() {
    let generated = emit(
        "main :: Unit -> Int32 := \\() {\n\
           base :: Int32 := 40;\n\
           add := \\(left :: Int32, right :: Int32) { base + left + right; };\n\
           add(1, 1) - 42;\n\
         };",
    )
    .expect("emit a non-escaping closure with product parameters");

    assert!(
        generated
            .source
            .contains("mal_direct_function_1(mal_context, &mal_stack_environment_")
    );
    assert!(!generated.source.contains("mal_new_environment_"));

    let fixture = NativeFixture::new("stack-closure-product-entry");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &["-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=0"],
    );
    assert!(fixture.run(executable).status.success());
}

#[test]
fn stack_closure_borrows_managed_captures_across_repeated_calls() {
    let generated = emit(
        "main :: Unit -> Int32 := \\() {\n\
           prefix := \"a\";\n\
           append := \\(suffix :: Symbol) { prefix + suffix; };\n\
           first := append(\"b\");\n\
           second := append(\"c\");\n\
           if (first == \"ab\" && second == \"ac\" && prefix == \"a\")\n\
           then { 0 }\n\
           else { 1 };\n\
         };",
    )
    .expect("emit a stack closure with a managed capture");

    assert!(
        generated
            .source
            .contains("MalEnvironment_1 mal_stack_environment_")
    );
    assert!(!generated.source.contains("mal_new_environment_"));
    let fixture = NativeFixture::new("stack-closure-managed-capture");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn heap_allocates_a_recursive_closure_whose_self_value_escapes_a_call() {
    let generated = emit(
        r#"apply :: ((Int32 -> Int32), Int32) -> Int32 := \(function :: Int32 -> Int32, value :: Int32) {
  function(value);
};
main :: Unit -> Int32 := \() {
  offset := 1i32;
  recurse :: Int32 -> Int32 := \(value :: Int32) {
    if (value == 0) then { offset } else { apply(recurse, value - 1) };
  };
  recurse(2) - 1;
};"#,
    )
    .expect("emit a recursive closure passed as a value");

    assert!(
        generated.source.contains("mal_new_environment_")
            || generated.source.contains("mal_control_environment_")
    );
    assert!(!generated.source.contains("mal_stack_environment_"));

    let fixture = NativeFixture::new("escaping-recursive-self-closure");
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
fn transfers_a_symbol_into_an_owned_stack_closure_call() {
    let generated = emit(
        r#"main :: Unit -> Int32 := \() {
  prefix := "p";
  append := \(value :: Symbol) { prefix + value };
  owned := "a" + "b";
  result := append(owned);
  if ((result == "pab") && (prefix == "p")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit an owned stack closure call");

    assert!(
        generated
            .source
            .contains("mal_owned_function_1(mal_context, &mal_stack_environment_")
    );
    assert!(
        generated
            .source
            .contains("mal_symbol_concatenate_consuming_right(")
    );
    assert!(!generated.source.contains("mal_new_environment_"));

    let fixture = NativeFixture::new("owned-stack-closure-call");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=2",
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
fn transfers_owned_parameters_on_each_direct_call_branch() {
    let generated = emit(
        r#"append :: Symbol -> Symbol := \(value :: Symbol) { value + "x" };
choose :: (Bool, Symbol) -> Symbol := \(condition :: Bool, value :: Symbol) {
  if (condition) then { append(value) } else { append(value) };
};
main :: Unit -> Int32 := \() {
  first := "a" + "b";
  second := "c" + "d";
  left := choose(true, first);
  right := choose(false, second);
  if ((left == "abx") && (right == "cdx")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit branch-local owned direct calls");

    assert!(
        generated
            .source
            .contains("mal_owned_function_0(mal_context")
    );
    assert!(
        generated
            .source
            .contains("mal_owned_function_1(mal_context")
    );

    let fixture = NativeFixture::new("branch-owned-direct-calls");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=4",
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
fn keeps_a_reused_direct_call_argument_borrowed() {
    let generated = emit(
        r#"append :: Symbol -> Symbol := \(value :: Symbol) { value + "x" };
main :: Unit -> Int32 := \() {
  value := "a" + "b";
  result := append(value);
  if ((value == "ab") && (result == "abx")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit a borrowed direct call argument");

    assert!(!generated.source.contains("mal_owned_function_0("));
    let fixture = NativeFixture::new("borrowed-reused-direct-argument");
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
fn preserves_a_shared_allocation_when_its_descriptor_is_transferred() {
    let generated = emit(
        r#"append :: Symbol -> Symbol := \(value :: Symbol) { value + "x" };
main :: Unit -> Int32 := \() {
  value := "a" + "b";
  alias := value;
  result := append(value);
  if ((alias == "ab") && (result == "abx")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit a shared owned direct call argument");

    assert!(
        generated
            .source
            .contains("mal_owned_function_0(mal_context")
    );
    let fixture = NativeFixture::new("shared-owned-direct-argument");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=2",
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
fn transfers_an_active_sum_payload_from_an_owned_parameter() {
    let generated = emit(
        r#"Choice :: [Unit, Symbol];
take :: Choice -> Symbol := \(choice :: Choice) {
  case (choice) [0](_) { "empty" } [1](value) { value };
};
main :: Unit -> Int32 := \() {
  value := "a" + "b";
  alias := value;
  choice := Choice[1](value);
  result := take(choice);
  if ((alias == "ab") && (result == "ab")) then { 0 } else { 1 };
};"#,
    )
    .expect("emit an owned sum parameter");

    let take = generated_function(&generated.source, "take");
    let owned = &take[take
        .find("mal_owned_function_0(")
        .expect("owned entry for take")..];
    assert_eq!(owned.matches("mal_symbol_retain(").count(), 1, "{owned}");

    let fixture = NativeFixture::new("owned-sum-parameter");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
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
fn generates_an_owned_entry_for_a_non_tail_self_call() {
    let generated = emit(
        r#"grow :: (Int64, Symbol) -> Symbol := \(remaining :: Int64, value :: Symbol) {
  if (remaining == 0)
  then { value }
  else {
    next := "x" + value;
    nested := grow(remaining - 1, next);
    "y" + nested;
  };
};
main :: Unit -> Int32 := \() {
  value := grow(20i64, "");
  if (#value == 40u64) then { 0 } else { 1 };
};"#,
    )
    .expect("emit an owned non-tail self call");

    assert!(
        generated
            .source
            .contains("mal_owned_function_0(mal_context")
    );
    let fixture = NativeFixture::new("owned-non-tail-self-call");
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
fn heap_allocates_a_local_closure_that_flows_to_another_function() {
    let generated = emit(
        "apply :: (Int32 -> Int32) -> Int32 := \\(operation :: Int32 -> Int32) {\n\
           operation(2);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           base :: Int32 := 40;\n\
           add := \\(value :: Int32) { base + value; };\n\
           apply(add) - 42;\n\
         };",
    )
    .expect("emit a conservatively escaping local closure");

    assert!(generated.source.contains("mal_new_environment_"));
    let fixture = NativeFixture::new("passed-closure-environment");
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
fn executes_top_level_and_local_recursive_closures() {
    let output = compile_and_run(
        "factorial :: Int32 -> Int32 := \\(n :: Int32) {\n\
           if (n == 0) then { 1 } else { n * factorial(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           base :: Int32 := 120;\n\
           local :: Int32 -> Int32 := \\(n :: Int32) {\n\
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
fn emits_direct_calls_for_known_acyclic_functions_and_controls_recursive_edges() {
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
    assert!(generated.source.contains("mal_control_push("));
    assert!(generated.source.contains("goto mal_control_state_"));
    let fixture = NativeFixture::new("direct-known-calls");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn omits_closure_representation_for_direct_only_top_level_functions() {
    let generated = emit(
        "add :: (Int64, Int64) -> Int64 := \\(left :: Int64, right :: Int64) { left + right; };\n\
         main :: Unit -> Int32 := \\() { Int32(add(20i64, 22i64) - 42i64); };",
    )
    .expect("emit a direct-only top-level function");

    assert!(
        generated
            .source
            .contains("static inline MalType_Int64 mal_direct_function_0(")
    );
    assert!(!generated.source.contains("mal_function_0("));
    assert!(!generated.source.contains("{ mal_function_0, NULL, NULL }"));

    let fixture = NativeFixture::new("direct-only-top-level");
    let executable = fixture.compile_generated(generated, "");
    assert!(fixture.run(executable).status.success());
}

#[test]
fn keeps_closure_representation_for_first_class_top_level_functions() {
    let generated = emit(
        "add :: (Int64, Int64) -> Int64 := \\(left :: Int64, right :: Int64) { left + right; };\n\
         apply :: (((Int64, Int64) -> Int64), Int64, Int64) -> Int64 :=\n\
           \\(operation :: (Int64, Int64) -> Int64, left :: Int64, right :: Int64) {\n\
             operation(left, right);\n\
           };\n\
         main :: Unit -> Int32 := \\() { Int32(apply(add, 20i64, 22i64) - 42i64); };",
    )
    .expect("emit a first-class top-level function");

    assert!(generated.source.contains("mal_direct_function_0("));
    assert!(generated.source.contains("mal_function_0("));
    assert!(generated.source.contains("{ mal_function_0, NULL, NULL }"));

    let fixture = NativeFixture::new("first-class-top-level");
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
fn transfers_owned_managed_products_through_a_direct_entry() {
    let generated = emit(
        r#"join :: (Symbol, Symbol) -> Symbol := \(left :: Symbol, right :: Symbol) {
  left + right;
};
main :: Unit -> Int32 := \() {
  left := "a" + "b";
  right := "c" + "d";
  pair := (left, right);
  value := join(pair);
  if (value == "abcd") then { 0 } else { 1 };
};"#,
    )
    .expect("emit an owned managed product entry");

    assert!(generated.source.contains(
        "static MalType_Symbol mal_owned_function_0(MalContext *mal_context, \
         const void *mal_environment, MalType_Symbol mal_direct_parameter_0, \
         MalType_Symbol mal_direct_parameter_1)"
    ));
    assert!(
        generated
            .source
            .contains("mal_owned_function_0(mal_context, NULL,")
    );

    let fixture = NativeFixture::new("owned-managed-product-entry");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
            "-DMAL_TEST_TOTAL_ALLOCATION_LIMIT=4",
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
fn keeps_owned_large_products_on_an_aggregate_entry() {
    let mut types = vec!["Symbol"];
    types.extend(std::iter::repeat_n("Int64", 16));
    let mut parameters = vec!["value :: Symbol".to_owned()];
    parameters.extend((0..16).map(|index| format!("unused{index} :: Int64")));
    let mut arguments = vec!["value".to_owned()];
    arguments.extend(std::iter::repeat_n("0i64".to_owned(), 16));
    let source = format!(
        "select :: ({}) -> Symbol := \\({}) {{ value }};\n\
         main :: Unit -> Int32 := \\() {{\n\
           value := \"a\" + \"b\";\n\
           bundle := ({});\n\
           result := select(bundle);\n\
           if (result == \"ab\") then {{ 0 }} else {{ 1 }};\n\
         }};",
        types.join(", "),
        parameters.join(", "),
        arguments.join(", ")
    );
    let generated = emit(&source).expect("emit an owned aggregate entry fallback");

    assert!(!generated.source.contains("mal_direct_function_0"));
    assert!(generated.source.contains("mal_owned_function_0("));
    let fixture = NativeFixture::new("owned-large-product-entry");
    let executable = fixture.compile_generated_with_options(
        generated,
        "",
        &[
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
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(1));
    mal_SymbolAdmission_data(&admission)[0] = bytes[0];
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(1));
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
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, UINT64_C(1));
    mal_SymbolAdmission_data(&admission)[0] = bytes[0];
    return mal_SymbolAdmission_finish(context, &admission, UINT64_C(1));
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
