use super::*;

#[test]
fn builds_a_constant_main_through_the_llvm_artifact_set() {
    let directory = NativeFixture::new("driver-llvm");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write("program.mal", "main :: Unit -> Int32 := \\() { 7; };");

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
    assert_eq!(directory.run(executable).status.code(), Some(7));
}

#[test]
fn builds_scalar_control_and_tail_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-control");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "increment :: Int32 -> Int32 := \\(value) { value + 1; };\n\
         countdown :: Int32 -> Int32 := \\(value) {\n\
           if (value == 0) then { increment(value) } else { countdown(value - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() { countdown(100000); };",
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
    assert_eq!(directory.run(executable).status.code(), Some(1));
}

#[test]
fn builds_deep_non_tail_self_recursion_with_a_c_runtime_arena() {
    let directory = NativeFixture::new("driver-llvm-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "sum :: Int32 -> Int32 := \\(value) {\n\
           if (value == 0) then { 0 } else {\n\
             rest := sum(value - 1);\n\
             value + rest;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { sum(10000) - 50005000; };",
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
    directory.write(
        "program.mal",
        "walk :: Int32 -> Int32 := \\(value) {\n\
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
         main :: Unit -> Int32 := \\() { walk(10000) - 10000; };",
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
fn resumes_managed_self_continuation_frames_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-managed-frame");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "walk :: (Int32, Symbol) -> Symbol := \\(depth, value) {\n\
           if (depth == 0i32) then { value } else {\n\
             resumed := walk(depth - 1i32, value);\n\
             if (resumed # 0u64 == 120u8) then { resumed } else { \"bad\" };\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           seed := \"x\" + \"y\";\n\
           result := walk(10000i32, seed);\n\
           Int32(result # 1u64) - 121i32;\n\
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
fn runs_managed_direct_self_tail_calls_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-managed-tail");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "count :: (Symbol, Int64) -> UInt64 := \\(value, remaining) {\n\
           if (remaining == 0i64) then { #value }\n\
           else { count(value, remaining - 1i64) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           seed := \"x\" + \"y\";\n\
           if (count(seed, 100000i64) == 2u64) then { 0 } else { 1 };\n\
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
fn builds_every_integer_width_with_signed_and_unsigned_llvm_comparisons() {
    let directory = NativeFixture::new("driver-llvm-integers");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "signed8 :: Int8 -> Int32 := \\(value) {\n\
           next := value + 1i8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed16 :: Int16 -> Int32 := \\(value) {\n\
           next := value + 1i16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signed64 :: Int64 -> Int32 := \\(value) {\n\
           next := value + 1i64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned8 :: UInt8 -> Int32 := \\(value) {\n\
           next := value + 1u8; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned16 :: UInt16 -> Int32 := \\(value) {\n\
           next := value + 1u16; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned32 :: UInt32 -> Int32 := \\(value) {\n\
           next := value + 1u32; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         unsigned64 :: UInt64 -> Int32 := \\(value) {\n\
           next := value + 1u64; if (next < value) then { 1 } else { 0 };\n\
         };\n\
         signedOps :: Int64 -> Int32 := \\(value) {\n\
           quotient := value / 2i64;\n\
           remainder := value % 2i64;\n\
           shifted := (value << 1i64) >> 1i64;\n\
           if (quotient == -4i64) then {\n\
             if (remainder == -1i64) then {\n\
               if (shifted == value) then { 1 } else { 0 };\n\
             } else { 0 };\n\
           } else { 0 };\n\
         };\n\
         unsignedOps :: UInt64 -> Int32 := \\(value) {\n\
           shifted := (value << 1u64) >> 1u64;\n\
           remainder := shifted % 3u64;\n\
           if (remainder == 1u64) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
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
        "check32 :: Float32 -> Int32 := \\(value) {\n\
           result := value * 2.0f32 + 0.5f32;\n\
           if (result == 3.5f32) then { 1 } else { 0 };\n\
         };\n\
         check64 :: Float64 -> Int32 := \\(value) {\n\
           result := -(value / 2.0f64);\n\
           if (result <= -0.75f64) then { 1 } else { 0 };\n\
         };\n\
         checkNaN :: Float64 -> Int32 := \\(value) {\n\
           zero := value - value;\n\
           nan := zero / zero;\n\
           if (nan != nan) then { 1 } else { 0 };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
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
        "sum :: Float64 -> Float64 := \\(value) {\n\
           if (value == 0.0f64) then { 0.0f64 } else {\n\
             narrow := Int16(value);\n\
             wide := UInt64(value);\n\
             rest := sum(value - 1.0f64);\n\
             rest + Float64(narrow) + Float64(wide);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(sum(10000.0f64) - 100010000.0f64); };",
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
        "build :: Int32 -> (Int16, UInt64) := \\(remaining) {\n\
           if (remaining == 0) then { (0i16, 0u64) } else {\n\
             (narrow, wide) := build(remaining - 1);\n\
             (narrow + 1i16, wide + 1u64);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
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
         choose :: Bool -> Choice := \\(flag) {\n\
           if (flag) then { Choice[1]((20u32, 22u64)) } else { Choice[0](42i16) };\n\
         };\n\
         score :: Choice -> Int32 := \\(choice) {\n\
           case (choice)\n\
             [0](value) { Int32(value) }\n\
             [1](pair) { (left, right) := pair; Int32(left) + Int32(right) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
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
fn accesses_unaligned_scalar_and_pointer_storage_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-memory");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern memory :: UInt64 -> Ptr;\n\
         main :: Unit -> Int32 := \\() {\n\
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

#[test]
fn owns_flat_symbols_across_direct_llvm_calls() {
    let directory = NativeFixture::new("driver-llvm-symbol");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "check :: Symbol -> Int32 := \\(value) {\n\
           if (#value == 2u64) then {\n\
             if (value == \"ab\") then {\n\
               if (value != \"ac\") then { 0 } else { 1 };\n\
             } else { 2 };\n\
           } else { 3 };\n\
         };\n\
         main :: Unit -> Int32 := \\() { joined := \"a\" + \"b\"; check(joined); };",
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
fn bridges_symbol_parameters_and_results_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-symbol-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern inspect :: Symbol -> UInt8;\n\
         extern fetch :: Unit -> Symbol;\n\
         main :: Unit -> Int32 := \\() {\n\
           seed := \"x\" + \"y\";\n\
           if (inspect(seed) == 1u8) then { Int32(fetch() # 1u64) - 107 }\n\
           else { 1 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_inspect(call, value) {\n\
             mal_span_t bytes = mal_Symbol_to_bytes(call, value);\n\
             return (uint8_t)(bytes.length == 2 && bytes.data[0] == 'x' && bytes.data[1] == 'y');\n\
         }\n\
         MAL_DEFINE_fetch(call) {\n\
             static const uint8_t bytes[] = {'o', 'k'};\n\
             return mal_Symbol_return(\n\
                 call,\n\
                 mal_Symbol_from_bytes((mal_span_t){.data = bytes, .length = sizeof(bytes)})\n\
             );\n\
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
fn marshals_managed_products_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-product-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Packet :: (UInt64, Symbol);\n\
         extern exchange :: Packet -> Packet;\n\
         main :: Unit -> Int32 := \\() {\n\
           (number, text) := exchange(41u64, \"a\" + \"b\");\n\
           if (number == 42u64) then {\n\
             if (text == \"ab\") then { 0 } else { 1 };\n\
           } else { 2 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_exchange(call, value) {\n\
             mal_span_t bytes = mal_Symbol_to_bytes(call, value.field_1);\n\
             if (bytes.length != 2 || bytes.data[0] != 'a' || bytes.data[1] != 'b') {\n\
                 mal_call_trap(call, \"unexpected packet\");\n\
             }\n\
             return mal_Packet_return(\n\
                 call,\n\
                 (mal_Packet_t){.field_0 = value.field_0 + 1, .field_1 = value.field_1}\n\
             );\n\
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
fn marshals_active_sum_payloads_recursively_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-sum-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Choice :: [Unit, (UInt64, Symbol)];\n\
         Envelope :: (UInt8, Choice);\n\
         extern exchange :: Envelope -> Envelope;\n\
         main :: Unit -> Int32 := \\() {\n\
           (number, choice) := exchange(41u8, Choice[1]((7u64, \"a\" + \"b\")));\n\
           case (choice)\n\
             [0](_) { 1 }\n\
             [1](packet) {\n\
               (bias, text) := packet;\n\
               if (number == 42u8) then {\n\
                 if (bias == 7u64) then {\n\
                   if (text == \"ab\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.field_1.tag != mal_Choice_tag_1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_span_t bytes = mal_Symbol_to_bytes(\n\
                 call, value.field_1.payload.variant_1.field_1\n\
             );\n\
             if (bytes.length != 2 || bytes.data[0] != 'a' || bytes.data[1] != 'b') {\n\
                 mal_call_trap(call, \"unexpected payload\");\n\
             }\n\
             return mal_Envelope_return(\n\
                 call,\n\
                 (mal_Envelope_t){.field_0 = value.field_0 + 1, .field_1 = value.field_1}\n\
             );\n\
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
fn owns_symbols_nested_in_products_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-symbol-product");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "inspect :: (Symbol, UInt64) -> (Symbol, UInt8) := \\(input) {\n\
           (value, index) := input;\n\
           (value, value # index);\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           joined := \"ab\" + \"cd\";\n\
           (copy, byte) := inspect(joined, 2u64);\n\
           if (copy == \"abcd\") then { Int32(byte) - 99 } else { 1 };\n\
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
fn retains_only_active_managed_sum_payloads_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-symbol-sum");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "Choice :: [Symbol, (UInt64, Symbol)];\n\
         choose :: Bool -> Choice := \\(second) {\n\
           if (second) then { Choice[1]((2u64, \"b\" + \"c\")) }\n\
           else { Choice[0](\"a\" + \"b\") };\n\
         };\n\
         score :: Choice -> Int32 := \\(choice) {\n\
           case (choice)\n\
             [0](value) { Int32(value # 0u64) }\n\
             [1](pair) {\n\
               (bias, value) := pair;\n\
               Int32(bias) + Int32(value # 1u64);\n\
             };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           score(choose(false)) + score(choose(true)) - 198;\n\
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
fn emit_c_writes_the_translation_unit_and_paired_header() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/program.c");
    directory.write("program.mal", "main :: Unit -> Int32 := \\() { 0; };");
    let output = directory.malc([
        OsStr::new("emit-c"),
        source.as_os_str(),
        OsStr::new("--output"),
        output_path.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_c = std::fs::read_to_string(output_path).unwrap();
    assert!(generated_c.starts_with("#include \"program.mal.h\"\n"));
    assert!(generated_c.contains("int main(void)"));
    let generated_header =
        std::fs::read_to_string(directory.join("generated/program.mal.h")).unwrap();
    assert!(generated_header.contains("#define MAL_C_ABI_VERSION 0x000600u"));
    assert!(generated_header.contains("_Noreturn void mal_trap("));
    assert!(generated_header.contains("mal_Symbol_t mal_Symbol_from_bytes("));
    assert!(generated_header.contains("MalType_Symbol mal_Symbol_return("));
}

#[test]
fn emit_header_writes_a_standalone_host_interface() {
    let directory = NativeFixture::new("driver-header");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;",
    );

    let output = directory.malc([
        OsStr::new("emit-header"),
        source.as_os_str(),
        OsStr::new("--output"),
        output_path.as_os_str(),
    ]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = std::fs::read_to_string(output_path).unwrap();
    assert!(header.contains("typedef MalType_UInt64 MalType_Count;"));
    assert!(header.contains("#define MAL_HAS_EXTERN_increment 1"));
    assert!(header.contains("#define MAL_DEFINE_increment(call, value)"));
    assert!(!header.contains("MAL_HAS_EXTERN_missing"));
    assert!(!directory.join("generated/program.c").exists());
}

#[test]
fn emit_header_defaults_to_the_source_directory() {
    let directory = NativeFixture::new("driver-default-header");
    let source = directory.join("source/program.mal");
    directory.write("source/program.mal", "extern print :: Symbol -> Unit;");

    let output = directory.malc([OsStr::new("emit-header"), source.as_os_str()]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = std::fs::read_to_string(directory.join("source/program.mal.h")).unwrap();
    assert!(header.contains("#define MAL_DEFINE_print(call, value)"));
}

#[test]
fn emit_host_prints_compilable_external_operation_stubs() {
    let directory = NativeFixture::new("driver-host");
    let source = directory.join("program.mal");
    let header = directory.join("custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         Request :: (Count, Int32);\n\
         extern increment :: Count -> Count;\n\
         extern inspect :: Request -> Count;\n\
         main :: Unit -> Int32 := \\() { Int32(increment(41u64) - 42u64); };",
    );

    let header_output = directory.malc([
        OsStr::new("emit-header"),
        source.as_os_str(),
        OsStr::new("--output"),
        header.as_os_str(),
    ]);
    assert!(header_output.status.success());
    let default_output = directory.malc([OsStr::new("emit-host"), source.as_os_str()]);
    assert!(default_output.status.success());
    assert!(
        default_output
            .stdout
            .starts_with(b"#include \"program.mal.h\"\n")
    );

    let output = directory.malc([
        OsStr::new("emit-host"),
        source.as_os_str(),
        OsStr::new("--header"),
        OsStr::new("custom.h"),
    ]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let host = String::from_utf8(output.stdout).unwrap();
    assert!(host.starts_with("#include \"custom.h\"\n"));
    assert!(host.contains("MAL_DEFINE_increment(call, value)"));
    assert!(host.contains("MAL_DEFINE_inspect(call, value)"));
    assert!(host.contains("(void)value;"));
    assert!(host.contains("external operation `increment` is not implemented"));

    directory.write("host.c", &host);
    let object = directory.join("host.o");
    let compilation = std::process::Command::new("clang")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic", "-c"])
        .arg(directory.join("host.c"))
        .arg("-o")
        .arg(object)
        .output()
        .expect("run clang");
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
}

#[test]
fn checked_in_example_headers_match_the_compiler() {
    let fixture = NativeFixture::new("example-headers");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler directory has a repository parent");
    let examples = [
        "fallible-tree",
        "integer-and-byte",
        "mini-database",
        "opaque-aggregate",
        "pointer-tree",
        "print-and-closure",
        "ptr-memory",
        "recoverable-file",
        "resizable-buffer",
        "socket-packet",
        "strict-float",
        "symbol-round-trip",
        "tail-recursion",
    ];

    for example in examples {
        let directory = repository.join("examples").join(example);
        let generated = fixture.join(format!("{example}.h"));
        let output = fixture.malc([
            OsStr::new("emit-header"),
            directory.join("program.mal").as_os_str(),
            OsStr::new("--output"),
            generated.as_os_str(),
        ]);
        assert!(
            output.status.success(),
            "failed to generate {example}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(generated).unwrap(),
            std::fs::read_to_string(directory.join("program.mal.h")).unwrap(),
            "checked-in header is stale for {example}"
        );
    }
}

#[test]
fn build_compiles_required_host_inputs_and_produces_an_executable() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let executable = directory.join("out/program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         require \"./host.c\";\n\
         require \"./helper.c\";\n\
         extern adjust :: Int32 -> Int32;\n\
         main :: Unit -> Int32 := \\() { adjust(40) - 42; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         int32_t host_increment(int32_t value);\n\
         MAL_DEFINE_adjust(call, value) {\n\
             return mal_Int32_return(call, host_increment(value));\n\
         }\n",
    );
    directory.write(
        "helper.c",
        "#include <stdint.h>\n\
         int32_t host_increment(int32_t value) { return value + 2; }\n",
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
    assert!(directory.run(executable).status.success());
}

#[test]
fn builds_public_functions_from_required_files_with_private_helpers() {
    let directory = NativeFixture::new("driver-required-mal");
    let source = directory.write(
        "program.mal",
        "require \"./left.mal\";\n\
         require \"./left.mal\";\n\
         require \"./right.mal\";\n\
         main :: Unit -> Int32 := \\() { left(39) + right(1) - 42 };",
    );
    directory.write(
        "left.mal",
        "_helper :: Int32 -> Int32 := \\(x) { x + 1 };\n\
         left :: Int32 -> Int32 := \\(x) { _helper(x) };",
    );
    directory.write(
        "right.mal",
        "_helper :: Int32 -> Int32 := \\(x) { x + 1 };\n\
         right :: Int32 -> Int32 := \\(x) { _helper(x) };",
    );
    let executable = directory.join("program");

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
    assert!(directory.run(executable).status.success());
}

#[test]
fn source_graph_overlays_open_mal_buffers() {
    let directory = NativeFixture::new("driver-overlays");
    let source = directory.write("program.mal", "not the open buffer");
    let dependency = directory.join("library.mal");
    let root_text = "require \"library.mal\";\nmain :: Unit -> Int32 := \\() { value - 42; };";
    let dependency_text = "value :: Int32 := 42;";
    let overlays =
        std::collections::HashMap::from([(dependency.clone(), dependency_text.to_owned())]);

    let graph = malc::driver::load_source_graph_with_overlays(&source, root_text, &overlays)
        .expect("load overlaid source graph");

    assert_eq!(graph.root_source().text(), root_text);
    assert_eq!(graph.files().len(), 2);
    assert_eq!(
        graph.source(malc::source::FileId::new(1)).unwrap().text(),
        dependency_text
    );
    malc::pipeline::check_graph(&graph).expect("check overlaid graph");
}
