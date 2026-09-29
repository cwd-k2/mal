use std::path::PathBuf;

use super::*;

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples")
        .join(name)
}

fn build(directory: &NativeFixture, name: &str, executable: &Path) {
    let output = directory.malc([
        OsStr::new("build"),
        example(name).join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(name: &str) -> std::process::Output {
    let directory = NativeFixture::new(name);
    let executable = directory.join("example");
    build(&directory, name, &executable);
    directory.run(executable)
}

fn assert_silent_success(name: &str) {
    let output = run(name);
    assert!(output.status.success(), "{name}: {}", output.status);
    assert!(output.stdout.is_empty(), "{name}: unexpected stdout");
    assert!(output.stderr.is_empty(), "{name}: unexpected stderr");
}

#[test]
fn language_examples_build_and_run() {
    for name in ["hash-map", "operation-family", "type-system"] {
        assert_silent_success(name);
    }

    let output = run("buffer-handles");
    assert_eq!(output.status.code(), Some(38));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn generic_loop_example_runs_in_baseline_and_production_profiles() {
    let directory = NativeFixture::new("generic-loop");
    let program = example("generic-loop").join("program.mal");
    for profile in ["baseline", "production"] {
        let executable = directory.join(profile);
        let output = directory.malc([
            OsStr::new("build"),
            program.as_os_str(),
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
        let output = directory.run(executable);
        assert!(output.status.success(), "{profile}: {}", output.status);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn output_examples_reproduce_host_results() {
    for (name, expected) in [
        ("print-and-closure", "1\n15\n-2147483648\n"),
        ("numeric-conversion", "255\n0\n18446744073709551615\n"),
        ("opaque-aggregate", "42\n"),
        ("resizable-buffer", "al\nmal-shared-buffer\n"),
    ] {
        let output = run(name);
        assert!(output.status.success(), "{name}: {}", output.status);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected,
            "{name}"
        );
        assert!(output.stderr.is_empty(), "{name}");
    }
}

#[test]
fn symbol_round_trip_example_copies_and_concatenates_bytes() {
    let directory = NativeFixture::new("symbol-round-trip");
    let program = example("symbol-round-trip").join("program.mal");
    let checked = directory.malc([OsStr::new("check"), program.as_os_str()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executable = directory.join("example");
    build(&directory, "symbol-round-trip", &executable);
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "9 bytes\n");
}

#[test]
fn socket_packet_example_transfers_bytes_through_borrowed_memory() {
    assert_silent_success("socket-packet");
}

#[test]
fn recoverable_file_example_copies_bytes_and_reports_open_errors() {
    let directory = NativeFixture::new("recoverable-file");
    let executable = directory.join("example");
    build(&directory, "recoverable-file", &executable);

    let contents = (0..9000)
        .map(|value| (value % 251) as u8)
        .collect::<Vec<_>>();
    let input = directory.write("input.bin", &contents);
    let output = Command::new(&executable)
        .arg(input)
        .output()
        .expect("copy recoverable file");
    assert!(output.status.success());
    assert_eq!(output.stdout, contents);
    assert!(output.stderr.is_empty());

    let output = Command::new(&executable)
        .arg(directory.join("missing.bin"))
        .output()
        .expect("report missing recoverable file");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("file error: "));
}
