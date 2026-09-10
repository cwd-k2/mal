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

    let source = directory.write("program.mal", "main :: Unit -> Int32 := () { 0; };");
    directory.write("blocked", "not a directory");
    let generated = directory.join("blocked/program.mal.h");
    let output = directory.malc([
        OsStr::new("emit-header"),
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
fn reports_missing_and_cyclic_requirements_at_the_declaration() {
    let directory = NativeFixture::new("driver-requirement-failure");
    let missing = directory.write("missing-root.mal", "require \"./absent.mal\";\n");
    let output = directory.malc([OsStr::new("check"), missing.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: cannot load requirement"));
    assert!(stderr.contains("missing-root.mal:1:9"));

    let first = directory.write("first.mal", "require \"./second.mal\";\n");
    directory.write("second.mal", "require \"./first.mal\";\n");
    let output = directory.malc([OsStr::new("check"), first.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: cyclic `.mal` requirement"));
    assert!(stderr.contains("second.mal:1:9"));
}

#[test]
fn rejects_empty_and_unsupported_requirement_paths() {
    let directory = NativeFixture::new("driver-invalid-requirement");
    let empty = directory.write("empty.mal", "require \"\";\n");
    let output = directory.malc([OsStr::new("check"), empty.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected a non-empty relative path"));

    let unsupported = directory.write("unsupported.mal", "require \"./data.txt\";\n");
    let output = directory.malc([OsStr::new("check"), unsupported.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("error: unsupported requirement type")
    );
}

#[test]
fn renders_frontend_errors_from_the_required_file() {
    let directory = NativeFixture::new("driver-required-diagnostic");
    let root = directory.write("root.mal", "require \"./broken.mal\";\n");
    directory.write("broken.mal", "value :: Int32 := true;\n");

    let output = directory.malc([OsStr::new("check"), root.as_os_str()]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("broken.mal:1:"));
    assert!(!stderr.contains("<invalid span>"));
}

#[test]
fn reports_required_c_source_and_c_compiler_failures() {
    let directory = NativeFixture::new("driver-failure");
    let source = directory.write(
        "program.mal",
        "require \"./missing-host.c\";\nmain :: Unit -> Int32 := () { 0; };",
    );
    let executable = directory.join("program");
    let arguments = [
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ];
    let output = directory.malc(arguments);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: cannot load requirement"));
    assert!(stderr.contains("missing-host.c"));

    directory.write(
        "program.mal",
        "require \"./broken.c\";\nmain :: Unit -> Int32 := () { 0; };",
    );
    directory.write("broken.c", "this is not C\n");
    let artifacts = directory.join("failed-artifacts");
    let output = directory.malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--artifact-dir"),
        artifacts.as_os_str(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malc: C compiler '"));
    assert!(stderr.contains("failed with exit status"));
    assert!(artifacts.join("program.ll").is_file());
    assert!(artifacts.join("program-shim.c").is_file());
    assert!(artifacts.join("program.mal.h").is_file());
}
