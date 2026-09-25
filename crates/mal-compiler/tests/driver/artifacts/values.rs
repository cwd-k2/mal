use super::*;

#[test]
fn builds_grows_and_aliases_managed_buffers() {
    let directory = NativeFixture::new("driver-buffer");
    let source = directory.write(
        "program.mal",
        "addRange :: (Buffer<USize>, USize, USize) -> Unit := (buffer, current, end) ->
           if (current == end) then () else {
             buffer.new(current); addRange(buffer, current + 1usize, end);
           };
         main :: Unit -> Int32 := () -> {
           values := make<USize>(1usize);
           alias := values;
           values.new(0usize);
           beforeGrowth := alias.get(0usize);
           addRange(values, 1usize, 40usize);
           alias.put(0usize, 41usize);
           if (beforeGrowth == 0usize && #values == 40usize && values.get(0usize) == 41usize
               && alias.get(39usize) == 39usize) then 0 else 1;
         };",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn preserves_symbol_snapshots_and_zero_stride_buffers() {
    let directory = NativeFixture::new("driver-buffer-snapshots");
    let source = directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           text := \"abc\";
           bytes := *text;
           before := *bytes;
           bytes.put(1usize, 99u8);
           after := *bytes;
           units := make<Unit>(0usize);
           units.new(()); units.new(()); units.new(());
           units.get(2usize);
           if (text == \"abc\" && before == \"abc\" && after == \"acc\"
               && #units == 3usize) then 0 else 1;
         };",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn fills_and_copies_buffer_ranges_with_snapshot_overlap() {
    let directory = NativeFixture::new("driver-buffer-ranges");
    let source = directory.write(
        "program.mal",
        "main :: Unit -> Int32 := () -> {
           values := make<UInt8>(8usize);
           values.fill(0usize, 4usize, 1u8);
           alias := values;
           alias.fill(1usize, 4usize, 2u8);
           source := make<UInt8>(4usize);
           source.new(10u8);
           source.new(20u8);
           source.new(30u8);
           source.new(40u8);
           values.copy(1usize, source, 1usize, 3usize);
           copied := values.get(0usize) == 1u8 && values.get(1usize) == 20u8
             && values.get(2usize) == 30u8 && values.get(3usize) == 40u8
             && values.get(4usize) == 2u8;
           // Both overlap directions observe the source range before the copy begins.
           values.copy(2usize, values, 0usize, 3usize);
           forward := values.get(2usize) == 1u8 && values.get(3usize) == 20u8
             && values.get(4usize) == 30u8;
           values.copy(0usize, values, 2usize, 3usize);
           reverse := values.get(0usize) == 1u8 && values.get(1usize) == 20u8
             && values.get(2usize) == 30u8;
           values.copy(5usize, values, 0usize, 5usize);
           units := make<Unit>(0usize);
           units.fill(0usize, 7usize, ());
           unitCopy := make<Unit>(0usize);
           unitCopy.copy(0usize, units, 2usize, 4usize);
           zeros := make<UInt8>(16usize);
           zeros.fill(0usize, 16usize, 0u8);
           zeros.fill(16usize, 0usize, 9u8);
           zeroGrowth := make<UInt8>(1usize);
           zeroGrowth.new(9u8);
           zeroGrowth.fill(0usize, 64usize, 0u8);
           pairs := make<(UInt8, UInt32)>(0usize);
           pairs.fill(0usize, 3usize, (7u8, 42u32));
           pairCopies := make<(UInt8, UInt32)>(0usize);
           pairCopies.copy(0usize, pairs, 1usize, 2usize);
           pairCopies.copy(2usize, pairs, 3usize, 0usize);
           (first, second) := pairCopies.get(1usize);
           if (copied && forward && reverse && #values == 10usize
               && values.get(9usize) == 30u8 && #units == 7usize
               && #unitCopy == 4usize && zeros.get(15usize) == 0u8
               && #zeroGrowth == 64usize && zeroGrowth.get(0usize) == 0u8
               && zeroGrowth.get(63usize) == 0u8
               && #pairCopies == 2usize && first == 7u8 && second == 42u32) then 0 else 1;
         };",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn transfers_between_c_host_storage_and_buffers() {
    let directory = NativeFixture::new("driver-buffer-host-copy");
    let source = directory.write(
        "program.mal",
        "require \"host.c\";
         extern sourceMemory :: Unit -> Address;
         extern targetMemory :: Unit -> Address;
         extern verify :: Unit -> Bool;
         main :: Unit -> Int32 := () -> {
           bytes := from<UInt8>(sourceMemory(), 0usize, 3usize);
           bytes.into(targetMemory(), 0usize, 3usize);
           if (bytes.get(0usize) + bytes.get(2usize) == 42u8 && verify()) then 0 else 1;
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t source_bytes[] = {20, 0, 22};\n\
         static uint8_t target_bytes[3];\n\
         MAL_DEFINE_sourceMemory(call) { return mal_Address_return(call, source_bytes); }\n\
         MAL_DEFINE_targetMemory(call) { return mal_Address_return(call, target_bytes); }\n\
         MAL_DEFINE_verify(call) { return mal_Bool_return(call, target_bytes[0] == 20 && target_bytes[2] == 22); }\n",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn transfers_zero_stride_units_from_a_one_past_address() {
    let directory = NativeFixture::new("driver-unit-buffer-copy");
    let source = directory.write(
        "program.mal",
        "require \"host.c\";
         extern onePast :: Unit -> Address;
         main :: Unit -> Int32 := () -> {
           address := onePast();
           empty := from<UInt8>(address, 0usize, 0usize);
           empty.into(address, 0usize, 0usize);
           units := from<Unit>(address, 0usize, 7usize);
           units.into(address, 0usize, 7usize);
           if (#empty == 0usize && #units == 7usize) then 0 else 1;
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         static uint8_t byte;\n\
         MAL_DEFINE_onePast(call) { return mal_Address_return(call, &byte + 1); }\n",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}

#[test]
fn stores_symbols_and_symbol_aggregates_in_buffers() {
    let directory = NativeFixture::new("driver-buffer-symbols");
    let source = directory.write(
        "program.mal",
        "Entry :: (Symbol, Int32);
         Maybe :: [Unit, Symbol];
         main :: Unit -> Int32 := () -> {
           names := make<Symbol>(1usize);
           alias := names;
           names.new(\"ab\" + \"cd\");
           names.new(\"efg\");
           names.fill(2usize, 3usize, \"x\" + \"yz\");
           alias.put(0usize, \"long\" + \"er\");
           names.copy(1usize, names, 0usize, 4usize);
           names.put(1usize, names.get(1usize) + \"!\");
           held := names.get(1usize);
           names.put(1usize, \"gone\");
           entries := make<Entry>(0usize);
           entries.new((\"pair\" + \"ed\", 7i32));
           entries.fill(1usize, 2usize, (\"q\" + \"q\", 1i32));
           (label, number) := entries.get(1usize);
           some :: Maybe := [none, some] => some(\"opt\" + \"ion\");
           nothing :: Maybe := [none, some] => none();
           options := make<Maybe>(0usize);
           options.new(some);
           options.new(nothing);
           options.fill(0usize, 2usize, some);
           if (held == \"longer!\" && names.get(1usize) == \"gone\" && names.get(0usize) == \"longer\"
               && names.get(2usize) == \"efg\" && names.get(4usize) == \"xyz\" && #names == 5usize
               && label == \"qq\" && number == 1i32 && #options == 2usize) then 0 else 1;
         };",
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
    assert_eq!(directory.run(executable).status.code(), Some(0));
}
