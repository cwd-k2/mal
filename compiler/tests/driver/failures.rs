use super::*;

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
