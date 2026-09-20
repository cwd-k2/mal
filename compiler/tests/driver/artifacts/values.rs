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
