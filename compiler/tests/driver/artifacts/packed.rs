use super::*;

#[test]
fn lowers_post_growth_buffer_access_through_the_active_data_slot() {
    let directory = NativeFixture::new("driver-llvm-stable-packed-data");
    let source = directory.join("program.mal");
    let baseline = directory.join("baseline");
    let production = directory.join("production");
    let baseline_artifacts = directory.join("baseline-artifacts");
    let production_artifacts = directory.join("production-artifacts");
    directory.write(
        "program.mal",
        "fill :: (Buffer<Int32>, Int32, Int32) -> Unit := (buffer, index, limit) -> {
           if (index > limit)
           then { () }
           else {
             _ := buffer.new(buffer.get((index - 1i32).usize) + 1i32);
             fill(buffer, index + 1i32, limit);
           };
         };

         main :: Unit -> Int32 := () -> {
           values := pack<Int32>((buffer) -> {
             _ := buffer.new(0i32);
             buffer.fill(1i32, 128i32);
           });
           (values # 128usize) - 128i32;
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
    assert!(baseline_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("call ptr @mal_runtime_packed_builder_data_slot"));
    assert!(production_llvm.contains("!tbaa !3"));
    assert!(production_llvm.contains("mal packed element storage"));
    assert!(!production_llvm.contains("call ptr @mal_runtime_packed_builder_get"));
    assert!(production_llvm.contains("getelementptr i8, ptr"));
}

#[test]
fn passes_current_packed_data_to_helpers_before_and_after_growth() {
    let directory = NativeFixture::new("driver-llvm-direct-packed-data");
    let source = directory.join("program.mal");
    directory.write(
        "program.mal",
        "adjust :: ((Buffer<Int32>, USize), Int32) -> Int32 := (view, increment) -> {
           (buffer, index) := view;
           previous := buffer.get(index);
           buffer.put(index, previous + increment);
           previous;
         };

         capturedAdjust :: (Buffer<Int32>, Int32) -> Int32 := (buffer, increment) -> {
           nested :: (Unit -> Int32) := () -> {
             previous := buffer.get(0usize);
             buffer.put(0usize, previous + increment);
             previous;
           };
           nested();
         };

         append :: (Buffer<Int32>, Int32) -> Unit := (buffer, remaining) -> {
           if (remaining == 0i32)
           then { () }
           else {
             _ := buffer.new(remaining);
             append(buffer, remaining - 1i32);
           };
         };

         main :: Unit -> Int32 := () -> {
           values := pack<Int32>((buffer) -> {
             _ := buffer.new(10i32);
             _ := adjust(((buffer, 0usize), 1i32));
             append(buffer, 64i32);
             _ := adjust(((buffer, 0usize), 31i32));
             previous := capturedAdjust(buffer, 1i32);
             _ := buffer.new(previous);
             ();
           });
           (values # 0usize) - 43i32 + (values # 65usize) - 42i32;
         };",
    );

    for (profile, name) in [(Some("baseline"), "baseline"), (None, "production")] {
        let executable = directory.join(name);
        let artifacts = directory.join(format!("{name}-artifacts"));
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
        assert_eq!(directory.run(&executable).status.code(), Some(0));
    }
}

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
           original := pack<Int32>((buffer) -> {
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
fn prepares_an_edit_before_entering_its_callback() {
    let directory = NativeFixture::new("driver-llvm-noop-packed-edit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           original := pack<Int32>((buffer) -> {
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
