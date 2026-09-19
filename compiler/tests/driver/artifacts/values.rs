use super::*;

#[test]
fn builds_packed_slices_indexing_and_symbol_conversion() {
    let directory = NativeFixture::new("driver-packed");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {\n\
           packed := *\"abc\";\n\
           prefix := packed / 2usize;\n\
           remainder := packed % 2usize;\n\
           text := *prefix;\n\
           (prefix # 1usize).i32 + (remainder # 0usize).i32 + (#text).i32;\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(199));
}

#[test]
fn constructs_and_edits_packed_values_with_scoped_buffers() {
    let directory = NativeFixture::new("driver-packed-builder");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "addRange :: (Buffer<USize>, USize, USize) -> Unit :=
           (buffer, current, end) ->
             if (current == end)
             then ()
             else {
               _ := buffer.new(current);
               addRange(buffer, current + 1usize, end);
             };
         main :: Unit -> Int32 := () -> {
           original := make<Int32>(0usize, (buffer) -> {
             first := buffer.new(10i32);
             _ := buffer.new(20i32);
             buffer.put(first, buffer.get(first) + 1i32);
             ();
           });
           updated := original.edit<Int32>((buffer) -> {
             buffer.put(0usize, buffer.get(0usize) + 30i32);
             added := buffer.new(7i32);
             buffer.put(added, buffer.get(added) + 1i32);
             ();
           });
           many := make<USize>(0usize, (buffer) -> addRange(buffer, 0usize, 40usize));
           inner := make<Int32>(0usize, (innerBuffer) -> {
             _ := innerBuffer.new(9i32);
             ();
           });
           nested := make<Int32>(0usize, (buffer) -> {
             outer := buffer.new(5i32);
             buffer.put(outer, inner # 0usize);
             ();
           });
           if (#original == 2usize && original # 0usize == 11i32
               && original # 1usize == 20i32 && #updated == 3usize
               && updated # 0usize == 41i32 && updated # 1usize == 20i32
               && updated # 2usize == 8i32 && #many == 40usize
               && many # 39usize == 39usize && nested # 0usize == 9i32)
           then 0
           else 1;
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
fn preserves_shared_sources_and_builds_zero_stride_packed_values() {
    let directory = NativeFixture::new("driver-packed-builder-sharing");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           text := \"abc\";
           bytes := *text;
           changed := bytes.edit<UInt8>((buffer) -> {
             buffer.put(1usize, buffer.get(1usize) + 1u8);
             ();
           });
           unchanged := bytes.edit<UInt8>((_) -> ());
           units := make<Unit>(0usize, (buffer) -> {
             _ := buffer.new(());
             _ := buffer.new(());
             _ := buffer.new(());
             ();
           });
           if (text == \"abc\" && *bytes == \"abc\" && *unchanged == \"abc\"
               && changed # 1usize == 99u8 && #units == 3usize)
           then 0
           else 1;
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
fn transfers_between_regions_and_packed_storage() {
    let directory = NativeFixture::new("driver-region-packed");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"host.c\";\n\
         extern sourceMemory :: Unit -> Address;\n\
         extern targetMemory :: Unit -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           packed := pack<UInt8>(sourceMemory(), 0usize, 3usize);\n\
           view<UInt8>(targetMemory(), 0usize, 3usize, (target) -> { _ := target.set(packed); (); });\n\
           (packed # 0usize).i32 + (packed # 2usize).i32;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t source_bytes[] = { 20, 0, 22 };\n\
         static uint8_t target_bytes[3];\n\
         MAL_DEFINE_sourceMemory(call) {\n\
             return mal_Address_return(call, source_bytes);\n\
         }\n\
         MAL_DEFINE_targetMemory(call) {\n\
             return mal_Address_return(call, target_bytes);\n\
         }\n",
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
fn transfers_zero_stride_units_from_a_one_past_address() {
    let directory = NativeFixture::new("driver-unit-region-packed");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"host.c\";\n\
         extern onePast :: Unit -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           address := onePast();\n\
           packed := pack<Unit>(address, 0usize, 7usize);\n\
           remainderLength := view<Unit>(address, 0usize, 7usize, (region) -> #region.set(packed));\n\
           if (#packed == 7usize && remainderLength == 0usize)\n\
           then 0\n\
           else 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t byte;\n\
         MAL_DEFINE_onePast(call) { return mal_Address_return(call, &byte + 1); }\n\
",
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
fn converts_static_and_dynamic_symbols_through_packed_views() {
    let directory = NativeFixture::new("driver-symbol-packed-round-trip");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "roundTrip :: Symbol -> Symbol := (value) -> { packed := *value; *packed };\n\
         main :: Unit -> Int32 := () -> {\n\
           flat := \"flat\";\n\
           dynamic := \"left\" + \"right\";\n\
           flatCopy := roundTrip(flat);\n\
           dynamicCopy := roundTrip(dynamic);\n\
           if (flat == \"flat\" && flatCopy == flat && dynamic == \"leftright\" && dynamicCopy == dynamic)\n\
           then 0\n\
           else 1;\n\
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
fn stores_and_loads_canonical_products_and_sums() {
    let directory = NativeFixture::new("driver-canonical-layout");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"host.c\";\n\
         Choice :: [Unit, UInt64];\n\
         extern memory :: Unit -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           address := memory();\n\
           (first, second) := view<(UInt8, UInt64)>(address, 0usize, 1usize, (product) -> {\n\
             product.put(0usize, (7u8, 35u64));\n\
             product.get(0usize);\n\
           });\n\
           choice :: Choice := [none, some] => some(42u64);\n\
           loaded := view<Choice>(address + #(u8, u64), 0usize, 1usize, (sum) -> {\n\
             sum.put(0usize, choice);\n\
             sum.get(0usize);\n\
           });\n\
           selected := loaded[() -> 0i32, (value) -> value.i32];\n\
           first.i32 + second.i32 + selected;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[32];\n\
         MAL_DEFINE_memory(call) { return mal_Address_return(call, bytes); }\n",
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
    assert_eq!(directory.run(executable).status.code(), Some(84));
}

#[test]
fn accesses_an_unaligned_region_with_pointer_provenance() {
    let directory = NativeFixture::new("driver-region-unaligned");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"host.c\";\n\
         extern memory :: Unit -> Address;\n\
         main :: Unit -> Int32 := () ->\n\
           view<UInt64>(memory() + 1bytes, 0usize, 1usize, (region) -> {\n\
             region.put(0usize, 42u64);\n\
             region.get(0usize).i32;\n\
           });",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[24];\n\
         MAL_DEFINE_memory(call) { return mal_Address_return(call, bytes); }\n",
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
           (-1i8).u64.i32 + 1;\n\
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
           3i32.f64.i32 + 1.75f64.f32.i32 - 7;\n\
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
         extern memory :: ByteSize -> Address;\n\
         main :: Unit -> Int32 := () -> {\n\
           base := memory(64bytes);\n\
           _ := view<UInt64>(base, 0usize, 1usize, (region) -> region.put(0usize, 42u64));\n\
           pointerSlot := base + #u64;\n\
           _ := view<Address>(pointerSlot, 0usize, 1usize, (region) -> region.put(0usize, base));\n\
           floatSlot := pointerSlot + #address;\n\
           _ := view<Float32>(floatSlot, 0usize, 1usize, (region) -> region.put(0usize, 1.5f32));\n\
           restored := view<Address>(pointerSlot, 0usize, 1usize, (region) -> region.get(0usize));\n\
           first := view<UInt64>(restored, 0usize, 1usize, (region) -> region.get(0usize));\n\
           second := view<UInt64>(base, 0usize, 1usize, (region) -> region.get(0usize));\n\
           float := view<Float32>(floatSlot, 0usize, 1usize, (region) -> region.get(0usize));\n\
           value := first + second;\n\
           if (float == 1.5f32) then { value.i32 - 84 } else { 1 };\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static unsigned char storage[65];\n\
         MAL_DEFINE_memory(call, size) {\n\
             (void)size;\n\
             return mal_Address_return(call, storage + 1);\n\
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
