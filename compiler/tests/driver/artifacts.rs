use super::*;

#[test]
fn builds_a_constant_main_through_the_llvm_artifact_set() {
    let directory = NativeFixture::new("driver-llvm");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write("program.mal", "main :: Unit -> Int32 := () { 7; };");

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(7));
}

#[test]
fn retains_artifacts_uses_the_generated_header_and_forwards_clang_arguments() {
    let directory = NativeFixture::new("driver-retained-artifacts");
    let source = directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern sine :: Float64 -> Float64;\n\
         main :: Unit -> Int32 := () { if (sine(0.0) == 0.0) then { 0 } else { 1 }; };",
    );
    directory.write(
        "program.mal.h",
        "#ifndef MAL_PROGRAM_MAL_H\n\
         #define MAL_PROGRAM_MAL_H\n\
         #error stale adjacent header must not be used\n\
         #endif\n",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         #include <math.h>\n\
         static volatile double zero;\n\
         MAL_DEFINE_sine(call, value) {\n\
           return mal_Float64_return(call, sin(value + zero));\n\
         }\n",
    );
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        artifacts.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-fno-builtin-sin"),
        OsStr::new("--clang-arg"),
        OsStr::new("-lm"),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
    assert!(
        std::fs::read_to_string(artifacts.join("program.ll"))
            .unwrap()
            .contains("target triple")
    );
    assert!(
        std::fs::read_to_string(artifacts.join("program-shim.c"))
            .unwrap()
            .contains("mal_bridge_external_0")
    );
    assert!(
        std::fs::read_to_string(artifacts.join("program.mal.h"))
            .unwrap()
            .contains("MAL_DEFINE_sine")
    );
    for runtime in [
        "runtime.h",
        "core.c",
        "control.c",
        "symbol.c",
        "symbol_internal.h",
    ] {
        assert!(artifacts.join(runtime).is_file(), "missing {runtime}");
    }
}

