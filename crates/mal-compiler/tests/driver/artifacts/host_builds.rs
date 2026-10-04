use super::*;

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
fn applies_an_external_operation_passed_as_a_value_in_every_mode() {
    let directory = NativeFixture::new("driver-first-class-extern");
    let source = directory.join("program.mal");
    directory.write(
        "program.mal",
        "require \"./host.c\";\n\
         extern adjust :: Int32 -> Int32;\n\
         apply :: ((Int32 -> Int32), Int32) -> Int32 := (operation, value) -> { operation(value); };\n\
         main :: Unit -> Int32 := () -> { chosen := adjust; apply(chosen, 40) - 42; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_adjust(call, value) { return mal_Int32_return(call, value + 2); }\n",
    );

    for profile in ["baseline", "production"] {
        let executable = directory.join(profile);
        let output = directory.malc([
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
            OsStr::new("--optimization"),
            OsStr::new(profile),
        ]);
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(directory.run(executable).status.success(), "{profile}");
    }
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

    let graph = mal_syntax::graph::load_with_overlays(&source, root_text, &overlays)
        .expect("load overlaid source graph");

    assert_eq!(graph.root_source().text(), root_text);
    assert_eq!(graph.files().len(), 2);
    assert_eq!(
        graph
            .source(mal_syntax::source::FileId::new(1))
            .unwrap()
            .text(),
        dependency_text
    );
    mal_frontend::analysis::check_graph(&graph).expect("check overlaid graph");
}
