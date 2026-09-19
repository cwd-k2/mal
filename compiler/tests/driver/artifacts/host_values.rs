use super::*;

#[test]
fn accesses_canonical_memory_through_named_alias_helpers() {
    let directory = NativeFixture::new("driver-canonical-memory-helpers");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Sample :: (Int64, UInt8);\n\
         extern storage :: Unit -> Address;\n\
         extern inspect :: Address -> Unit;\n\
         main :: Unit -> Int32 := () -> {\n\
           address := storage();\n\
           view<(Int64, UInt8)>(address, 0usize, 1usize, (region) -> {\n\
             region.put(0usize, (41i64, 1u8));\n\
             inspect(address);\n\
             (number, byte) := region.get(0usize);\n\
             (number + byte.i64 - 44i64).i32;\n\
           });\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[16];\n\
         MAL_DEFINE_storage(call) {\n\
             return mal_Address_return(call, bytes);\n\
         }\n\
         MAL_DEFINE_inspect(call, address) {\n\
             mal_Sample_t sample = mal_Sample_read(call, address, 0);\n\
             if (sample.field_0 != 41 || sample.field_1 != 1) {\n\
                 mal_call_trap(call, \"unexpected canonical value\");\n\
             }\n\
             sample.field_0 += 1;\n\
             sample.field_1 += 1;\n\
             mal_Sample_write(call, address, 0, sample);\n\
             return mal_Unit_return(call);\n\
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
fn bridges_bytes_through_borrowed_addresses_in_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-byte-address-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern memory :: Unit -> Address;\n\
         extern inspect :: (Address, USize) -> UInt8;\n\
         main :: Unit -> Int32 := () -> {\n\
           address := memory();\n\
           bytes := *(\"x\" + \"y\");\n\
           view<UInt8>(address, 0usize, #bytes, (region) -> { _ := region.set(bytes); (); });\n\
           inspect(address, #bytes).i32 - 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[2];\n\
         MAL_DEFINE_memory(call) {\n\
             return mal_Address_return(call, bytes);\n\
         }\n\
         MAL_DEFINE_inspect(call, value) {\n\
             uint8_t valid = value.field_0 == bytes\n\
                 && value.field_1 == 2\n\
                 && bytes[0] == 'x'\n\
                 && bytes[1] == 'y';\n\
             return mal_UInt8_return(call, valid);\n\
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
fn marshals_address_products_through_the_public_c_abi() {
    let directory = NativeFixture::new("driver-llvm-product-extern");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         Packet :: (UInt64, Address, USize);\n\
         extern memory :: Unit -> Address;\n\
         extern exchange :: Packet -> Packet;\n\
         main :: Unit -> Int32 := () -> {\n\
           address := memory();\n\
           bytes := *\"ab\";\n\
           view<UInt8>(address, 0usize, #bytes, (region) -> { _ := region.set(bytes); (); });\n\
           (number, returned, length) := exchange(41u64, address, #bytes);\n\
           if (number == 42u64 && *pack<UInt8>(returned, 0usize, length) == \"ab\")\n\
           then 0\n\
           else 1;\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[2];\n\
         MAL_DEFINE_memory(call) {\n\
             return mal_Address_return(call, bytes);\n\
         }\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.field_1 != bytes || value.field_2 != 2\n\
                 || bytes[0] != 'a' || bytes[1] != 'b') {\n\
                 mal_call_trap(call, \"unexpected packet\");\n\
             }\n\
             return mal_Packet_return(\n\
                 call,\n\
                 (mal_Packet_t){\n\
                     .field_0 = value.field_0 + 1,\n\
                     .field_1 = value.field_1,\n\
                     .field_2 = value.field_2,\n\
                 }\n\
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
         Bytes :: (Address, USize);\n\
         Choice :: [Unit, (UInt64, Bytes)];\n\
         Envelope :: (UInt8, Choice);\n\
         extern memory :: Unit -> Address;\n\
         extern exchange :: Envelope -> Envelope;\n\
         makeChoice :: (UInt64, Bytes) -> Choice := (value) -> [none, some] => { some(value) };\n\
         main :: Unit -> Int32 := () -> {\n\
           address := memory();\n\
           bytes := *\"ab\";\n\
           view<UInt8>(address, 0usize, #bytes, (region) -> { _ := region.set(bytes); (); });\n\
           (number, choice) := exchange(41u8, makeChoice(7u64, (address, #bytes)));\n\
           choice[\n\
             () -> { 1 },\n\
             (packet) -> {\n\
               (bias, returned) := packet;\n\
               (returnedAddress, length) := returned;\n\
               if (number == 42u8) then {\n\
                 if (bias == 7u64) then {\n\
                   if (*pack<UInt8>(returnedAddress, 0usize, length) == \"ab\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             }];\n\
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t bytes[2];\n\
         MAL_DEFINE_memory(call) {\n\
             return mal_Address_return(call, bytes);\n\
         }\n\
         MAL_DEFINE_exchange(call, value) {\n\
             if (value.field_1.tag != mal_Choice_tag_1) {\n\
                 mal_call_trap(call, \"unexpected choice\");\n\
             }\n\
             mal_Bytes_t payload = value.field_1.payload.variant_1.field_1;\n\
             if (payload.field_0 != bytes || payload.field_1 != 2\n\
                 || bytes[0] != 'a' || bytes[1] != 'b') {\n\
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