#[test]
fn references_closed_top_level_numeric_constants_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-top-level-constant");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "answer :: UInt64 := UInt64(42);\n\
         main :: Unit -> Int32 := () { Int32(answer) - 42; };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn references_structural_closed_top_level_values_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-structural-top-level");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Unit, Symbol];\n\
         (number, text) :: (Int32, Symbol) := (-7i32, \"ok\");\n\
         choice :: Choice := 1[Choice](\"yes\");\n\
         enabled :: Bool := true;\n\
         reader :: Ptr -> Int64 := Int64.load;\n\
         main :: Unit -> Int32 := () {\n\
           choice[\n\
             () { 1 },\n\
             (value) {\n\
               if (enabled) then {\n\
                 if (number == -7i32) then {\n\
                   if (text == \"ok\" && value == \"yes\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             }]\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn passes_process_arguments_through_the_llvm_entry_bridge() {
    let directory = NativeFixture::new("driver-llvm-arguments");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Arguments :: (UInt64, Ptr);\n\
         argumentAt :: (Ptr, UInt64) -> Symbol := (arguments, index) {\n\
           slot := arguments + index * (Ptr.size + UInt64.size);\n\
           Symbol.read(Ptr.load(slot), UInt64.load(slot + Ptr.size));\n\
         };\n\
         main :: Arguments -> Int32 := (count, arguments) {\n\
           first := argumentAt(arguments, 0u64);\n\
           second := argumentAt(arguments, 1u64);\n\
           if (count == 2u64) then {\n\
             if (first == \"alpha\") then {\n\
               if (second == \"\") then { 0 } else { 1 };\n\
             } else { 2 };\n\
           } else { 3 };\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = std::process::Command::new(executable)
        .args(["alpha", ""])
        .output()
        .expect("run argument-aware LLVM executable");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn builds_scalar_control_and_tail_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-control");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "increment :: Int32 -> Int32 := (value) { value + 1; };\n\
         countdown :: Int32 -> Int32 := (value) {\n\
           if (value == 0) then { increment(value) } else { countdown(value - 1) };\n\
         };\n\
         main :: Unit -> Int32 := () { countdown(100000); };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(1));
}

#[test]
fn calls_capture_free_first_class_functions_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-indirect-call");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) {\n\
           operation(value);\n\
         };\n\
         increment :: Int32 -> Int32 := (value) { value + 1i32; };\n\
         main :: Unit -> Int32 := () { apply(increment, 41i32) - 42i32; };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn calls_first_class_memory_functions_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-memory-function");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Reader :: Ptr -> Int64;\n\
         Writer :: (Ptr, Int64) -> Unit;\n\
         extern memory :: Unit -> Ptr;\n\
         readWith :: (Reader, Ptr) -> Int64 := (reader, pointer) { reader(pointer); };\n\
         writeWith :: (Writer, Ptr, Int64) -> Unit := (writer, pointer, value) { writer(pointer, value); };\n\
         main :: Unit -> Int32 := () {\n\
           pointer := memory();\n\
           writeWith(Int64.store, pointer, 42i64);\n\
           Int32(readWith(Int64.load, pointer) - 42i64);\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static unsigned char storage[8];\n\
         MAL_DEFINE_memory(call) { return mal_Ptr_return(call, storage); }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn calls_a_memory_target_from_an_indirect_recursive_region_site() {
    let directory = NativeFixture::new("driver-llvm-region-memory-target");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Reader :: Ptr -> Int64;\n\
         extern memory :: Unit -> Ptr;\n\
         apply :: (Reader, Ptr) -> Int64 := (reader, pointer) { reader(pointer); };\n\
         recurse :: Ptr -> Int64 := (pointer) { apply(recurse, pointer); };\n\
         main :: Unit -> Int32 := () {\n\
           pointer := memory();\n\
           Int64.store(pointer, 42i64);\n\
           Int32(apply(Int64.load, pointer) - 42i64);\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static unsigned char storage[8];\n\
         MAL_DEFINE_memory(call) { return mal_Ptr_return(call, storage); }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn returns_a_managed_native_result_from_a_tail_only_recursive_region() {
    let directory = NativeFixture::new("driver-llvm-tail-region-native-target");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern touch :: Unit -> Unit;\n\
         apply :: ((Int32 -> Symbol), Int32) -> Symbol := (operation, value) {\n\
           touch();\n\
           operation(value);\n\
         };\n\
         recurse :: Int32 -> Symbol := (value) { apply(recurse, value); };\n\
         main :: Unit -> Int32 := () {\n\
           prefix := \"x\" + \"y\";\n\
           identity :: Int32 -> Symbol := (value) { prefix; };\n\
           result := apply(identity, 0i32);\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_touch(call) { return mal_Unit_return(call); }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runs_deep_first_class_call_cycles_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-first-class-cycle");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) { operation(value); };\n\
         main :: Unit -> Int32 := () {\n\
           recurse :: Int32 -> Int32 := (value) {\n\
             if (value == 0i32)\n\
             then { 0i32 }\n\
             else {\n\
               child := apply(recurse, value - 1i32);\n\
               child + 1i32;\n\
             };\n\
           };\n\
           recurse(300000i32) - 300000i32;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn preserves_managed_environments_through_llvm_first_class_cycles() {
    let directory = NativeFixture::new("driver-llvm-managed-first-class-cycle");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Symbol), Int32) -> Symbol := (operation, value) { operation(value); };\n\
         main :: Unit -> Int32 := () {\n\
           prefix := \"x\" + \"y\";\n\
           recurse :: Int32 -> Symbol := (value) {\n\
             if (value == 0i32)\n\
             then { prefix }\n\
             else {\n\
               child := apply(recurse, value - 1i32);\n\
               if (child == prefix)\n\
               then { child }\n\
               else { \"bad\" };\n\
             };\n\
           };\n\
           result := recurse(50000i32);\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn dispatches_all_recursive_closure_targets_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-recursive-targets");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) { operation(value); };\n\
         main :: Unit -> Int32 := () {\n\
           left :: Int32 -> Int32 := (value) {\n\
             if (value == 0i32)\n\
             then { 0i32 }\n\
             else { child := apply(left, value - 1i32); child + 1i32; };\n\
           };\n\
           right :: Int32 -> Int32 := (value) {\n\
             if (value == 0i32)\n\
             then { 0i32 }\n\
             else { child := apply(right, value - 1i32); child + 1i32; };\n\
           };\n\
           left(100000i32) + right(100000i32) - 200000i32;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn runs_continuations_when_value_and_answer_function_types_overlap() {
    let directory = NativeFixture::new("driver-llvm-cont-overlapping-functions");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Answer :: Int32;\n\
         Continuation :: Int32 -> Answer;\n\
         Computation :: Continuation -> Answer;\n\
         Next :: Int32 -> Computation;\n\
         Mapper :: Int32 -> Int32;\n\
         pure :: Int32 -> Computation := (value) {\n\
           (continuation) { value[continuation] };\n\
         };\n\
         bind :: (Computation, Next) -> Computation := (computation, next) {\n\
           (continuation) {\n\
             resume :: Continuation := (value) { continuation[value[next]]; };\n\
             resume[computation];\n\
           };\n\
         };\n\
         map :: (Computation, Mapper) -> Computation := (computation, mapper) {\n\
           next :: Next := (value) { value[mapper][pure]; };\n\
           (computation, next)[bind];\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           mapped := (10[pure], (value) { value * 2 })[map];\n\
           (value) { value - 20 }[mapped];\n\
         };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn runs_call_cc_encoded_with_capturing_closures() {
    let directory = NativeFixture::new("driver-llvm-cont-call-cc");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Answer :: Int64;\n\
         Continuation :: Int32 -> Answer;\n\
         Computation :: Continuation -> Answer;\n\
         Escape :: Int32 -> Computation;\n\
         CallCcBody :: Escape -> Computation;\n\
         callCc :: CallCcBody -> Computation := (body) {\n\
           (continuation) {\n\
             escape :: Escape := (value) { (_) { value[continuation] }; };\n\
             continuation[escape[body]];\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           body :: CallCcBody := (escape) {\n\
             (_) { (value) { Int64(value) }[42[escape]] };\n\
           };\n\
           computation := body[callCc];\n\
           Int32((value) { Int64(value) }[computation] - 42i64);\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         #include <stdlib.h>\n\
         static unsigned long live_allocations;\n\
         void *__real_malloc(size_t size);\n\
         void *__real_realloc(void *allocation, size_t size);\n\
         void __real_free(void *allocation);\n\
         void *__wrap_malloc(size_t size) {\n\
           void *allocation = __real_malloc(size);\n\
           if (allocation != NULL) { ++live_allocations; }\n\
           return allocation;\n\
         }\n\
         void *__wrap_realloc(void *allocation, size_t size) {\n\
           void *resized = __real_realloc(allocation, size);\n\
           if (resized != NULL && allocation == NULL) { ++live_allocations; }\n\
           return resized;\n\
         }\n\
         void __wrap_free(void *allocation) {\n\
           if (allocation != NULL) { --live_allocations; }\n\
           __real_free(allocation);\n\
         }\n\
         __attribute__((destructor)) static void check_allocations(void) {\n\
           if (live_allocations != 0) { _Exit(99); }\n\
         }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=malloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=realloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=free"),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn owns_capturing_closure_environments_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-capturing-closure");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () {\n\
           base :: Int32 := 40;\n\
           add :: Int32 -> Int32 := (value) { base + value; };\n\
           add(2) - 42;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn calls_escaping_closures_with_managed_captures_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-escaping-closure");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "makePrefix :: Symbol -> (Symbol -> Symbol) := (prefix) {\n\
           append :: Symbol -> Symbol := (suffix) { prefix + suffix; };\n\
           append;\n\
         };\n\
         apply :: ((Symbol -> Symbol), Symbol) -> Symbol := (operation, value) {\n\
           operation(value);\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           append := makePrefix(\"a\" + \"b\");\n\
           result := apply(append, \"c\" + \"d\");\n\
           if (result == \"abcd\") then { 0 } else { 1 };\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn builds_deep_non_tail_self_recursion_with_a_c_runtime_arena() {
    let directory = NativeFixture::new("driver-llvm-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "sum :: Int32 -> Int32 := (value) {\n\
           if (value == 0) then { 0 } else {\n\
             rest := sum(value - 1);\n\
             value + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { sum(10000) - 50005000; };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
    let module = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
    assert!(!module.contains("mal_invalid_frame_"));
}

#[test]
fn resumes_single_constructor_frames_without_live_payloads() {
    let directory = NativeFixture::new("driver-llvm-empty-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "countdown :: Int32 -> Int32 := (value) {\n\
           if (value == 0i32)\n\
           then { 0i32 }\n\
           else {\n\
             ignored := countdown(value - 1i32);\n\
             0i32;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { countdown(100000i32); };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn preserves_outer_frames_across_a_nested_recursive_region() {
    let directory = NativeFixture::new("driver-llvm-nested-control-regions");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "inner :: Int64 -> Int64 := (n) {\n\
           if (n == 0i64)\n\
           then { 0i64 }\n\
           else {\n\
             rest := inner(n - 1i64);\n\
             n + rest;\n\
           };\n\
         };\n\
         outer :: Int32 -> Int64 := (n) {\n\
           if (n == 0i32)\n\
           then { inner(100i64) }\n\
           else {\n\
             rest := outer(n - 1i32);\n\
             Int64(n) + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { Int32(outer(20i32)) - 5260i32; };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn resumes_frames_when_a_recursive_branch_ends_in_a_direct_tail_call() {
    let directory = NativeFixture::new("driver-llvm-frame-tail-exit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "finish :: Int32 -> Int32 := (value) { value + 1i32; };\n\
         unwind :: Int32 -> Int32 := (value) {\n\
           if (value == 0i32) then { finish(0i32) } else {\n\
             child := unwind(value - 1i32);\n\
             finish(child);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { unwind(100000i32) - 100001i32; };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn dispatches_multiple_typed_self_continuation_frames_in_llvm() {
    let directory = NativeFixture::new("driver-llvm-frames");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "walk :: Int32 -> Int32 := (value) {\n\
           if (value == 0) then { 0 } else {\n\
             if (value == 1) then {\n\
               rest := walk(value - 1);\n\
               rest + 1;\n\
             } else {\n\
               rest := walk(value - 1);\n\
               rest + 1;\n\
             };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { walk(10000) - 10000; };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
    let module = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
    assert!(module.contains("mal_invalid_frame_"));
    assert!(module.contains("i32 0, label %mal_frame_"));
    assert!(module.contains("i32 1, label %mal_frame_"));
}

#[test]
fn calls_a_native_target_from_an_indirect_recursive_region_site() {
    let directory = NativeFixture::new("driver-llvm-region-native-target");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) {\n\
           called := operation(value);\n\
           called + 0i32;\n\
         };\n\
         identity :: Int32 -> Int32 := (value) { value + 1i32; };\n\
         recurse :: Int32 -> Int32 := (value) {\n\
           if (value == 0i32)\n\
           then { 0i32 }\n\
           else {\n\
             child := apply(recurse, value - 1i32);\n\
             child + 1i32;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { apply(identity, 41i32) - 42i32; };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn transfers_a_managed_native_target_result_back_into_a_recursive_region() {
    let directory = NativeFixture::new("driver-llvm-region-managed-native-target");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Packet :: (Int32, Symbol);\n\
         apply :: ((Packet -> Packet), Packet) -> Packet := (operation, value) {\n\
           operation(value);\n\
         };\n\
         identity :: Packet -> Packet := (value) { value; };\n\
         recurse :: Packet -> Packet := (value) {\n\
           (remaining, text) := value;\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else {\n\
             child := apply(recurse, (remaining - 1i32, text));\n\
             (next, result) := child;\n\
             (next + 1i32, result + \"!\");\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           seed := \"a\" + \"b\";\n\
           (_, result) := apply(identity, (0i32, seed));\n\
           if (result == \"ab\")\n\
           then { 0i32 }\n\
           else { 1i32 };\n\
         };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn resumes_managed_self_continuation_frames_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-managed-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "walk :: (Int32, Symbol) -> Symbol := (depth, value) {\n\
           if (depth == 0i32) then { value } else {\n\
             resumed := walk(depth - 1i32, value);\n\
             if (resumed # 0u64 == 120u8) then { resumed } else { \"bad\" };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(10000i32, seed);\n\
           Int32(result # 1u64) - 121i32;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn runs_managed_direct_self_tail_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-managed-tail");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "count :: (Symbol, Int64) -> UInt64 := (value, remaining) {\n\
           if (remaining == 0i64) then { #value }\n\
           else { count(value, remaining - 1i64) };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           seed := \"x\" + \"y\";\n\
           if (count(seed, 100000i64) == 2u64) then { 0 } else { 1 };\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn hands_direct_self_arguments_to_wildcard_parameters() {
    let directory = NativeFixture::new("driver-llvm-self-wildcard");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern again :: Unit -> Bool;\n\
         make :: Int32 -> (Unit -> Int32) := (value) { () { value }; };\n\
         unmanagedFrame :: Int32 -> Int32 := (_) {\n\
           if (again()) then { child := unmanagedFrame(1i32); child + 1i32; }\n\
           else { 0i32 };\n\
         };\n\
         managedFrame :: (Unit -> Int32) -> Int32 := (_) {\n\
           if (again()) then { child := managedFrame(make(1i32)); child + 1i32; }\n\
           else { 0i32 };\n\
         };\n\
         unmanagedTail :: Int32 -> Int32 := (_) {\n\
           if (again()) then { unmanagedTail(1i32) } else { 0i32 };\n\
         };\n\
         managedTail :: (Unit -> Int32) -> Int32 := (_) {\n\
           if (again()) then { managedTail(make(1i32)) } else { 0i32 };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           framed := unmanagedFrame(0i32) + managedFrame(make(0i32));\n\
           framed + unmanagedTail(0i32) + managedTail(make(0i32)) - 2i32;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         #include <stdlib.h>\n\
         static unsigned long calls;\n\
         static unsigned long live_allocations;\n\
         void *__real_malloc(size_t size);\n\
         void *__real_realloc(void *allocation, size_t size);\n\
         void __real_free(void *allocation);\n\
         void *__wrap_malloc(size_t size) {\n\
           void *allocation = __real_malloc(size);\n\
           if (allocation != NULL) { ++live_allocations; }\n\
           return allocation;\n\
         }\n\
         void *__wrap_realloc(void *allocation, size_t size) {\n\
           void *resized = __real_realloc(allocation, size);\n\
           if (resized != NULL && allocation == NULL) { ++live_allocations; }\n\
           return resized;\n\
         }\n\
         void __wrap_free(void *allocation) {\n\
           if (allocation != NULL) { --live_allocations; }\n\
           __real_free(allocation);\n\
         }\n\
         __attribute__((destructor)) static void check_allocations(void) {\n\
           if (live_allocations != 0) { _Exit(99); }\n\
         }\n\
         MAL_DEFINE_again(call) {\n\
           ++calls;\n\
           return mal_Bool_return(call, (calls & 1UL) != 0 ? mal_true : mal_false);\n\
         }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=malloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=realloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=free"),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn builds_every_integer_width_with_signed_and_unsigned_llvm_comparisons() {
    let directory = NativeFixture::new("driver-llvm-integers");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "signed8 :: Int8 -> Int32 := (value) {\n\
           next := value + 1i8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed16 :: Int16 -> Int32 := (value) {\n\
           next := value + 1i16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed64 :: Int64 -> Int32 := (value) {\n\
           next := value + 1i64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned8 :: UInt8 -> Int32 := (value) {\n\
           next := value + 1u8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned16 :: UInt16 -> Int32 := (value) {\n\
           next := value + 1u16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned32 :: UInt32 -> Int32 := (value) {\n\
           next := value + 1u32; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned64 :: UInt64 -> Int32 := (value) {\n\
           next := value + 1u64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signedOps :: Int64 -> Int32 := (value) {\n\
           quotient := value / 2i64;\n\
           remainder := value % 2i64;\n\
           shifted := (value << 1i64) >> 1i64;\n\
           if (quotient == -4i64) then {\n\
             if (remainder == -1i64) then {\n\
               if (shifted == value) then { 1 } else { 0 };\n\
             } else { 0 };\n\
           } else { 0 };\n\
         };\n\
         unsignedOps :: UInt64 -> Int32 := (value) {\n\
           shifted := (value << 1u64) >> 1u64;\n\
           remainder := shifted % 3u64;\n\
           if (remainder == 1u64) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           signed8(127i8) + signed16(32767i16) + signed64(9223372036854775807i64) +\n\
           unsigned8(255u8) + unsigned16(65535u16) +\n\
           unsigned32(4294967295u32) + unsigned64(18446744073709551615u64) +\n\
           signedOps(-9i64) + unsignedOps(10u64) - 9 +\n\
           Int32(UInt64(-1i8)) + 1;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn builds_strict_float_arithmetic_and_nan_comparisons_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-float");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "check32 :: Float32 -> Int32 := (value) {\n\
           result := value * 2.0f32 + 0.5f32;\n\
           if (result == 3.5f32) then { 1 } else { 0 };\n\
         };\n\
         check64 :: Float64 -> Int32 := (value) {\n\
           result := -(value / 2.0f64);\n\
           if (result <= -0.75f64) then { 1 } else { 0 };\n\
         };\n\
         checkNaN :: Float64 -> Int32 := (value) {\n\
           zero := value - value;\n\
           nan := zero / zero;\n\
           if (nan != nan) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           check32(1.5f32) + check64(1.5f64) + checkNaN(1.0f64) +\n\
           Int32(Float64(3)) + Int32(Float32(1.75f64)) - 7;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn resumes_mixed_numeric_scalar_frames_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-scalar-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "sum :: Float64 -> Float64 := (value) {\n\
           if (value == 0.0f64) then { 0.0f64 } else {\n\
             narrow := Int16(value);\n\
             wide := UInt64(value);\n\
             rest := sum(value - 1.0f64);\n\
             rest + Float64(narrow) + Float64(wide);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () { Int32(sum(10000.0f64) - 100010000.0f64); };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn constructs_and_resumes_unmanaged_products_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-product");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "build :: Int32 -> (Int16, UInt64) := (remaining) {\n\
           if (remaining == 0) then { (0i16, 0u64) } else {\n\
             (narrow, wide) := build(remaining - 1);\n\
             (narrow + 1i16, wide + 1u64);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           (narrow, wide) := build(10000);\n\
           Int32(narrow) + Int32(wide) - 20000;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn branches_over_bool_and_unmanaged_sums_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-sum");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Int16, (UInt32, UInt64)];\n\
         choose :: Bool -> Choice := (flag) {\n\
           if (flag) then { 1[Choice]((20u32, 22u64)) } else { 0[Choice](42i16) };\n\
         };\n\
         score :: Choice -> Int32 := (choice) {\n\
           choice[\n\
             (value) { Int32(value) },\n\
             (pair) { (left, right) := pair; Int32(left) + Int32(right) }];\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           flag := true != false;\n\
           score(choose(false)) + score(choose(flag)) - 84;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn runs_first_class_sum_constructors_and_postfix_application_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-first-class-sum-constructor");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Unit, Int32];\n\
         none :: Unit -> Choice := 0[Choice];\n\
         some :: Int32 -> Choice := 1[Choice];\n\
         score :: Choice -> Int32 := (choice) {\n\
           choice[\n\
           () { 0 },\n\
           (value) { value }\n\
           ];\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           score([none]) + score(41[some]) - 41;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn preserves_short_circuit_effect_order_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-short-circuit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern forbidden :: Unit -> Bool;\n\
         main :: Unit -> Int32 := () {\n\
           if (false && forbidden()) then { 1 } else { 0 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_forbidden(call) {\n\
             mal_call_trap(call, \"short-circuit operand was evaluated\");\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn accesses_unaligned_scalar_and_pointer_storage_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-memory");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern memory :: UInt64 -> Ptr;\n\
         main :: Unit -> Int32 := () {\n\
           base := memory(64u64);\n\
           UInt64.store(base, 42u64);\n\
           pointerSlot := base + UInt64.size;\n\
           Ptr.store(pointerSlot, base);\n\
           floatSlot := pointerSlot + Ptr.size;\n\
           Float32.store(floatSlot, 1.5f32);\n\
           restored := Ptr.load(pointerSlot);\n\
           start := floatSlot - Ptr.size - UInt64.size;\n\
           value := UInt64.load(restored) + UInt64.load(start);\n\
           if (Float32.load(floatSlot) == 1.5f32) then { Int32(value) - 84 } else { 1 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static unsigned char storage[65];\n\
         MAL_DEFINE_memory(call, size) {\n\
             (void)size;\n\
             return mal_Ptr_return(call, storage + 1);\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn owns_symbols_across_direct_llvm_calls() {
    let directory = NativeFixture::new("driver-llvm-symbol");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "check :: Symbol -> Int32 := (value) {\n\
           if (#value == 2u64)\n\
           then {\n\
             if (value == \"ab\")\n\
             then {\n\
               if (value != \"ac\")\n\
               then { 0 }\n\
               else { 1 };\n\
             }\n\
             else { 2 };\n\
           }\n\
           else { 3 };\n\
         };\n\
         main :: Unit -> Int32 := () {
           joined := \"a\" + \"b\";
           extended := joined + \"c\";
           if (check(joined) == 0i32)
           then {
             if (extended == \"abc\")
             then { 0i32 }
             else { 4i32 };
           }
           else { 5i32 };
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn balances_persistent_symbols_and_materializes_only_at_the_host_boundary() {
    let directory = NativeFixture::new("driver-llvm-symbol-rope");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern inspect :: Symbol -> UInt64;\n\
         extern allocationCount :: Unit -> UInt64;\n\
         append :: (Int32, Symbol) -> Symbol := (remaining, value) {\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else { append(remaining - 1i32, value + \"x\") };\n\
         };\n\
         prepend :: (Int32, Symbol) -> Symbol := (remaining, value) {\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else { prepend(remaining - 1i32, \"x\" + value) };\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           left := append(10000i32, \"\");\n\
           right := prepend(10000i32, \"\");\n\
           middle := \"a\" + \"b\";\n\
           prefixed := \"x\" + middle;\n\
           mixed := prefixed + \"c\";\n\
           if (mixed == \"xabc\")\n\
           then {\n\
             if (left == right)\n\
             then {\n\
               if (left # 9999u64 == 120u8)\n\
               then {\n\
                 if (inspect(left) == 10000u64)\n\
                 then {\n\
                   if (allocationCount() <= 32u64)\n\
                   then { 0i32 }\n\
                   else { 3i32 };\n\
                 }\n\
                 else { 4i32 };\n\
               }\n\
               else { 2i32 };\n\
             }\n\
             else { 1i32 };\n\
           }\n\
           else { 5i32 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         #include <stdlib.h>\n\
         static uint64_t live_allocations;\n\
         static uint64_t total_allocations;\n\
         void *__real_malloc(size_t size);\n\
         void *__real_realloc(void *allocation, size_t size);\n\
         void __real_free(void *allocation);\n\
         void *__wrap_malloc(size_t size) {\n\
             void *allocation = __real_malloc(size);\n\
             if (allocation != NULL) {\n\
                 ++live_allocations;\n\
                 ++total_allocations;\n\
             }\n\
             return allocation;\n\
         }\n\
         void *__wrap_realloc(void *allocation, size_t size) {\n\
             void *resized = __real_realloc(allocation, size);\n\
             if (resized != NULL) {\n\
                 if (allocation == NULL) {\n\
                     ++live_allocations;\n\
                 }\n\
                 ++total_allocations;\n\
             }\n\
             return resized;\n\
         }\n\
         void __wrap_free(void *allocation) {\n\
             if (allocation != NULL) {\n\
                 --live_allocations;\n\
             }\n\
             __real_free(allocation);\n\
         }\n\
         __attribute__((destructor)) static void check_allocations(void) {\n\
             if (live_allocations != 0) {\n\
                 _Exit(99);\n\
             }\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             mal_span_t bytes = mal_Symbol_to_bytes(call, value);\n\
             for (uint64_t index = 0; index < bytes.length; ++index) {\n\
                 if (bytes.data[index] != 'x') {\n\
                     return mal_UInt64_return(call, UINT64_C(0));\n\
                 }\n\
             }\n\
             return mal_UInt64_return(call, bytes.length);\n\
         }\n\
         MAL_DEFINE_allocationCount(call) {\n\
             return mal_UInt64_return(call, total_allocations);\n\
         }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        artifacts.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=malloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=realloc"),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=free"),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
    let module = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
    assert!(module.contains("call ptr @mal_runtime_symbol_concatenate_consuming_left"));
    assert!(module.contains("call ptr @mal_runtime_symbol_concatenate_consuming_right"));
}

#[test]
fn derives_symbol_runtime_dependencies_from_symbol_operations() {
    let closure_directory = NativeFixture::new("driver-llvm-closure-runtime-dependencies");
    let closure_source = closure_directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) {\n\
           operation(value);\n\
         };\n\
         main :: Unit -> Int32 := () { apply((value) { value; }, 0i32); };",
    );
    let closure_executable = closure_directory.join("program");
    let closure_artifacts = closure_directory.join("artifacts");
    let closure_output = closure_directory.malc([
        OsStr::new("build"),
        closure_source.as_os_str(),
        OsStr::new("--output"),
        closure_executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        closure_artifacts.as_os_str(),
    ]);
    assert!(
        closure_output.status.success(),
        "{}",
        String::from_utf8_lossy(&closure_output.stderr)
    );
    let closure_module = std::fs::read_to_string(closure_artifacts.join("program.ll")).unwrap();
    assert!(!closure_module.contains("mal_runtime_symbol_"));

    let symbol_directory = NativeFixture::new("driver-llvm-discarded-symbol-operation");
    let symbol_source = symbol_directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () { discarded := \"a\" + \"b\"; 0i32; };",
    );
    let symbol_executable = symbol_directory.join("program");
    let symbol_artifacts = symbol_directory.join("artifacts");
    let symbol_output = symbol_directory.malc([
        OsStr::new("build"),
        symbol_source.as_os_str(),
        OsStr::new("--output"),
        symbol_executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        symbol_artifacts.as_os_str(),
    ]);
    assert!(
        symbol_output.status.success(),
        "{}",
        String::from_utf8_lossy(&symbol_output.stderr)
    );
    let symbol_module = std::fs::read_to_string(symbol_artifacts.join("program.ll")).unwrap();
    assert!(symbol_module.contains("declare ptr @mal_runtime_symbol_concatenate"));
}

#[test]
fn reuses_owned_symbols_across_empty_concatenation() {
    let directory = NativeFixture::new("driver-llvm-symbol-empty-concatenation");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () {\n\
           value := \"a\" + \"b\";\n\
           left := \"\" + value;\n\
           right := value + \"\";\n\
           if (left == right)\n\
           then { Int32(value # 1u64) - 98i32 }\n\
           else { 1i32 };\n\
         };",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn bridges_symbol_parameters_and_results_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-symbol-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern inspect :: Symbol -> UInt8;\n\
         extern fetch :: Unit -> Symbol;\n\
         main :: Unit -> Int32 := () {\n\
           seed := \"x\" + \"y\";\n\
           if (inspect(seed) == 1u8) then { Int32(fetch() # 1u64) - 107 }\n\
           else { 1 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_inspect(call, value) {\n\
             mal_span_t bytes = mal_Symbol_to_bytes(call, value);\n\
             return (uint8_t)(bytes.length == 2 && bytes.data[0] == 'x' && bytes.data[1] == 'y');\n\
         }\n\
         MAL_DEFINE_fetch(call) {\n\
             static const uint8_t bytes[] = {'o', 'k'};\n\
             return mal_Symbol_return(\n\
                 call,\n\
                 mal_Symbol_from_bytes((mal_span_t){.data = bytes, .length = sizeof(bytes)})\n\
             );\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn marshals_managed_products_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-product-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Packet :: (UInt64, Symbol);\n\
         extern exchange :: Packet -> Packet;\n\
         main :: Unit -> Int32 := () {\n\
           (number, text) := exchange(41u64, \"a\" + \"b\");\n\
           if (number == 42u64) then {\n\
             if (text == \"ab\") then { 0 } else { 1 };\n\
           } else { 2 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_exchange(call, value) {\n\
             mal_span_t bytes = mal_Symbol_to_bytes(call, value.field_1);\n\
             if (bytes.length != 2 || bytes.data[0] != 'a' || bytes.data[1] != 'b') {\n\
                 mal_call_trap(call, \"unexpected packet\");\n\
             }\n\
             return mal_Packet_return(\n\
                 call,\n\
                 (mal_Packet_t){.field_0 = value.field_0 + 1, .field_1 = value.field_1}\n\
             );\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn marshals_active_sum_payloads_recursively_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-sum-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Choice :: [Unit, (UInt64, Symbol)];\n\
         Envelope :: (UInt8, Choice);\n\
         extern exchange :: Envelope -> Envelope;\n\
         main :: Unit -> Int32 := () {\n\
           (number, choice) := exchange(41u8, 1[Choice]((7u64, \"a\" + \"b\")));\n\
           choice[\n\
             () { 1 },\n\
             (packet) {\n\
               (bias, text) := packet;\n\
               if (number == 42u8) then {\n\
                 if (bias == 7u64) then {\n\
                   if (text == \"ab\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             }];\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.field_1.tag != mal_Choice_tag_1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_span_t bytes = mal_Symbol_to_bytes(\n\
                 call, value.field_1.payload.variant_1.field_1\n\
             );\n\
             if (bytes.length != 2 || bytes.data[0] != 'a' || bytes.data[1] != 'b') {\n\
                 mal_call_trap(call, \"unexpected payload\");\n\
             }\n\
             return mal_Envelope_return(\n\
                 call,\n\
                 (mal_Envelope_t){.field_0 = value.field_0 + 1, .field_1 = value.field_1}\n\
             );\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn transfers_external_opaque_values_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-opaque-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern Handle;\n\
         Packet :: (UInt8, Handle);\n\
         Choice :: [Unit, Packet];\n\
         extern create :: UInt64 -> Handle;\n\
         extern exchange :: Choice -> Choice;\n\
         extern inspect :: Handle -> UInt64;\n\
         main :: Unit -> Int32 := () {\n\
           choice := exchange(1[Choice]((1u8, create(40u64))));\n\
           choice[\n\
             () { 1 },\n\
             (packet) {\n\
               (bias, handle) := packet;\n\
               Int32(inspect(handle) + UInt64(bias) - 42u64);\n\
             }];\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_create(call, value) {\n\
             return mal_Handle_return(call, mal_Handle_from_bits((uintptr_t)value));\n\
         }\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.tag != mal_Choice_tag_1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_Packet_t packet = value.payload.variant_1;\n\
             uintptr_t bits = mal_Handle_to_bits(packet.field_1);\n\
             return mal_Choice_return_1(\n\
                 call,\n\
                 (mal_Packet_t){\n\
                     .field_0 = packet.field_0,\n\
                     .field_1 = mal_Handle_from_bits(bits + (uintptr_t)1),\n\
                 }\n\
             );\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             return mal_UInt64_return(call, (uint64_t)mal_Handle_to_bits(value));\n\
         }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn owns_symbols_nested_in_products_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-symbol-product");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "inspect :: (Symbol, UInt64) -> (Symbol, UInt8) := (input) {\n\
           (value, index) := input;\n\
           (value, value # index);\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           joined := \"ab\" + \"cd\";\n\
           (copy, byte) := inspect(joined, 2u64);\n\
           if (copy == \"abcd\") then { Int32(byte) - 99 } else { 1 };\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn retains_only_active_managed_sum_payloads_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-symbol-sum");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Symbol, (UInt64, Symbol)];\n\
         choose :: Bool -> Choice := (second) {\n\
           if (second) then { 1[Choice]((2u64, \"b\" + \"c\")) }\n\
           else { 0[Choice](\"a\" + \"b\") };\n\
         };\n\
         score :: Choice -> Int32 := (choice) {\n\
           choice[\n\
             (value) { Int32(value # 0u64) },\n\
             (pair) {\n\
               (bias, value) := pair;\n\
               Int32(bias) + Int32(value # 1u64);\n\
             }];\n\
         };\n\
         main :: Unit -> Int32 := () {\n\
           score(choose(false)) + score(choose(true)) - 198;\n\
         };",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn emit_header_writes_a_standalone_host_interface() {
    let directory = NativeFixture::new("driver-header");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;",
    );

    let output = directory.malc([
        OsStr::new("emit-header"),
        source.as_os_str(),
        OsStr::new("--output"),
        output_path.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = std::fs::read_to_string(output_path).unwrap();
    assert!(header.contains("typedef MalType_UInt64 MalType_Count;"));
    assert!(header.contains("#define MAL_HAS_EXTERN_increment 1"));
    assert!(header.contains("#define MAL_DEFINE_increment(call, value)"));
    assert!(!header.contains("MAL_HAS_EXTERN_missing"));
    assert!(!directory.join("generated/program.c").exists());
}

#[test]
fn emit_header_defaults_to_the_source_directory() {
    let directory = NativeFixture::new("driver-default-header");
    let source = directory.join("source/program.mal");
    directory.write("source/program.mal", "extern print :: Symbol -> Unit;");

    let output = directory.malc([OsStr::new("emit-header"), source.as_os_str()]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = std::fs::read_to_string(directory.join("source/program.mal.h")).unwrap();
    assert!(header.contains("#define MAL_DEFINE_print(call, value)"));
}

#[test]
fn emit_host_prints_compilable_external_operation_stubs() {
    let directory = NativeFixture::new("driver-host");
    let source = directory.join("program.mal");
    let header = directory.join("custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         Request :: (Count, Int32);\n\
         extern increment :: Count -> Count;\n\
         extern inspect :: Request -> Count;\n\
         main :: Unit -> Int32 := () { Int32(increment(41u64) - 42u64); };",
    );

    let header_output = directory.malc([
        OsStr::new("emit-header"),
        source.as_os_str(),
        OsStr::new("--output"),
        header.as_os_str(),
    ]);
    assert!(header_output.status.success());
    let default_output = directory.malc([OsStr::new("emit-host"), source.as_os_str()]);
    assert!(default_output.status.success());
    assert!(
        default_output
            .stdout
            .starts_with(b"#include \"program.mal.h\"\n")
    );

    let output = directory.malc([
        OsStr::new("emit-host"),
        source.as_os_str(),
        OsStr::new("--header"),
        OsStr::new("custom.h"),
    ]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let host = String::from_utf8(output.stdout).unwrap();
    assert!(host.starts_with("#include \"custom.h\"\n"));
    assert!(host.contains("MAL_DEFINE_increment(call, value)"));
    assert!(host.contains("MAL_DEFINE_inspect(call, value)"));
    assert!(host.contains("(void)value;"));
    assert!(host.contains("external operation `increment` is not implemented"));

    directory.write("host.c", &host);
    let object = directory.join("host.o");
    let compilation = std::process::Command::new("clang")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic", "-c"])
        .arg(directory.join("host.c"))
        .arg("-o")
        .arg(object)
        .output()
        .expect("run clang");
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
}

#[test]
fn checked_in_example_headers_match_the_compiler() {
    let fixture = NativeFixture::new("example-headers");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler directory has a repository parent");
    let examples = [
        "fallible-tree",
        "integer-and-byte",
        "mini-database",
        "opaque-aggregate",
        "pointer-tree",
        "print-and-closure",
        "ptr-memory",
        "recoverable-file",
        "resizable-buffer",
        "socket-packet",
        "strict-float",
        "symbol-round-trip",
        "tail-recursion",
    ];

    for example in examples {
        let directory = repository.join("examples").join(example);
        let generated = fixture.join(format!("{example}.h"));
        let output = fixture.malc([
            OsStr::new("emit-header"),
            directory.join("program.mal").as_os_str(),
            OsStr::new("--output"),
            generated.as_os_str(),
        ]);
        assert!(
            output.status.success(),
            "failed to generate {example}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(generated).unwrap(),
            std::fs::read_to_string(directory.join("program.mal.h")).unwrap(),
            "checked-in header is stale for {example}"
        );
    }
}

#[test]
fn build_compiles_required_host_inputs_and_produces_an_executable() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let executable = directory.join("out/program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         require \"./host.c\";\n\
         require \"./helper.c\";\n\
         extern adjust :: Int32 -> Int32;\n\
         main :: Unit -> Int32 := () { adjust(40) - 42; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         int32_t host_increment(int32_t value);\n\
         MAL_DEFINE_adjust(call, value) {\n\
             return mal_Int32_return(call, host_increment(value));\n\
         }\n",
    );
    directory.write(
        "helper.c",
        "#include <stdint.h>\n\
         int32_t host_increment(int32_t value) { return value + 2; }\n",
    );

    let unavailable = directory.join("must-not-be-used");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable.as_os_str(),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn builds_public_functions_from_required_files_with_private_helpers() {
    let directory = NativeFixture::new("driver-required-mal");
    let source = directory.write(
        "program.mal",
        "require \"./left.mal\";\n\
         require \"./left.mal\";\n\
         require \"./right.mal\";\n\
         main :: Unit -> Int32 := () { left(39) + right(1) - 42 };",
    );
    directory.write(
        "left.mal",
        "_helper :: Int32 -> Int32 := (x) { x + 1 };\n\
         left :: Int32 -> Int32 := (x) { _helper(x) };",
    );
    directory.write(
        "right.mal",
        "_helper :: Int32 -> Int32 := (x) { x + 1 };\n\
         right :: Int32 -> Int32 := (x) { _helper(x) };",
    );
    let executable = directory.join("program");

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn source_graph_overlays_open_mal_buffers() {
    let directory = NativeFixture::new("driver-overlays");
    let source = directory.write("program.mal", "not the open buffer");
    let dependency = directory.join("library.mal");
    let root_text = "require \"library.mal\";\nmain :: Unit -> Int32 := () { value - 42; };";
    let dependency_text = "value :: Int32 := 42;";
    let overlays =
        std::collections::HashMap::from([(dependency.clone(), dependency_text.to_owned())]);

    let graph = malc::driver::load_source_graph_with_overlays(&source, root_text, &overlays)
        .expect("load overlaid source graph");

    assert_eq!(graph.root_source().text(), root_text);
    assert_eq!(graph.files().len(), 2);
    assert_eq!(
        graph.source(malc::source::FileId::new(1)).unwrap().text(),
        dependency_text
    );
    malc::pipeline::check_graph(&graph).expect("check overlaid graph");
}
