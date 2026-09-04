use std::ffi::OsStr;
use std::path::Path;

mod support;

use support::NativeFixture;

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
fn emit_c_writes_the_translation_unit_and_paired_header() {
    let directory = NativeFixture::new("driver");
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/program.c");
    directory.write(
        "program.mal",
        "main :: Unit -> Int32 := \\() { return 0; };",
    );
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
    assert!(
        std::fs::read_to_string(output_path)
            .unwrap()
            .contains("int main(void)")
    );
    assert!(
        std::fs::read_to_string(directory.join("generated/program.mal.h"))
            .unwrap()
            .contains("MAL_C_ABI_VERSION")
    );
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
         main :: Unit -> Int32 := \\() { return extern adjust(40) - 42; };",
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
fn checked_in_m0_example_builds_and_runs_through_the_public_cli() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/m0/print-and-closure");
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
fn checked_in_m1_example_reproduces_host_results_and_a_trap() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/m1/integer-and-byte");

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
fn reports_source_and_output_filesystem_failures() {
    let directory = NativeFixture::new("driver-failure");
    let missing = directory.join("missing.mal");
    let output = directory.malc([OsStr::new("check"), missing.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: cannot read"));
    assert!(stderr.contains(missing.to_string_lossy().as_ref()));

    let source = directory.write(
        "program.mal",
        "main :: Unit -> Int32 := \\() { return 0; };",
    );
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
    let source = directory.write(
        "program.mal",
        "main :: Unit -> Int32 := \\() { return 0; };",
    );
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
