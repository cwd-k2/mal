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
           fixed := make<Int32>(1usize, (buffer) -> { _ := buffer.new(1i32); (); });
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

#[path = "frames/frame_native.rs"]
mod frame_native;
