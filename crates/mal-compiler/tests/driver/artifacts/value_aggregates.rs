use super::*;

#[test]
fn specializes_generic_functions_to_distinct_llvm_functions() {
    let directory = NativeFixture::new("driver-llvm-generics");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "identity<A> :: A -> A := (value) -> value;\n\
         main :: Unit -> Int32 := () -> identity<Int32>(40) + identity<UInt8>(2u8).i32;",
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
    assert_eq!(directory.run(executable).status.code(), Some(42));
}

#[test]
fn resumes_mixed_numeric_scalar_frames_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-scalar-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "sum :: Float64 -> Float64 := (value) -> {\n\
           if (value == 0.0f64) then { 0.0f64 } else {\n\
             narrow := value.i16;\n\
             wide := value.u64;\n\
             rest := sum(value - 1.0f64);\n\
             rest + narrow.f64 + wide.f64;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { (sum(10000.0f64) - 100010000.0f64).i32; };",
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
fn constructs_and_resumes_unmanaged_products_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-product");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "build :: Int32 -> (Int16, UInt64) := (remaining) -> {\n\
           if (remaining == 0) then { (0i16, 0u64) } else {\n\
             (narrow, wide) := build(remaining - 1);\n\
             (narrow + 1i16, wide + 1u64);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           (narrow, wide) := build(10000);\n\
           narrow.i32 + wide.i32 - 20000;\n\
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
fn branches_over_bool_and_unmanaged_sums_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-sum");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Int16, (UInt32, UInt64)];\n\
         choose :: Bool -> Choice := (flag) -> [first, second] => {\n\
           when (flag) { second(20u32, 22u64) };\n\
           first(42i16)\n\
         };\n\
         score :: Choice -> Int32 := (choice) -> {\n\
           choice[\n\
             (value) -> { value.i32 },\n\
             (pair) -> { (left, right) := pair; left.i32 + right.i32 }];\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           flag := true != false;\n\
           score(choose(false)) + score(choose(flag)) - 84;\n\
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
fn preserves_payload_bytes_that_overlap_another_sum_variants_padding() {
    let directory = NativeFixture::new("driver-llvm-sum-padding");
    let source = directory.join("program.mal");
    directory.write(
        "program.mal",
        "Choice :: [Buffer<UInt8>, (UInt8, UInt64)];\n\
         wrap :: Buffer<UInt8> -> Choice := (bytes) -> [withBytes, withPair] => withBytes(bytes);\n\
         identity :: Choice -> Choice := (choice) -> choice;\n\
         main :: Unit -> Int32 := () -> {\n\
           bytes := make<UInt8>(1usize);\n\
           bytes.new(42u8);\n\
           identity(wrap(bytes))[\n\
             (result) -> { result.get(0usize).i32 - 42 },\n\
             (_) -> { 1 }];\n\
         };",
    );

    for optimization in ["baseline", "production"] {
        let executable = directory.join(optimization);
        let output = directory.malc([
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--optimization"),
            OsStr::new(optimization),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(directory.run(executable).status.code(), Some(0));
    }
}

#[test]
fn preserves_sum_payloads_stored_in_buffers_under_optimization() {
    let directory = NativeFixture::new("driver-llvm-buffered-sum");
    let source = directory.join("program.mal");
    directory.write(
        "program.mal",
        "Choice :: [Unit, UInt64];\n\
         none :: Unit -> Choice := () -> [none, some] => [none];\n\
         some :: UInt64 -> Choice := (value) -> [none, some] => some(value);\n\
         main :: Unit -> Int32 := () -> {\n\
           choices := make<Choice>(2usize);\n\
           choices.fill(0usize, 2usize, none());\n\
           choices.put(1usize, some(42u64));\n\
           empty := choices.get(0usize)[() -> { 0i32 }, (_) -> { 1i32 }];\n\
           value := choices.get(1usize)[() -> { 1i32 }, (found) -> { found.i32 - 42i32 }];\n\
           empty + value;\n\
         };",
    );

    for optimization in ["baseline", "production"] {
        let executable = directory.join(optimization);
        let output = directory.malc([
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--optimization"),
            OsStr::new(optimization),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(directory.run(executable).status.code(), Some(0));
    }
}

#[test]
fn runs_sum_results_and_postfix_application_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-sum-result");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Unit, Int32];\n\
         none :: Unit -> Choice := () -> [none, some] => { [none] };\n\
         some :: Int32 -> Choice := (value) -> [none, some] => { some(value) };\n\
         score :: Choice -> Int32 := (choice) -> {\n\
           choice[\n\
           () -> { 0 },\n\
           (value) -> { value }\n\
           ];\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           score([none]) + score(41[some]) - 41;\n\
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
