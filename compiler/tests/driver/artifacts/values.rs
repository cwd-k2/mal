use super::*;
#[test]
fn builds_every_integer_width_with_signed_and_unsigned_llvm_comparisons() {
    let directory = NativeFixture::new("driver-llvm-integers");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "signed8 :: Int8 -> Int32 := (value) -> {\n\
           next := value + 1i8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed16 :: Int16 -> Int32 := (value) -> {\n\
           next := value + 1i16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed64 :: Int64 -> Int32 := (value) -> {\n\
           next := value + 1i64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned8 :: UInt8 -> Int32 := (value) -> {\n\
           next := value + 1u8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned16 :: UInt16 -> Int32 := (value) -> {\n\
           next := value + 1u16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned32 :: UInt32 -> Int32 := (value) -> {\n\
           next := value + 1u32; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned64 :: UInt64 -> Int32 := (value) -> {\n\
           next := value + 1u64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signedOps :: Int64 -> Int32 := (value) -> {\n\
           quotient := value / 2i64;\n\
           remainder := value % 2i64;\n\
           shifted := (value << 1i64) >> 1i64;\n\
           if (quotient == -4i64) then {\n\
             if (remainder == -1i64) then {\n\
               if (shifted == value) then { 1 } else { 0 };\n\
             } else { 0 };\n\
           } else { 0 };\n\
         };\n\
         unsignedOps :: UInt64 -> Int32 := (value) -> {\n\
           shifted := (value << 1u64) >> 1u64;\n\
           remainder := shifted % 3u64;\n\
           if (remainder == 1u64) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           signed8(127i8) + signed16(32767i16) + signed64(9223372036854775807i64) +\n\
           unsigned8(255u8) + unsigned16(65535u16) +\n\
           unsigned32(4294967295u32) + unsigned64(18446744073709551615u64) +\n\
           signedOps(-9i64) + unsignedOps(10u64) - 9 +\n\
           Int32(UInt64(-1i8)) + 1;\n\
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
fn builds_strict_float_arithmetic_and_nan_comparisons_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-float");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "check32 :: Float32 -> Int32 := (value) -> {\n\
           result := value * 2.0f32 + 0.5f32;\n\
           if (result == 3.5f32) then { 1 } else { 0 };\n\
         };\n\
         check64 :: Float64 -> Int32 := (value) -> {\n\
           result := -(value / 2.0f64);\n\
           if (result <= -0.75f64) then { 1 } else { 0 };\n\
         };\n\
         checkNaN :: Float64 -> Int32 := (value) -> {\n\
           zero := value - value;\n\
           nan := zero / zero;\n\
           if (nan != nan) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           check32(1.5f32) + check64(1.5f64) + checkNaN(1.0f64) +\n\
           Int32(Float64(3)) + Int32(Float32(1.75f64)) - 7;\n\
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
fn resumes_mixed_numeric_scalar_frames_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-scalar-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "sum :: Float64 -> Float64 := (value) -> {\n\
           if (value == 0.0f64) then { 0.0f64 } else {\n\
             narrow := Int16(value);\n\
             wide := UInt64(value);\n\
             rest := sum(value - 1.0f64);\n\
             rest + Float64(narrow) + Float64(wide);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { Int32(sum(10000.0f64) - 100010000.0f64); };",
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
           Int32(narrow) + Int32(wide) - 20000;\n\
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
             (value) -> { Int32(value) },\n\
             (pair) -> { (left, right) := pair; Int32(left) + Int32(right) }];\n\
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
fn runs_sum_results_and_postfix_application_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-sum-return");
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

#[test]
fn preserves_short_circuit_effect_order_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-short-circuit");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern forbidden :: Unit -> Bool;\n\
         main :: Unit -> Int32 := () -> {\n\
           if (false && forbidden()) then { 1 } else { 0 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_forbidden(call) {\n\
             mal_call_trap(call, \"short-circuit operand was evaluated\");\n\
         }\n",
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
fn accesses_unaligned_scalar_and_pointer_storage_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-memory");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern memory :: UInt64 -> Ptr;\n\
         main :: Unit -> Int32 := () -> {\n\
           base := memory(64u64);\n\
           UInt64.store(base, 42u64);\n\
           pointerSlot := base + UInt64.size;\n\
           Ptr.store(pointerSlot, base);\n\
           floatSlot := pointerSlot + Ptr.size;\n\
           Float32.store(floatSlot, 1.5f32);\n\
           restored := Ptr.load(pointerSlot);\n\
           start := floatSlot - Ptr.size - UInt64.size;\n\
           value := UInt64.load(restored) + UInt64.load(start);\n\
           if (Float32.load(floatSlot) == 1.5f32) then { Int32(value) - 84 } else { 1 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static unsigned char storage[65];\n\
         MAL_DEFINE_memory(call, size) {\n\
             (void)size;\n\
             return mal_Ptr_return(call, storage + 1);\n\
         }\n",
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
