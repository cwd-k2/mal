use super::*;

#[test]
fn calls_a_native_target_from_an_indirect_recursive_region_site() {
    let directory = NativeFixture::new("driver-llvm-region-native-target");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> {\n\
           called := operation(value);\n\
           called + 0i32;\n\
         };\n\
         identity :: Int32 -> Int32 := (value) -> { value + 1i32; };\n\
         recurse :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0i32)\n\
           then { 0i32 }\n\
           else {\n\
             child := apply(recurse, value - 1i32);\n\
             child + 1i32;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { apply(identity, 41i32) - 42i32; };",
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
         apply :: ((Packet -> Packet), Packet) -> Packet := (operation, value) -> {\n\
           operation(value);\n\
         };\n\
         identity :: Packet -> Packet := (value) -> { value; };\n\
         recurse :: Packet -> Packet := (value) -> {\n\
           (remaining, text) := value;\n\
           if (remaining == 0i32)\n\
           then { value }\n\
           else {\n\
             child := apply(recurse, (remaining - 1i32, text));\n\
             (next, result) := child;\n\
             (next + 1i32, result + \"!\");\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
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
        "walk :: (Int32, Symbol) -> Symbol := (depth, value) -> {\n\
           if (depth == 0i32) then { value } else {\n\
             resumed := walk(depth - 1i32, value);\n\
             if (resumed # 0usize == 120u8) then { resumed } else { \"bad\" };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(10000i32, seed);\n\
           (result # 1usize).i32 - 121i32;\n\
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
fn replaces_retired_frame_storage_with_managed_live_fields() {
    let directory = NativeFixture::new("driver-llvm-managed-frame-replacement");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "walk :: (Int32, Symbol) -> Symbol := (depth, value) -> {\n\
           if (depth == 0i32) then { value } else {\n\
             p1 := depth.i64;\n\
             p2 := depth.i64;\n\
             p3 := depth.i64;\n\
             p4 := depth.i64;\n\
             first := walk(depth - 1i32, value);\n\
             offset := p1 - p1 + p2 - p2 + p3 - p3 + p4 - p4;\n\
             second := walk(depth - 1i32 + offset.i32, first);\n\
             if (second # 0usize == 120u8) then { first } else { second };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(12i32, seed);\n\
           (result # 1usize).i32 - 121i32;\n\
         };",
    );

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
    assert_eq!(
        module
            .matches("call ptr @mal_control_reserve_frame")
            .count(),
        1
    );
    assert!(module.contains("%mal_local_control_storage = alloca ptr"));
    assert!(module.contains("call i64 @mal_control_capacity"));
}

#[test]
fn runs_managed_direct_self_tail_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-managed-tail");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "count :: (Symbol, Int64) -> USize := (value, remaining) -> {\n\
           if (remaining == 0i64) then { #value }\n\
           else { count(value, remaining - 1i64) };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           seed := \"x\" + \"y\";\n\
           if (count(seed, 100000i64) == 2usize) then { 0 } else { 1 };\n\
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
