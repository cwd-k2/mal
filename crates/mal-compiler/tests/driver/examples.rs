use super::*;

#[test]
fn generic_loop_example_runs_in_baseline_and_production_profiles() {
    let directory = NativeFixture::new("generic-loop");
    let program = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/generic-loop/program.mal");

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
fn print_and_closure_example_builds_and_runs_through_the_public_cli() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/print-and-closure");
    let executable = directory.join("example");
    let program = example.join("program.mal");
    let output = directory.malc([
        OsStr::new("build"),
        program.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
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
fn numeric_conversion_example_reproduces_host_results() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/numeric-conversion");

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
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
}

#[test]
fn opaque_aggregate_example_round_trips_through_the_host() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/opaque-aggregate");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
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
fn symbol_round_trip_example_copies_and_concatenates_bytes() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/symbol-round-trip");
    let program = example.join("program.mal");

    let checked = directory.malc([OsStr::new("check"), program.as_os_str()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        program.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = directory.run(executable);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "9 bytes\n");
}

#[test]
fn socket_packet_example_transfers_bytes_through_borrowed_memory() {
    let directory = NativeFixture::new("socket-packet");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/socket-packet");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = directory.run(executable);
    assert!(
        output.status.success(),
        "status: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn recoverable_file_example_copies_bytes_and_reports_open_errors() {
    let directory = NativeFixture::new("recoverable-file");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/recoverable-file");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

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

#[test]
fn resizable_buffer_example_observes_growth_through_an_alias() {
    let directory = NativeFixture::new("resizable-buffer");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("compiler has a repository parent")
        .join("examples/resizable-buffer");
    let executable = directory.join("example");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
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
        "al\nmal-shared-buffer\n"
    );
    assert!(output.stderr.is_empty());
}
