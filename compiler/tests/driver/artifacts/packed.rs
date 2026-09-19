use super::*;

#[test]
fn lowers_post_growth_packed_access_through_stable_data() {
    let directory = NativeFixture::new("driver-llvm-stable-packed-data");
    let source = directory.join("program.mal");
    let baseline = directory.join("baseline");
    let production = directory.join("production");
    let baseline_artifacts = directory.join("baseline-artifacts");
    let production_artifacts = directory.join("production-artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           values := pack<Int32>((new, get, put) -> {
             new(40i32);
             value := get(0usize);
             put(0usize, value + 2i32);
             ();
           });
           (values # 0usize) - 42i32;
         };",
    );

    for (profile, executable, artifacts) in [
        (Some("baseline"), &baseline, &baseline_artifacts),
        (None, &production, &production_artifacts),
    ] {
        let mut arguments = vec![
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
        ];
        if let Some(profile) = profile {
            arguments.extend([OsStr::new("--optimization"), OsStr::new(profile)]);
        }
        let output = directory.malc(arguments);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(directory.run(executable).status.code(), Some(0));
    }

    let baseline_llvm = std::fs::read_to_string(baseline_artifacts.join("program.ll")).unwrap();
    let production_llvm = std::fs::read_to_string(production_artifacts.join("program.ll")).unwrap();
    assert!(!baseline_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @llvm.invariant.start.p0"));
    assert!(production_llvm.contains("getelementptr i8, ptr"));
}

#[test]
fn prepares_edit_lazily_while_lowering_recursive_packed_access() {
    let directory = NativeFixture::new("driver-llvm-prepared-packed-edit");
    let source = directory.join("program.mal");
    let baseline = directory.join("baseline");
    let production = directory.join("production");
    let baseline_artifacts = directory.join("baseline-artifacts");
    let production_artifacts = directory.join("production-artifacts");
    directory.write(
        "program.mal",
        "update :: (USize -> Int32, (USize, Int32) -> Unit, USize) -> Unit := (get, put, remaining) -> {
           if (remaining == 0usize)
           then { () }
           else {
             put(0usize, get(0usize) + 1i32);
             update(get, put, remaining - 1usize);
           };
         };

         main :: Unit -> Int32 := () -> {
           original := pack<Int32>((new, _, _) -> {
             new(40i32);
             ();
           });
           changed := original.edit<Int32>((_, get, put) -> update(get, put, 2usize));
           (original # 0usize) - 40i32 + (changed # 0usize) - 42i32;
         };",
    );

    for (profile, executable, artifacts) in [
        (Some("baseline"), &baseline, &baseline_artifacts),
        (None, &production, &production_artifacts),
    ] {
        let mut arguments = vec![
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--artifact-dir"),
            artifacts.as_os_str(),
        ];
        if let Some(profile) = profile {
            arguments.extend([OsStr::new("--optimization"), OsStr::new(profile)]);
        }
        let output = directory.malc(arguments);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(directory.run(executable).status.code(), Some(0));
    }

    let baseline_llvm = std::fs::read_to_string(baseline_artifacts.join("program.ll")).unwrap();
    let production_llvm = std::fs::read_to_string(production_artifacts.join("program.ll")).unwrap();
    assert!(!baseline_llvm.contains("call ptr @mal_runtime_packed_builder_prepare_edit"));
    assert!(!baseline_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_prepare_edit"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @llvm.ptrmask.p0.i64"));
}

#[test]
fn does_not_prepare_an_edit_without_a_put_application() {
    let directory = NativeFixture::new("driver-llvm-noop-packed-edit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           original := pack<Int32>((new, _, _) -> {
             new(40i32);
             ();
           });
           unchanged := original.edit<Int32>((_, get, _) -> {
             _ := get(0usize);
             ();
           });
           (original # 0usize) - 40i32 + (unchanged # 0usize) - 40i32;
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
    assert_eq!(directory.run(&executable).status.code(), Some(0));

    let llvm = std::fs::read_to_string(artifacts.join("program.ll")).unwrap();
    assert!(!llvm.contains("call ptr @mal_runtime_packed_builder_prepare_edit"));
}
