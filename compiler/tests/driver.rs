use std::ffi::OsStr;
use std::path::Path;

mod support;

use support::NativeFixture;

#[test]
fn public_cli_reports_help_version_and_usage_status() {
    let directory = NativeFixture::new("driver-cli");

    let output = directory.malc(std::iter::empty::<&OsStr>());
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), malc::cli::HELP);
    assert!(output.stderr.is_empty());

    let output = directory.malc([OsStr::new("--version")]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", malc::version_line())
    );
    assert!(output.stderr.is_empty());

    let output = directory.malc([OsStr::new("unknown")]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "malc: unknown command or invalid arguments\nTry 'malc --help' for usage.\n"
    );
}

#[test]
fn check_reports_frontend_success_and_failure_through_exit_status() {
    let directory = NativeFixture::new("driver");
    let valid = directory.join("valid.mal");
    directory.write("valid.mal", "value :: Int32 := 1;");
    let output = directory.malc([OsStr::new("check"), valid.as_os_str()]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    let invalid = directory.join("invalid.mal");
    directory.write("invalid.mal", "value :: Unit := 1;");
    let output = directory.malc([OsStr::new("check"), invalid.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("type mismatch"));
}

#[test]
fn format_prints_canonical_source_without_modifying_the_input() {
    let directory = NativeFixture::new("driver-format");
    let source = directory.join("program.mal");
    let original = "value::Int32:=40+2;// answer\n";
    directory.write("program.mal", original);

    let output = directory.malc([OsStr::new("format"), source.as_os_str()]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "value :: Int32 := 40 + 2; // answer\n"
    );
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[test]
fn emit_c_writes_the_translation_unit_and_paired_header() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/program.c");
    directory.write("program.mal", "main :: Unit -> Int32 := \\() { 0; };");
    let output = directory.malc([
        OsStr::new("emit-c"),
        source.as_os_str(),
        OsStr::new("--output"),
        output_path.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_c = std::fs::read_to_string(output_path).unwrap();
    assert!(generated_c.starts_with("#include \"program.mal.h\"\n"));
    assert!(generated_c.contains("int main(void)"));
    let generated_header =
        std::fs::read_to_string(directory.join("generated/program.mal.h")).unwrap();
    assert!(generated_header.contains("#define MAL_C_ABI_VERSION 0x000500u"));
    assert!(generated_header.contains("_Noreturn void mal_trap("));
    assert!(generated_header.contains("MalEngram mal_engram_copy("));
}

#[test]
fn emit_header_writes_a_standalone_host_interface() {
    let directory = NativeFixture::new("driver-header");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;",
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
    assert!(header.contains("typedef uint64_t MalType_Count;"));
    assert!(header.contains("#define MAL_HAS_EXTERN_increment 1"));
    assert!(header.contains("#define MAL_DEFINE_increment(context, value)"));
    assert!(!header.contains("MAL_HAS_EXTERN_missing"));
    assert!(!directory.join("generated/program.c").exists());
}

#[test]
fn emit_header_defaults_to_the_source_directory() {
    let directory = NativeFixture::new("driver-default-header");
    let source = directory.join("source/program.mal");
    directory.write("source/program.mal", "extern print :: Engram -> Unit;");

    let output = directory.malc([OsStr::new("emit-header"), source.as_os_str()]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header = std::fs::read_to_string(directory.join("source/program.mal.h")).unwrap();
    assert!(header.contains("#define MAL_DEFINE_print(context, value)"));
}

#[test]
fn emit_host_prints_compilable_external_operation_stubs() {
    let directory = NativeFixture::new("driver-host");
    let source = directory.join("program.mal");
    let header = directory.join("custom.h");
    directory.write(
        "program.mal",
        "Count :: UInt64;\n\
         extern increment :: Count -> Count;\n\
         main :: Unit -> Int32 := \\() { Int32(extern increment(41u64) - 42u64); };",
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
    assert!(host.contains("MAL_DEFINE_increment(context, value)"));
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
        "integer-and-byte",
        "opaque-aggregate",
        "pointer-tree",
        "print-and-closure",
        "ptr-memory",
        "strict-float",
        "engram-round-trip",
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
fn build_links_multiple_host_inputs_and_produces_an_executable() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let host = directory.join("host.c");
    let helper = directory.join("helper.c");
    let executable = directory.join("out/program");
    directory.write(
        "program.mal",
        "extern adjust :: Int32 -> Int32;\n\
         main :: Unit -> Int32 := \\() { extern adjust(40) - 42; };",
    );
    directory.write(
        "host.c",
        "#include \"program.mal.h\"\n\
         int32_t host_increment(int32_t value);\n\
         int32_t mal_ext_adjust(MalContext *context, int32_t value) {\n\
             (void)context;\n\
             return host_increment(value);\n\
         }\n",
    );
    directory.write(
        "helper.c",
        "#include <stdint.h>\n\
         int32_t host_increment(int32_t value) { return value + 2; }\n",
    );

    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        host.as_os_str(),
        OsStr::new("--link"),
        helper.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn print_and_closure_example_builds_and_runs_through_the_public_cli() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/print-and-closure");
    let executable = directory.join("example");
    let program = example.join("program.mal");
    let host = example.join("host.c");
    let output = directory.malc([
        OsStr::new("build"),
        program.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        host.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1\n15\n-2147483648\n"
    );
}

#[test]
fn integer_and_byte_example_reproduces_host_results_and_a_trap() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/integer-and-byte");

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "255\n0\n18446744073709551615\n"
    );

    let trap = directory.join("trap");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("trap.mal").as_os_str(),
        OsStr::new("--output"),
        trap.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(trap);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("mal trap: shift count out of range"));
}

#[test]
fn opaque_aggregate_example_round_trips_through_the_host() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/opaque-aggregate");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn engram_round_trip_example_copies_host_bytes() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/engram-round-trip");
    let program = example.join("program.mal");

    let checked = directory.malc([OsStr::new("check"), program.as_os_str()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let emitted = directory.join("generated/program.c");
    let output = directory.malc([
        OsStr::new("emit-c"),
        program.as_os_str(),
        OsStr::new("--output"),
        emitted.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(emitted.is_file());

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        program.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "8 bytes\n");
}

#[test]
fn tail_recursion_example_executes_a_large_direct_tail_call() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/tail-recursion/program.mal");

    let checked = directory.malc([OsStr::new("check"), example.as_os_str()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let emitted = directory.join("generated/program.c");
    let output = directory.malc([
        OsStr::new("emit-c"),
        example.as_os_str(),
        OsStr::new("--output"),
        emitted.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read_to_string(&emitted)
            .expect("read generated C")
            .contains("goto mal_tail_entry;")
    );

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.as_os_str(),
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
fn ptr_memory_example_accesses_unaligned_storage() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/ptr-memory");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn pointer_tree_example_builds_and_traverses_a_tree() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/pointer-tree");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn strict_float_example_preserves_bits_across_the_host_abi() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/strict-float");
    let program = example.join("program.mal");

    let checked = directory.malc([OsStr::new("check"), program.as_os_str()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let emitted = directory.join("generated/program.c");
    let output = directory.malc([
        OsStr::new("emit-c"),
        program.as_os_str(),
        OsStr::new("--output"),
        emitted.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = std::fs::read_to_string(&emitted).expect("read generated C");
    assert!(generated.contains("_Static_assert(FLT_RADIX == 2"));
    assert!(generated.contains("#pragma STDC FP_CONTRACT OFF"));

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        program.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        example.join("host.c").as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn reports_source_and_output_filesystem_failures() {
    let directory = NativeFixture::new("driver-failure");
    let missing = directory.join("missing.mal");
    let output = directory.malc([OsStr::new("check"), missing.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: cannot read"));
    assert!(stderr.contains(missing.to_string_lossy().as_ref()));

    let source = directory.write("program.mal", "main :: Unit -> Int32 := \\() { 0; };");
    directory.write("blocked", "not a directory");
    let generated = directory.join("blocked/program.c");
    let output = directory.malc([
        OsStr::new("emit-c"),
        source.as_os_str(),
        OsStr::new("--output"),
        generated.as_os_str(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: cannot create output directory"));
    assert!(stderr.contains(directory.join("blocked").to_string_lossy().as_ref()));
}

#[test]
fn reports_linker_input_and_c_compiler_failures() {
    let directory = NativeFixture::new("driver-failure");
    let source = directory.write("program.mal", "main :: Unit -> Int32 := \\() { 0; };");
    let executable = directory.join("program");
    let missing_linker_input = directory.join("missing-host.c");
    let arguments = [
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        missing_linker_input.as_os_str(),
    ];
    let output = directory.malc(arguments);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: C compiler '"));
    assert!(stderr.contains("failed with exit status"));
    assert!(stderr.contains(missing_linker_input.to_string_lossy().as_ref()));

    let unavailable_compiler = directory.join("missing-clang");
    let output = directory.malc_with_env(
        [
            OsStr::new("build"),
            source.as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ],
        OsStr::new("CC"),
        unavailable_compiler.as_os_str(),
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: cannot run C compiler"));
    assert!(stderr.contains(unavailable_compiler.to_string_lossy().as_ref()));
}
