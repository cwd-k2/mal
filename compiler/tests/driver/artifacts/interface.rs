use super::*;
#[test]
fn emit_header_writes_a_standalone_host_interface() {
    let directory = NativeFixture::new("driver-header");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/custom.h");
    directory.write(
        "program.mal",
        "Later :: Earlier;\n\
         Earlier :: UInt8;\n\
         Nothing :: Unit;\n\
         Flag :: Bool;\n\
         Choice :: [Unit, UInt32];\n\
         Count :: UInt64;\n\
         Bytes :: (Address, USize);\n\
         _Internal :: Bytes;\n\
         Managed :: (Int64, Symbol);\n\
         extern increment :: Count -> Count;\n\
         extern consume :: Bytes -> USize;\n\
         internal :: Symbol := \"mal-owned\";",
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
    assert!(header.contains("typedef mal_repr_product_0_t mal_Bytes_t;"));
    assert!(header.contains("mal_Count_read(mal_call_t *call, mal_Address_t address"));
    assert!(header.contains("mal_Bytes_write(mal_call_t *call, mal_Address_t address"));
    assert!(!header.contains("mal__Internal_read"));
    assert!(!header.contains("mal__Internal_t"));
    assert!(!header.contains("mal_Managed_read"));
    assert!(header.contains("#define MAL_C_ABI_VERSION 0x000800u"));
    assert!(header.contains("#define MAL_HAS_EXTERN_increment 1"));
    assert!(header.contains("#define MAL_HAS_EXTERN_consume 1"));
    assert!(header.contains("#define MAL_DEFINE_increment(call, value)"));
    assert!(!header.contains("Symbol"));
    assert!(!header.contains("MAL_HAS_EXTERN_missing"));
    assert!(!directory.join("generated/program.c").exists());
}

#[test]
fn emit_header_defaults_to_the_source_directory() {
    let directory = NativeFixture::new("driver-default-header");
    let source = directory.join("source/program.mal");
    directory.write(
        "source/program.mal",
        "extern print :: (Address, USize) -> Unit;",
    );

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
        "Later :: Earlier;\n\
         Earlier :: UInt8;\n\
         Nothing :: Unit;\n\
         Flag :: Bool;\n\
         Choice :: [Unit, UInt32];\n\
         Count :: UInt64;\n\
         Request :: (Count, Int32);\n\
         Empty :: [];\n\
         extern increment :: Count -> Count;\n\
         extern inspect :: Request -> Count;\n\
         extern consumeEmpty :: Empty -> Unit;\n\
         extern produceEmpty :: Unit -> Empty;\n\
         main :: Unit -> Int32 := () -> { (increment(41u64) - 42u64).i32; };",
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
    assert!(host.contains("MAL_DEFINE_consumeEmpty(call, value)"));
    assert!(host.contains("MAL_DEFINE_produceEmpty(call)"));
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
        "brainfuck-llvm",
        "fallible-tree",
        "numeric-conversion",
        "json-query",
        "mini-database",
        "opaque-aggregate",
        "packed-tree",
        "print-and-closure",
        "typed-memory",
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
         main :: Unit -> Int32 := () -> { adjust(40) - 42; };",
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
         main :: Unit -> Int32 := () -> { left(39) + right(1) - 42 };",
    );
    directory.write(
        "left.mal",
        "require \"./left-types.mal\";\n\
         _helper :: Int32 -> Int32 := (x) -> { x + 1 };\n\
         left :: Int32 -> Int32 := (x) -> { _helper(x) };",
    );
    directory.write(
        "right.mal",
        "require \"./right-types.mal\";\n\
         _helper :: Int32 -> Int32 := (x) -> { x + 1 };\n\
         right :: Int32 -> Int32 := (x) -> { _helper(x) };",
    );
    directory.write("left-types.mal", "Shared :: (Int64, UInt8);");
    directory.write("right-types.mal", "Shared :: (UInt64, UInt8);");
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
fn specializes_a_generic_declared_in_a_required_file() {
    let directory = NativeFixture::new("driver-required-generic");
    let source = directory.write(
        "program.mal",
        "require \"./library.mal\";\n\
         main :: Unit -> Int32 := () -> identity<Int32>(42) - identity<Int32>(42);",
    );
    directory.write("library.mal", "identity<A> :: A -> A := (value) -> value;");
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
    let root_text = "require \"library.mal\";\nmain :: Unit -> Int32 := () -> { value - 42; };";
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
