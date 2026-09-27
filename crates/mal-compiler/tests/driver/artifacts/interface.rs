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
         extern _privateConsume :: Bytes -> USize;\n\
         internal :: Symbol := \"mal-owned\";",
    );

    let output = directory.malc([
        OsStr::new("emit"),
        OsStr::new("header"),
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
    assert!(header.contains("typedef mal_repr_product_"));
    assert!(header.contains("_t mal_Bytes_t;"));
    assert!(header.contains("mal_Count_read(mal_call_t *call, mal_Address_t address"));
    assert!(header.contains("mal_Bytes_write(mal_call_t *call, mal_Address_t address"));
    assert!(!header.contains("mal__Internal_read"));
    assert!(!header.contains("mal__Internal_t"));
    assert!(!header.contains("mal_Managed_read"));
    assert!(header.contains("#include <mal.h>"));
    assert!(header.contains("MAL_C_ABI_VERSION == 0x000900u"));
    assert!(header.contains("#define MAL_HAS_EXTERN_increment 1"));
    assert!(header.contains("#define MAL_HAS_EXTERN_consume 1"));
    assert!(header.contains("#define MAL_HAS_EXTERN__privateConsume 1"));
    assert!(header.contains("#define MAL_DEFINE_increment(call, value)"));
    assert!(header.contains("#define MAL_DEFINE__privateConsume(call, value)"));
    assert!(!header.contains("Symbol"));
    assert!(!header.contains("MAL_HAS_EXTERN_missing"));
    assert!(!directory.join("generated/program.c").exists());
}

#[test]
fn emit_header_prints_to_stdout_without_output() {
    let directory = NativeFixture::new("driver-default-header");
    let source = directory.join("source/program.mal");
    directory.write(
        "source/program.mal",
        "extern print :: (Address, USize) -> Unit;",
    );

    let output = directory.malc([OsStr::new("emit"), OsStr::new("header"), source.as_os_str()]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = String::from_utf8(output.stdout).unwrap();
    assert!(header.contains("#define MAL_DEFINE_print(call, value)"));
    assert!(!directory.join("source/program.mal.h").exists());
    assert!(header.contains("#include <mal.h>"));
    assert!(!header.contains("FLT_MANT_DIG"));
    for requirement in [
        "FLT_MANT_DIG == 24",
        "DBL_MANT_DIG == 53",
        "FLT_HAS_SUBNORM == 1",
        "FLT_EVAL_METHOD == 0",
    ] {
        assert!(
            mal_backend::pipeline::COMMON_HEADER.contains(requirement),
            "missing assertion for {requirement}"
        );
    }
}

#[test]
fn emit_header_owns_one_file_and_includes_required_file_headers() {
    let directory = NativeFixture::new("driver-file-header");
    let root = directory.write(
        "program.mal",
        "require \"./dependency.mal\";\n\
         Root :: (UInt32, UInt64);\n\
         extern rootOperation :: Dep -> Root;",
    );
    let dependency = directory.write(
        "dependency.mal",
        "Dep :: (UInt8, UInt16);\n\
         extern dependencyOperation :: Dep -> Unit;",
    );
    let root_header = directory.join("program.mal.h");
    let dependency_header = directory.join("dependency.mal.h");

    for (source, header) in [(&root, &root_header), (&dependency, &dependency_header)] {
        let output = directory.malc([
            OsStr::new("emit"),
            OsStr::new("header"),
            source.as_os_str(),
            OsStr::new("--output"),
            header.as_os_str(),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_interface = std::fs::read_to_string(&root_header).unwrap();
    assert!(root_interface.contains("#include \"dependency.mal.h\""));
    assert!(root_interface.contains("MAL_DEFINE_rootOperation"));
    assert!(!root_interface.contains("MAL_DEFINE_dependencyOperation"));
    let dependency_interface = std::fs::read_to_string(&dependency_header).unwrap();
    assert!(dependency_interface.contains("MAL_DEFINE_dependencyOperation"));
    assert!(!dependency_interface.contains("MAL_DEFINE_rootOperation"));
    let host = directory.malc([
        OsStr::new("emit"),
        OsStr::new("host"),
        dependency.as_os_str(),
    ]);
    assert!(host.status.success());
    assert!(host.stdout.starts_with(b"#include \"dependency.mal.h\"\n"));

    directory.write("mal.h", mal_backend::pipeline::COMMON_HEADER);
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         MAL_DEFINE_dependencyOperation(call, value) {\n\
             (void)value;\n\
             return mal_Unit_return(call);\n\
         }\n\
         MAL_DEFINE_rootOperation(call, value) {\n\
             (void)value;\n\
             return mal_Root_return(call, (mal_Root_t){0});\n\
         }\n",
    );
    let compilation = std::process::Command::new("clang")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic", "-c"])
        .arg("-I")
        .arg(directory.join(""))
        .arg(directory.join("host.c"))
        .arg("-o")
        .arg(directory.join("host.o"))
        .output()
        .expect("compile composed file headers");
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
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
        OsStr::new("emit"),
        OsStr::new("header"),
        source.as_os_str(),
        OsStr::new("--output"),
        header.as_os_str(),
    ]);
    assert!(header_output.status.success());
    let default_output =
        directory.malc([OsStr::new("emit"), OsStr::new("host"), source.as_os_str()]);
    assert!(default_output.status.success());
    assert!(
        default_output
            .stdout
            .starts_with(b"#include \"program.mal.h\"\n")
    );

    let output = directory.malc([
        OsStr::new("emit"),
        OsStr::new("host"),
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
    directory.write("mal.h", mal_backend::pipeline::COMMON_HEADER);
    let object = directory.join("host.o");
    let compilation = std::process::Command::new("clang")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic", "-c"])
        .arg("-I")
        .arg(directory.join(""))
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
    fn collect_files(
        directory: &Path,
        predicate: fn(&Path) -> bool,
        files: &mut Vec<std::path::PathBuf>,
    ) {
        for entry in std::fs::read_dir(directory).expect("read example directory") {
            let path = entry.expect("read example entry").path();
            if path.is_dir() {
                collect_files(&path, predicate, files);
            } else if predicate(&path) {
                files.push(path);
            }
        }
    }

    fn collect_header_closure(
        header: std::path::PathBuf,
        headers: &mut std::collections::BTreeSet<std::path::PathBuf>,
    ) {
        let header = std::fs::canonicalize(header).expect("generated dependency header exists");
        if !headers.insert(header.clone()) {
            return;
        }
        let contents = std::fs::read_to_string(&header).expect("read generated example header");
        for line in contents.lines() {
            let Some(include) = line
                .strip_prefix("#include \"")
                .and_then(|line| line.strip_suffix('"'))
                .filter(|include| include.ends_with(".mal.h"))
            else {
                continue;
            };
            collect_header_closure(
                header
                    .parent()
                    .expect("header has a directory")
                    .join(include),
                headers,
            );
        }
    }

    let fixture = NativeFixture::new("example-headers");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler directory has a repository parent");
    let examples = repository.join("examples");
    let mut sources = Vec::new();
    collect_files(
        &examples,
        |path| path.extension() == Some(OsStr::new("mal")),
        &mut sources,
    );
    let mut required_headers = std::collections::BTreeSet::new();
    for source in sources {
        let contents = std::fs::read_to_string(&source).expect("read example source");
        let owns_c_surface = contents.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("extern ") || (line.starts_with("require \"") && line.contains(".c\""))
        });
        if owns_c_surface {
            collect_header_closure(source.with_extension("mal.h"), &mut required_headers);
        }
    }

    let mut checked_in_headers = Vec::new();
    collect_files(
        &examples,
        |path| path.to_string_lossy().ends_with(".mal.h"),
        &mut checked_in_headers,
    );
    let checked_in_headers = checked_in_headers
        .into_iter()
        .map(|path| std::fs::canonicalize(path).expect("checked-in header exists"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(checked_in_headers, required_headers);

    for (index, checked_in) in required_headers.into_iter().enumerate() {
        let source = checked_in.with_extension("");
        let relative = source
            .strip_prefix(&examples)
            .expect("example source has an examples-relative path");
        let generated = fixture.join(format!(
            "{}.h",
            relative.to_string_lossy().replace('/', "-")
        ));
        let output = fixture.malc([
            OsStr::new("emit"),
            OsStr::new("header"),
            source.as_os_str(),
            OsStr::new("--output"),
            generated.as_os_str(),
        ]);
        assert!(
            output.status.success(),
            "failed to generate {}: {}",
            relative.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(generated).unwrap(),
            std::fs::read_to_string(&checked_in).unwrap(),
            "checked-in header is stale for {}",
            relative.display()
        );
        let compilation = std::process::Command::new("clang")
            .args([
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic",
                "-Wno-unused-function",
                "-c",
                "-xc",
            ])
            .arg("-I")
            .arg(repository.join("crates/mal-backend/include"))
            .arg(&checked_in)
            .arg("-o")
            .arg(fixture.join(format!("header-{index}.o")))
            .output()
            .expect("compile checked-in header closure");
        assert!(
            compilation.status.success(),
            "{}: {}",
            relative.display(),
            String::from_utf8_lossy(&compilation.stderr)
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
fn build_umbrella_exposes_required_file_memory_helpers_to_host_inputs() {
    let directory = NativeFixture::new("driver-required-file-header");
    let source = directory.write(
        "program.mal",
        "require \"./host.mal\";\nmain :: Unit -> Int32 := () -> 0;",
    );
    directory.write(
        "host.mal",
        "require \"./types.mal\";\n\
         require \"./host.c\";\n\
         extern readFirst :: Address -> Int32;",
    );
    directory.write("types.mal", "Record :: (Int32, UInt8);");
    directory.write(
        "host.mal.h",
        "#ifndef MAL_BUILD_UMBRELLA\n#error build must preinclude its current umbrella\n#endif\n",
    );
    directory.write(
        "host.c",
        "#include \"host.mal.h\"\n\
         MAL_DEFINE_readFirst(call, address) {\n\
             mal_Record_t value = mal_Record_read(call, address, 0);\n\
             return mal_Int32_return(call, value.field_0);\n\
         }\n",
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

#[test]
fn copies_between_c_host_storage_and_a_shared_managed_buffer() {
    let directory = NativeFixture::new("driver-c-host-buffer-copy");
    let source = directory.join("program.mal");
    let executable = directory.join("program");
    directory.write(
        "program.mal",
        "require \"./host.c\";
         extern storage :: Unit -> Address;
         extern verify :: Unit -> Int32;
         main :: Unit -> Int32 := () -> {
           values := from<UInt32>(storage(), 1usize, 2usize);
           alias := values;
           alias.put(0usize, 41u32);
           values.new(99u32);
           values.into(storage(), 0usize, 2usize);
           if (values.get(0usize) == 41u32 && alias.get(2usize) == 99u32)
           then verify()
           else 1i32;
         };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         #include <stdint.h>\n\
         static uint32_t values[] = {10, 20, 30};\n\
         MAL_DEFINE_storage(call) { return mal_Address_return(call, values); }\n\
         MAL_DEFINE_verify(call) {\n\
             return mal_Int32_return(call, values[0] == 41 && values[1] == 30 ? 0 : 2);\n\
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
