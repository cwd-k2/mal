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
           addRange(values, 0usize, 40usize);
           alias.put(0usize, 41usize);
           if (#values == 40usize && values.get(0usize) == 41usize
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
