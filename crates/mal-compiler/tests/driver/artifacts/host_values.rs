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
         static void retain_symbol(MalContext *context, void *carrier) {\n\
             mal_Symbol_t *value = carrier;\n\
             (void)mal_runtime_bytes_retain(context, value->owner);\n\
         }\n\
         static void release_symbol(void *carrier) {\n\
             mal_Symbol_t *value = carrier;\n\
             mal_runtime_bytes_release(value->owner);\n\
         }\n\
         MAL_DEFINE_words(call) {\n\
             mal_Buffer_t values = mal_Buffer_make_managed(\n\
                 call, sizeof(mal_Symbol_t), 2, retain_symbol, release_symbol\n\
             );\n\
             mal_Symbol_t first = mal_Symbol_from_bytes(call, \"mal\", 3);\n\
             mal_Symbol_t second = mal_Symbol_from_bytes(call, \"runtime\", 7);\n\
             mal_Buffer_new_managed_move(call, values, &first, sizeof(first));\n\
             mal_Buffer_new_managed_move(call, values, &second, sizeof(second));\n\
             return mal_Buffer_return_move(call, values);\n\
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
             return mal_Handle_return(call, mal_Handle_from_bits((uintptr_t)value));\n\
         }\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.tag != mal_Choice_tag_1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_Packet_t packet = value.payload.variant_1;\n\
             uintptr_t bits = mal_Handle_to_bits(packet.field_1);\n\
             return mal_Choice_return_1(\n\
                 call,\n\
                 (mal_Packet_t){\n\
                     .field_0 = packet.field_0,\n\
                     .field_1 = mal_Handle_from_bits(bits + (uintptr_t)1),\n\
                 }\n\
             );\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             return mal_UInt64_return(call, (uint64_t)mal_Handle_to_bits(value));\n\
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
             return mal_Handle_return(call, mal_Handle_from_bits((uintptr_t)value));\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             return mal_UInt64_return(call, (uint64_t)mal_Handle_to_bits(value));\n\
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
