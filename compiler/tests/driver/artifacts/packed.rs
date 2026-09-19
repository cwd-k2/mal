use super::*;

#[test]
fn starts_a_count_zero_buffer_with_requested_capacity() {
    let directory = NativeFixture::new("driver-llvm-packed-make");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           values := make<Int32>(8usize, (buffer) -> {
             first := buffer.new(40i32);
             second := buffer.new(2i32);
             buffer.put(first, buffer.get(first) + buffer.get(second));
             ();
           });
           (#values).i32 - 2i32 + (values # 0usize) - 42i32;
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
    assert!(llvm.contains("call ptr @mal_runtime_packed_builder_make"));
}

#[test]
fn initializes_builder_scratch_padding_before_runtime_byte_inspection() {
    let directory = NativeFixture::new("driver-llvm-packed-builder-padding");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           values := make<(UInt8, Int64)>(1usize, (buffer) -> {
             _ := buffer.new((0u8, 0));
             ();
           });
           (byte, integer) := values # 0usize;
           byte.i32 + integer.i32;
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
    assert!(llvm.contains("store [16 x i8] zeroinitializer, ptr %mal_packed_new_value, align 8"));
}

#[test]
fn takes_a_capture_only_from_a_uniquely_invoked_callback() {
    let directory = NativeFixture::new("driver-llvm-packed-unique-capture");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           source := make<Int32>(0usize, (buffer) -> { _ := buffer.new(1i32); (); });
           _ := make<Unit>(0usize, (_) -> {
             changed := source.edit<Int32>((buffer) -> buffer.put(0usize, 2i32));
             _ := changed # 0usize;
             ();
           });
           0;
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
    assert!(llvm.contains("call i8 @mal_runtime_environment_is_unique"));
    assert!(llvm.contains("mal_capture_take_"));
}

#[test]
fn keeps_a_capture_in_a_callback_invoked_more_than_once() {
    let directory = NativeFixture::new("driver-llvm-packed-shared-callback");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let artifacts = directory.join("artifacts");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           source := make<Int32>(0usize, (buffer) -> { _ := buffer.new(1i32); (); });
           callback :: Buffer<Unit> -> Unit := (_) -> {
             changed := source.edit<Int32>((buffer) -> buffer.put(0usize, 2i32));
             _ := changed # 0usize;
             ();
           };
           _ := make<Unit>(0usize, callback);
           _ := make<Unit>(0usize, callback);
           0;
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
    assert!(!llvm.contains("call i8 @mal_runtime_environment_is_unique"));
}

#[test]
fn passes_repeated_buffer_positions_to_one_non_growing_helper() {
    let directory = NativeFixture::new("driver-llvm-packed-multi-buffer-abi");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    let baseline = directory.join("program-baseline");
    directory.write(
        "program.mal",
        "combine :: ((Buffer<Int32>, Buffer<Int32>), USize) -> Unit := ((left, right), index) -> { left.put(index, left.get(index) + right.get(index)); }; main :: Unit -> Int32 := () -> { values := make<Int32>(2usize, (buffer) -> { _ := buffer.new(5i32); combine(((buffer, buffer), 0usize)); _ := buffer.new(10i32); (); }); (values # 0usize) + (values # 1usize) - 20i32; };",
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
    assert_eq!(directory.run(&executable).status.code(), Some(0));

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        baseline.as_os_str(),
        OsStr::new("--optimization"),
        OsStr::new("baseline"),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(directory.run(&baseline).status.code(), Some(0));
}

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
           values := make<Int32>(0usize, (buffer) -> {
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

         append :: (Buffer<Int32>, Int32) -> Unit := (buffer, remaining) -> {
           when (remaining != 0i32) {
             _ := buffer.new(remaining);
             append(buffer, remaining - 1i32);
           };
           ();
         };

         main :: Unit -> Int32 := () -> {
           values := make<Int32>(0usize, (buffer) -> {
             _ := buffer.new(10i32);
             _ := adjust(((buffer, 0usize), 1i32));
             append(buffer, 64i32);
             _ := adjust(((buffer, 0usize), 31i32));
             previous := adjust(((buffer, 0usize), 1i32));
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
