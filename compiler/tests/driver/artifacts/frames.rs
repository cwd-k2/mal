use super::*;

#[test]
fn omits_self_recursive_parameter_fields_preserved_by_every_edge() {
    let directory = NativeFixture::new("driver-llvm-frame-pass-through");
    let source = directory.join("program.mal");
    let baseline = directory.join("baseline");
    let production = directory.join("production");
    let baseline_artifacts = directory.join("baseline-artifacts");
    let production_artifacts = directory.join("production-artifacts");
    directory.write(
        "program.mal",
        "Values :: Packed<Int32>;
         walk :: (Values, Int32) -> Int32 := (fixed, depth) -> {
           if (depth == 0i32) then { fixed # 0usize } else {
             child := walk(fixed, depth - 1i32);
             child + fixed # 0usize;
           };
         };
         main :: Unit -> Int32 := () -> {
           fixed := bulk<Int32>(1usize, (buffer) -> { _ := buffer.new(1i32); (); });
           walk(fixed, 10000i32) - 10001i32;
         };",
    );

    for (executable, artifacts, profile) in [
        (&baseline, &baseline_artifacts, "baseline"),
        (&production, &production_artifacts, "production"),
    ] {
        let output = directory.malc([
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
            OsStr::new("--optimization"),
            OsStr::new(profile),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(directory.run(executable).status.code(), Some(0));
    }

    let baseline_module = std::fs::read_to_string(baseline_artifacts.join("program.ll")).unwrap();
    let production_module =
        std::fs::read_to_string(production_artifacts.join("program.ll")).unwrap();
    assert!(baseline_module.contains("i64 24)"));
    assert!(production_module.contains("i64 8)"));
    assert!(!baseline_module.contains("%mal_local_control_top"));
    assert!(production_module.contains("%mal_local_control_top"));
    assert!(!baseline_module.contains("%mal_local_control_storage"));
    assert!(production_module.contains("%mal_local_control_storage"));
}
#[test]
fn builds_deep_non_tail_self_recursion_with_a_c_runtime_arena() {
    let directory = NativeFixture::new("driver-llvm-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "sum :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0) then { 0 } else {\n\
             rest := sum(value - 1);\n\
             value + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { sum(10000) - 50005000; };",
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
fn removes_identity_result_continuations_before_frame_emission() {
    let directory = NativeFixture::new("driver-llvm-identity-continuation");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "countdown :: Int32 -> Int32 := (value) -> [return] => {\n\
           when (value == 0i32) { return(0i32); };\n\
           child := countdown(value - 1i32);\n\
           return(child);\n\
         };\n\
         main :: Unit -> Int32 := () -> { countdown(1000000i32); };",
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
    assert!(!module.contains("mal_control_reserve_frame"));
}

#[test]
fn removes_unit_result_continuations_before_frame_emission() {
    let directory = NativeFixture::new("driver-llvm-unit-continuation");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "countdown :: Int32 -> Unit := (value) -> {\n\
           if (value == 0i32) then { () } else {\n\
             countdown(value - 1i32);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { countdown(1000000i32); 0i32; };",
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
    assert!(!module.contains("mal_control_reserve_frame"));
}

#[test]
fn resumes_single_constructor_frames_without_live_payloads() {
    let directory = NativeFixture::new("driver-llvm-empty-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "countdown :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0i32)\n\
           then { 0i32 }\n\
           else {\n\
             ignored := countdown(value - 1i32);\n\
             0i32;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { countdown(100000i32); };",
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
        "inner :: Int64 -> Int64 := (n) -> {\n\
           if (n == 0i64)\n\
           then { 0i64 }\n\
           else {\n\
             rest := inner(n - 1i64);\n\
             n + rest;\n\
           };\n\
         };\n\
         outer :: Int32 -> Int64 := (n) -> {\n\
           if (n == 0i32)\n\
           then { inner(100i64) }\n\
           else {\n\
             rest := outer(n - 1i32);\n\
             n.i64 + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { outer(20i32).i32 - 5260i32; };",
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
        "finish :: Int32 -> Int32 := (value) -> { value + 1i32; };\n\
         unwind :: Int32 -> Int32 := (value) -> {\n\
           if (value == 0i32) then { finish(0i32) } else {\n\
             child := unwind(value - 1i32);\n\
             finish(child);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { unwind(100000i32) - 100001i32; };",
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
        "walk :: Int32 -> Int32 := (value) -> {\n\
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
         main :: Unit -> Int32 := () -> { walk(10000) - 10000; };",
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
