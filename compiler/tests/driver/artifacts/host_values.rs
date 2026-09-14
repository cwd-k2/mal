use super::*;
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
         main :: Unit -> Int32 := () -> {\n\
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
         main :: Unit -> Int32 := () -> {\n\
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
         makeChoice :: (UInt64, Symbol) -> Choice := (value)[none, some] -> { some(value) };\n\
         main :: Unit -> Int32 := () -> {\n\
           (number, choice) := exchange(41u8, makeChoice(7u64, \"a\" + \"b\"));\n\
           choice[\n\
             () -> { 1 },\n\
             (packet) -> {\n\
               (bias, text) := packet;\n\
               if (number == 42u8) then {\n\
                 if (bias == 7u64) then {\n\
                   if (text == \"ab\") then { 0 } else { 2 };\n\
                 } else { 3 };\n\
               } else { 4 };\n\
             }];\n\
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
         makeChoice :: Packet -> Choice := (value)[none, some] -> { some(value) };\n\
         main :: Unit -> Int32 := () -> {\n\
           choice := exchange(makeChoice(1u8, create(40u64)));\n\
           choice[\n\
             () -> { 1 },\n\
             (packet) -> {\n\
               (bias, handle) := packet;\n\
               Int32(inspect(handle) + UInt64(bias) - 42u64);\n\
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
        "inspect :: (Symbol, UInt64) -> (Symbol, UInt8) := (input) -> {\n\
           (value, index) := input;\n\
           (value, value # index);\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
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
         choose :: Bool -> Choice := (second)[firstReturn, secondReturn] -> {\n\
           when (second) { secondReturn(2u64, \"b\" + \"c\") };\n\
           firstReturn(\"a\" + \"b\")\n\
         };\n\
         score :: Choice -> Int32 := (choice) -> {\n\
           choice[\n\
             (value) -> { Int32(value # 0u64) },\n\
             (pair) -> {\n\
               (bias, value) := pair;\n\
               Int32(bias) + Int32(value # 1u64);\n\
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
