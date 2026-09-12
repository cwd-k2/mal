use super::*;

#[test]
fn runs_receiver_first_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-receiver-first-call");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "add :: (Int32, Int32) -> Int32 := (left, right) { left + right; };\n\
         main :: Unit -> Int32 := () { 40i32.add(1i32).add(1i32) - 42i32; };",
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
