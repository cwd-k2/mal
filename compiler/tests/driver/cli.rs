use super::*;

#[test]
fn public_cli_reports_help_version_and_usage_status() {
    let directory = NativeFixture::new("driver-cli");

    let output = directory.malc(std::iter::empty::<&OsStr>());
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert_eq!(help, malc::cli::HELP);
    assert!(output.stderr.is_empty());
    assert!(!help.contains("emit-c"));
    assert!(help.contains("Optimization profile [default: production]"));
    assert!(help.contains("Executable output path (required)"));
    assert!(help.contains("Add a Clang argument; may be repeated"));

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

    let output = directory.malc([OsStr::new("emit-c"), OsStr::new("program.mal")]);
    assert_eq!(output.status.code(), Some(2));
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
