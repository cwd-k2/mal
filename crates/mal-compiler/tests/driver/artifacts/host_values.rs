use super::*;

#[test]
fn constructs_a_managed_element_buffer_in_the_c_runtime_extension() {
    let directory = NativeFixture::new("driver-c-managed-buffer");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern words :: Unit -> Buffer<Symbol>;\n\
         main :: Unit -> Int32 := () -> {\n\
           values := words();\n\
           if (#values == 2usize && values.get(0usize) == \"mal\" && values.get(1usize) == \"runtime\")\n\
           then 0\n\
           else 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_words(call) {\n\
             mal_owned(Buffer) values = mal_buffer(call, mal_storage(mal_type(Symbol)), 2);\n\
             mal_push(call, values, mal_symbol(call, \"discard\", 7));\n\
             mal_replace(call, values, 0, mal_symbol(call, \"mal\", 3));\n\
             mal_fill(call, values, 1, 1, mal_symbol(call, \"runtime\", 7));\n\
             mal_owned(Buffer) copied = mal_buffer(call, mal_storage(mal_type(Symbol)), 2);\n\
             mal_copy(call, copied, 0, values, 0, 2);\n\
             mal_append(call, copied, mal_data(values), 2);\n\
             mal_truncate(copied, 2);\n\
             mal_reserve(call, copied, 8);\n\
             mal_type(Buffer) shared = mal_share(call, copied);\n\
             mal_drop(shared);\n\
             return mal_move(copied);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn constructs_nested_buffers_with_the_erased_host_carrier() {
    let directory = NativeFixture::new("driver-c-nested-buffer");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern nested :: Unit -> Buffer<Buffer<UInt8>>;\n\
         main :: Unit -> Int32 := () -> {\n\
           values := nested();\n\
           if (#values == 1usize && #values.get(0usize) == 2usize\n\
             && values.get(0usize).get(0usize) == 40u8\n\
             && values.get(0usize).get(1usize) == 2u8)\n\
           then 0\n\
           else 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_nested(call) {\n\
             mal_owned(Buffer) inner = mal_buffer(call, mal_storage(mal_type(UInt8)), 2);\n\
             mal_type(UInt8) *tail = mal_extend(call, inner, 2);\n\
             tail[0] = UINT8_C(40);\n\
             tail[1] = UINT8_C(2);\n\
             mal_type(Buffer) outer = mal_buffer(call, mal_storage(mal_type(Buffer)), 1);\n\
             mal_push(call, outer, mal_move(inner));\n\
             return mal_move(outer);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn overwrites_truncated_host_buffer_storage_with_zero_values() {
    let directory = NativeFixture::new("driver-c-truncated-zero-buffer");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern bytes :: Unit -> Buffer<UInt8>;\n\
         main :: Unit -> Int32 := () -> { bytes().get(0usize).i32; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_bytes(call) {\n\
             mal_owned(Buffer) bytes = mal_buffer(call, mal_storage(mal_type(UInt8)), 8);\n\
             mal_push(call, bytes, UINT8_C(7));\n\
             mal_truncate(bytes, 0);\n\
             mal_push(call, bytes, UINT8_C(0));\n\
             return mal_move(bytes);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn constructs_buffers_of_aligned_sums_with_the_public_c_carrier() {
    let directory = NativeFixture::new("driver-c-aligned-sum-buffer");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Choice :: [Buffer<UInt8>, UInt32];\n\
         extern choices :: Unit -> Buffer<Choice>;\n\
         main :: Unit -> Int32 := () -> {\n\
           values := choices();\n\
           values.get(0usize)[\n\
             (bytes) -> { bytes.get(0usize).i32 - 42 },\n\
             (_) -> { 1 }];\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_choices(call) {\n\
             mal_owned(Buffer) bytes = mal_buffer(call, mal_storage(mal_type(UInt8)), 1);\n\
             mal_push(call, bytes, UINT8_C(42));\n\
             mal_owned(Choice) choice = {\n\
                 .tag = 0,\n\
                 .payload.variant_0 = mal_move(bytes),\n\
             };\n\
             mal_owned(Buffer) values = mal_buffer(call, mal_storage(mal_type(Choice)), 1);\n\
             mal_push(call, values, mal_move(choice));\n\
             return mal_move(values);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn lexically_owns_a_named_managed_aggregate_in_c() {
    let directory = NativeFixture::new("driver-c-owned-aggregate");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Packet :: (Buffer<Symbol>, Symbol);\n\
         extern packet :: Unit -> Packet;\n\
         main :: Unit -> Int32 := () -> {\n\
           (values, label) := packet();\n\
           if (#values == 1usize && values.get(0usize) == \"mal\" && label == \"runtime\")\n\
           then 0\n\
           else 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_packet(call) {\n\
             mal_owned(Packet) discarded = {\n\
                 .field_0 = mal_buffer(call, mal_storage(mal_type(Symbol)), 0),\n\
                 .field_1 = mal_symbol(call, \"discarded\", 9),\n\
             };\n\
             (void)discarded;\n\
             mal_owned(Packet) packet = {\n\
                 .field_0 = mal_buffer(call, mal_storage(mal_type(Symbol)), 1),\n\
                 .field_1 = mal_symbol(call, \"runtime\", 7),\n\
             };\n\
             mal_push(call, packet.field_0, mal_symbol(call, \"mal\", 3));\n\
             return mal_move(packet);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn transfers_external_opaque_values_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-opaque-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern Handle;\n\
         Packet :: (UInt8, Handle);\n\
         Choice :: [Unit, Packet];\n\
         extern create :: UInt64 -> Handle;\n\
         extern exchange :: Choice -> Choice;\n\
         extern inspect :: Handle -> UInt64;\n\
         makeChoice :: Packet -> Choice := (value) -> [none, some] => { some(value) };\n\
         main :: Unit -> Int32 := () -> {\n\
           choice := exchange(makeChoice(1u8, create(40u64)));\n\
           choice[\n\
             () -> { 1 },\n\
             (packet) -> {\n\
               (bias, handle) := packet;\n\
               (inspect(handle) + bias.u64 - 42u64).i32;\n\
             }];\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_create(call, value) {\n\
             return mal_from_bits(mal_type(Handle), (uintptr_t)value);\n\
         }\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.tag != 1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_type(Packet) packet = value.payload.variant_1;\n\
             uintptr_t bits = mal_bits(packet.field_1);\n\
             return (mal_type(Choice)){\n\
                 .tag = 1,\n\
                 .payload.variant_1 = (mal_type(Packet)){\n\
                     .field_0 = packet.field_0,\n\
                     .field_1 = mal_from_bits(mal_type(Handle), bits + (uintptr_t)1),\n\
                 },\n\
             };\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             return (uint64_t)mal_bits(value);\n\
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
fn stores_external_opaque_values_in_buffers_without_a_lifecycle_callback() {
    let directory = NativeFixture::new("driver-buffer-external-carrier");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern Handle;\n\
         extern create :: UInt64 -> Handle;\n\
         extern inspect :: Handle -> UInt64;\n\
         main :: Unit -> Int32 := () -> {\n\
           values := make<Handle>(1usize);\n\
           values.new(create(1u64));\n\
           values.fill(1usize, 2usize, create(2u64));\n\
           values.put(0usize, create(40u64));\n\
           values.copy(1usize, values, 0usize, 2usize);\n\
           total := inspect(values.get(0usize)) + inspect(values.get(1usize))\n\
             + inspect(values.get(2usize));\n\
           (total - 82u64).i32;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_create(call, value) {\n\
             return mal_from_bits(mal_type(Handle), (uintptr_t)value);\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             return (uint64_t)mal_bits(value);\n\
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn owns_symbols_nested_in_products_through_llvm() {
    let directory = NativeFixture::new("driver-llvm-symbol-product");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "inspect :: (Symbol, USize) -> (Symbol, UInt8) := (input) -> {\n\
           (value, index) := input;\n\
           (value, value # index);\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           joined := \"ab\" + \"cd\";\n\
           (copy, byte) := inspect(joined, 2usize);\n\
           if (copy == \"abcd\") then { byte.i32 - 99 } else { 1 };\n\
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
         choose :: Bool -> Choice := (second) -> [firstReturn, secondReturn] => {\n\
           when (second) { secondReturn(2u64, \"b\" + \"c\") };\n\
           firstReturn(\"a\" + \"b\")\n\
         };\n\
         score :: Choice -> Int32 := (choice) -> {\n\
           choice[\n\
             (value) -> { (value # 0usize).i32 },\n\
             (pair) -> {\n\
               (bias, value) := pair;\n\
               bias.i32 + (value # 1usize).i32;\n\
             }];\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
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
