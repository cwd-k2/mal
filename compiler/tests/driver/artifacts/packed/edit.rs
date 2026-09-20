use super::*;

#[test]
fn prepares_edit_once_before_lowering_recursive_packed_access() {
    let directory = NativeFixture::new("driver-llvm-prepared-packed-edit");
    let source = directory.join("program.mal");
    let baseline = directory.join("baseline");
    let production = directory.join("production");
    let baseline_artifacts = directory.join("baseline-artifacts");
    let production_artifacts = directory.join("production-artifacts");
    directory.write(
        "program.mal",
        "update :: (Buffer<Int32>, USize) -> Unit := (buffer, remaining) -> {
           if (remaining == 0usize)
           then { () }
           else {
             buffer.put(0usize, buffer.get(0usize) + 1i32);
             update(buffer, remaining - 1usize);
           };
         };

         main :: Unit -> Int32 := () -> {
           original := make<Int32>(0usize, (buffer) -> {
             buffer.new(40i32);
             ();
           });
           changed := original.edit<Int32>((buffer) -> update(buffer, 2usize));
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
    assert!(baseline_llvm.contains("call ptr @mal_runtime_packed_builder_prepare_edit"));
    assert!(baseline_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_prepare_edit"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert_eq!(
        production_llvm
            .matches("call ptr @mal_runtime_packed_builder_prepare_edit")
            .count(),
        1
    );
    assert!(!production_llvm.contains("call ptr @mal_runtime_packed_builder_get"));
}

#[test]
fn preserves_a_packed_source_observed_during_its_edit() {
    let directory = NativeFixture::new("driver-llvm-packed-buffer-noalias");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "update :: (Buffer<Int32>, Packed<Int32>) -> Unit := (buffer, source) -> {
           buffer.put(0usize, (source # 0usize) + 2i32);
           buffer.put(1usize, (source # 1usize) + 3i32);
         };

         main :: Unit -> Int32 := () -> {
           original := make<Int32>(0usize, (buffer) -> {
             _ := buffer.new(40i32);
             _ := buffer.new(50i32);
             ();
           });
           changed := original.edit<Int32>((buffer) -> update(buffer, original));
           (original # 0usize) - 40i32 + (original # 1usize) - 50i32
             + (changed # 0usize) - 42i32 + (changed # 1usize) - 53i32;
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
    assert!(llvm.contains("ptr noalias %mal_buffer_data"));
}

#[test]
fn preserves_a_packed_source_across_recursive_buffer_updates() {
    let directory = NativeFixture::new("driver-llvm-recursive-packed-buffer-noalias");
    let source = directory.join("program.mal");
    directory.write(
        "program.mal",
        "update :: (Buffer<Int32>, Packed<Int32>, USize) -> Unit := (buffer, source, remaining) -> {
           if (remaining == 0usize)
           then { () }
           else {
             buffer.put(0usize, buffer.get(0usize) + (source # 0usize) - 39i32);
             update(buffer, source, remaining - 1usize);
           };
         };

         main :: Unit -> Int32 := () -> {
           original := make<Int32>(0usize, (buffer) -> {
             _ := buffer.new(40i32);
             ();
           });
           changed := original.edit<Int32>((buffer) -> update(buffer, original, 2usize));
           (original # 0usize) - 40i32 + (changed # 0usize) - 42i32;
         };",
    );

    for (profile, name) in [(Some("baseline"), "baseline"), (None, "production")] {
        let executable = directory.join(name);
        let mut arguments = vec![
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
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
        assert_eq!(directory.run(&executable).status.code(), Some(0));
    }
}

#[test]
fn prepares_an_edit_before_entering_its_callback() {
    let directory = NativeFixture::new("driver-llvm-noop-packed-edit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           original := make<Int32>(0usize, (buffer) -> {
             buffer.new(40i32);
             ();
           });
           unchanged := original.edit<Int32>((buffer) -> {
             _ := buffer.get(0usize);
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
    assert_eq!(
        llvm.matches("call ptr @mal_runtime_packed_builder_prepare_edit")
            .count(),
        1
    );
}
