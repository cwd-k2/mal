use super::*;
#[test]
fn owns_symbols_across_direct_llvm_calls() {
    let directory = NativeFixture::new("driver-llvm-symbol");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "check :: Symbol -> Int32 := (value) -> {\n\
           if (#value == 2usize)\n\
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
         main :: Unit -> Int32 := () -> {
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
fn reuses_flat_symbol_storage_for_long_accumulations() {
    let directory = NativeFixture::new("driver-llvm-symbol-rope");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern allocationCount :: Unit -> UInt64;\n\
         append :: (Int32, Symbol) -> Symbol := (remaining, value) -> {\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else { append(remaining - 1i32, value + \"x\") };\n\
         };\n\
         prepend :: (Int32, Symbol) -> Symbol := (remaining, value) -> {\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else { prepend(remaining - 1i32, \"x\" + value) };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           left := append(10000i32, \"\");\n\
           right := prepend(10000i32, \"\");\n\
           middle := \"a\" + \"b\";\n\
           prefixed := \"x\" + middle;\n\
           mixed := prefixed + \"c\";\n\
           if (mixed == \"xabc\")\n\
           then {\n\
             if (left == right)\n\
             then {\n\
               if (left # 9999usize == 120u8)\n\
               then {\n\
                 if (#left == 10000usize)\n\
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
        OsStr::new("--optimization"),
        OsStr::new("production"),
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
    assert!(module.contains("call void @mal_runtime_symbol_concatenate_consuming_left"));
    assert!(module.contains("call void @mal_runtime_symbol_concatenate_consuming_right"));
}

#[test]
fn allocates_symbol_call_results_once_per_function_activation() {
    let directory = NativeFixture::new("driver-symbol-loop-result-storage");
    let source = directory.write(
        "program.mal",
        "churn :: (USize, Int64) -> USize := (total, remaining) -> {\n\
           if (remaining == 0i64)\n\
           then total\n\
           else {\n\
             value := \"left\" + \"right\";\n\
             churn(total + #value, remaining - 1i64);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           if (churn(0usize, 1000000i64) == 9000000usize)\n\
           then 0i32\n\
           else 1i32;\n\
         };",
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
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(executable).status.code(), Some(0));

    let module = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
    assert_eq!(module.matches("%mal_symbol_result = alloca").count(), 1);
}

#[test]
fn derives_symbol_runtime_dependencies_from_symbol_operations() {
    let closure_directory = NativeFixture::new("driver-llvm-closure-runtime-dependencies");
    let closure_source = closure_directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> {\n\
           operation(value);\n\
         };\n\
         main :: Unit -> Int32 := () -> { apply((value) -> { value; }, 0i32); };",
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
        "main :: Unit -> Int32 := () -> { discarded := \"a\" + \"b\"; 0i32; };",
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
    assert!(symbol_module.contains("declare void @mal_runtime_symbol_concatenate"));
    for runtime in ["bytes.c", "bytes_internal.h", "symbol.c"] {
        assert!(
            symbol_artifacts.join(runtime).is_file(),
            "missing {runtime}"
        );
    }
}

#[test]
fn reuses_owned_symbols_across_empty_concatenation() {
    let directory = NativeFixture::new("driver-llvm-symbol-empty-concatenation");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {\n\
           value := \"a\" + \"b\";\n\
           left := \"\" + value;\n\
           right := value + \"\";\n\
           if (left == right)\n\
           then { (value # 1usize).i32 - 98i32 }\n\
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
fn traps_when_symbol_storage_cannot_be_allocated() {
    let directory = NativeFixture::new("driver-symbol-allocation-failure");
    let source = directory.write(
        "program.mal",
        "require \"allocation.c\";\n\
         main :: Unit -> Int32 := () -> { value := \"left\" + \"right\"; if (value == \"leftright\") then 0i32 else 1i32; };",
    );
    let executable = directory.join("program");
    directory.write(
        "allocation.c",
        "#include <stddef.h>\nvoid *__wrap_malloc(size_t size) { (void)size; return NULL; }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--clang-arg"),
        OsStr::new("-Wl,--wrap=malloc"),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = directory.run(executable);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("mal trap: symbol allocation failed"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}
