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

#[test]
fn focused_examples_build_and_run() {
    for name in [
        "generic-map",
        "monads-and-comonads",
        "extern-runtime",
        "indexed-graph",
    ] {
        let output = run(name);
        assert!(output.status.success(), "{name}: {}", output.status);
        assert!(output.stdout.is_empty(), "{name}: unexpected stdout");
        assert!(output.stderr.is_empty(), "{name}: unexpected stderr");
    }
}

#[test]
fn control_example_runs_in_baseline_and_production_profiles() {
    let directory = NativeFixture::new("control-and-iteration");
    let program = example("control-and-iteration").join("program.mal");
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
fn host_observable_examples_preserve_their_contracts() {
    for (name, expected) in [
        ("language-tour", "42\n42\n15\n42\n-1\n-2147483648\n"),
        ("managed-bytes", "9 bytes\n"),
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
fn resource_error_example_copies_bytes_and_reports_open_errors() {
    let directory = NativeFixture::new("resource-errors");
    let executable = directory.join("example");
    build(&directory, "resource-errors", &executable);

    let contents = (0..9000)
        .map(|value| (value % 251) as u8)
        .collect::<Vec<_>>();
    let input = directory.write("input.bin", &contents);
    let output = Command::new(&executable)
        .arg(input)
        .output()
        .expect("copy resource-error input");
    assert!(output.status.success());
    assert_eq!(output.stdout, contents);
    assert!(output.stderr.is_empty());

    let output = Command::new(&executable)
        .arg(directory.join("missing.bin"))
        .output()
        .expect("report missing resource-error input");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("file error: "));
}
