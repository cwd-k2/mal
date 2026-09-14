use super::*;
#[test]
fn hands_direct_self_arguments_to_wildcard_parameters() {
    let directory = NativeFixture::new("driver-llvm-self-wildcard");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern again :: Unit -> Bool;\n\
         make :: Int32 -> (Unit -> Int32) := (value) -> { () -> { value }; };\n\
         unmanagedFrame :: Int32 -> Int32 := (_) -> {\n\
           if (again()) then { child := unmanagedFrame(1i32); child + 1i32; }\n\
           else { 0i32 };\n\
         };\n\
         managedFrame :: (Unit -> Int32) -> Int32 := (_) -> {\n\
           if (again()) then { child := managedFrame(make(1i32)); child + 1i32; }\n\
           else { 0i32 };\n\
         };\n\
         unmanagedTail :: Int32 -> Int32 := (_) -> {\n\
           if (again()) then { unmanagedTail(1i32) } else { 0i32 };\n\
         };\n\
         managedTail :: (Unit -> Int32) -> Int32 := (_) -> {\n\
           if (again()) then { managedTail(make(1i32)) } else { 0i32 };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
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
fn borrows_managed_wildcard_arguments_across_native_calls() {
    let directory = NativeFixture::new("driver-llvm-native-wildcard");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Callback :: Unit -> Int32;\n\
         Consumer :: Callback -> Int32;\n\
         make :: Int32 -> Callback := (value) -> { () -> { value }; };\n\
         discard :: Consumer := (_) -> { 0i32; };\n\
         directNonTail :: Int32 -> Int32 := (value) -> {\n\
           result := discard(make(value));\n\
           result + 0i32;\n\
         };\n\
         directTail :: Int32 -> Int32 := (value) -> { discard(make(value)); };\n\
         dispatchNonTail :: (Consumer, Callback) -> Int32 := (consumer, callback) -> {\n\
           result := consumer(callback);\n\
           result + 0i32;\n\
         };\n\
         dispatchTail :: (Consumer, Callback) -> Int32 := (consumer, callback) -> {\n\
           consumer(callback);\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           directNonTail(1i32) + directTail(2i32)\n\
             + dispatchNonTail(discard, make(3i32))\n\
             + dispatchTail(discard, make(4i32));\n\
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
fn self_tail_transition_in_a_common_machine_uses_the_active_function_entry() {
    let directory = NativeFixture::new("driver-llvm-common-self-tail-entry");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> {\n\
           called := operation(value);\n\
           called + 0i32;\n\
         };\n\
         recurse :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0i32) then { 0i32 } else {\n\
             if (value == 1i32) then { recurse(0i32) } else {\n\
               child := apply(recurse, value - 1i32);\n\
               child + 1i32;\n\
             };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { apply(recurse, 1i32); };",
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
