use super::*;

#[test]
fn print_and_closure_example_builds_and_runs_through_the_public_cli() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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
fn integer_and_byte_example_reproduces_host_results() {
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
        .parent()
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
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/symbol-round-trip");
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
fn socket_packet_example_transfers_a_managed_packet_through_the_host() {
    let directory = NativeFixture::new("socket-packet");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn recoverable_file_example_copies_bytes_and_reports_open_errors() {
    let directory = NativeFixture::new("recoverable-file");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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
fn resizable_buffer_example_handles_growth_slices_and_stale_aliases() {
    let directory = NativeFixture::new("resizable-buffer");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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
        "al\nmal-shared-buffer\nresize rejected\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn mini_database_example_persists_queries_across_processes() {
    let directory = NativeFixture::new("mini-database");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/mini-database");
    let executable = directory.join("example");
    let database = directory.join("data.bin");
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

    let run_session = |input: &str| {
        let mut child = Command::new(&executable)
            .arg(&database)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run mini database");
        child
            .stdin
            .take()
            .expect("open mini database stdin")
            .write_all(input.as_bytes())
            .expect("write mini database queries");
        child.wait_with_output().expect("wait for mini database")
    };

    let output = run_session("put language mal\nput greeting hello world\nget language\nquit\n");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "OK\nOK\nVALUE mal\n"
    );
    assert_eq!(std::fs::metadata(&database).unwrap().len(), 4112);

    let output = run_session("get greeting\ndel language\nget language\nquit\n");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "VALUE hello world\nOK\nNOT FOUND\n"
    );

    let output = run_session("get greeting\r\nquit\r\n");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "VALUE hello world\n"
    );

    let longest_line = format!("{}\nquit\n", "x".repeat(256));
    let output = run_session(&longest_line);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "ERROR commands: put/get/del/quit\n"
    );

    let chunked_input = format!("{}quit\n", "get missing\n".repeat(400));
    let output = run_session(&chunked_input);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "NOT FOUND\n".repeat(400)
    );

    let overlong_line = format!("{}\n", "x".repeat(257));
    let output = run_session(&overlong_line);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("query line exceeds its buffer"));

    std::fs::write(&database, vec![0; 4113]).unwrap();
    let output = run_session("quit\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid database file"));
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
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}

#[test]
fn fallible_tree_example_cleans_partial_construction() {
    let directory = NativeFixture::new("fallible-tree");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/fallible-tree");
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
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
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
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.run(executable).status.success());
}
