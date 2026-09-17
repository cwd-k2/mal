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
fn numeric_conversion_example_reproduces_host_results() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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
fn resizable_buffer_example_handles_growth_borrows_and_stale_aliases() {
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

    let mut fill_input = String::from("put key-that-is-longer-than-sixteen value\n");
    for index in 0..64 {
        fill_input.push_str(&format!("put key{index} value{index}\n"));
    }
    fill_input.push_str("quit\n");
    let output = run_session(&fill_input);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "ERROR key<=16 value<=40\n{}ERROR database full\n",
            "OK\n".repeat(63)
        )
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
fn json_query_example_parses_stdin_and_selects_an_argument_query() {
    let directory = NativeFixture::new("json-query");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/json-query");
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

    let run_query = |query: &str, input: &str| {
        let mut child = Command::new(&executable)
            .arg(query)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run JSON query example");
        child
            .stdin
            .take()
            .expect("open JSON query stdin")
            .write_all(input.as_bytes())
            .expect("write JSON query input");
        child.wait_with_output().expect("wait for JSON query")
    };

    let document = r#"{"name":"mal","items":[true,null,35]}"#;
    let output = run_query("count", document);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{\"ok\":true,\"count\":6}\n");
    assert!(output.stderr.is_empty());

    let output = run_query("depth", document);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{\"ok\":true,\"depth\":3}\n");
    assert!(output.stderr.is_empty());

    let output = run_query("count", r#"{"escaped":"line\n\u3042","number":-1.25e+3}"#);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{\"ok\":true,\"count\":3}\n");

    let deepest_supported = format!("{}0{}", "[".repeat(15), "]".repeat(15));
    let output = run_query("depth", &deepest_supported);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{\"ok\":true,\"depth\":16}\n");

    let beyond_fixed_stack = format!("{}0{}", "[".repeat(16), "]".repeat(16));
    let output = run_query("depth", &beyond_fixed_stack);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"{\"ok\":false,\"error\":\"JSON nesting exceeds 15 containers\"}\n"
    );

    let output = run_query("count", "[1,]");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"{\"ok\":false,\"error\":\"expected JSON value\"}\n"
    );

    let output = Command::new(&executable)
        .arg("unknown")
        .output()
        .expect("run unknown JSON query");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"{\"ok\":false,\"error\":\"query must be count or depth\"}\n"
    );
}

#[test]
fn brainfuck_compiler_example_emits_executable_llvm_ir() {
    let directory = NativeFixture::new("brainfuck-llvm");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/brainfuck-llvm");
    let compiler = directory.join("brainfuck-llvm");
    let output = directory.malc([
        OsStr::new("build"),
        example.join("program.mal").as_os_str(),
        OsStr::new("--output"),
        compiler.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let source = directory.write("hello.bf", "+[>++++++++[>++++++++<-]<-]>>+.\n");
    let generated = Command::new(&compiler)
        .arg(source)
        .output()
        .expect("compile Brainfuck source");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    assert!(generated.stdout.starts_with(b"declare i32 @getchar()\n"));
    let module = directory.write("program.ll", generated.stdout);
    let executable = directory.join("program");
    let compiled = Command::new("clang")
        .args([module.as_os_str(), OsStr::new("-o"), executable.as_os_str()])
        .output()
        .expect("compile generated LLVM IR");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = directory.run(executable);
    assert!(executed.status.success());
    assert_eq!(executed.stdout, b"A");

    let empty = directory.write("empty.bf", "");
    let empty_generated = Command::new(&compiler)
        .arg(empty)
        .output()
        .expect("compile empty Brainfuck source");
    assert!(empty_generated.status.success());
    assert!(empty_generated.stdout.ends_with(b"  ret i32 0\n}\n"));

    let size_unknown = Command::new(&compiler)
        .arg("/proc/self/cmdline")
        .output()
        .expect("compile a growing Linux pseudo-file snapshot");
    assert!(size_unknown.status.success());
    assert!(
        size_unknown
            .stdout
            .windows(b"call void @bf_decrement()".len())
            .any(|window| window == b"call void @bf_decrement()")
    );

    let unmatched = directory.write("unmatched.bf", "[+");
    let rejected = Command::new(&compiler)
        .arg(unmatched)
        .output()
        .expect("reject unmatched Brainfuck loop");
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    assert_eq!(rejected.stderr, b"unmatched '['\n");

    let missing = Command::new(&compiler)
        .arg(directory.join("missing.bf"))
        .output()
        .expect("report a missing Brainfuck source");
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).starts_with("cannot read source, errno "));

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open the Linux full device");
    let write_failure = Command::new(&compiler)
        .arg(example.join("hello.bf"))
        .stdout(Stdio::from(full))
        .output()
        .expect("report an LLVM output failure");
    assert_eq!(write_failure.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&write_failure.stderr).starts_with("cannot write output, errno ")
    );
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
fn typed_memory_example_accesses_unaligned_storage() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/typed-memory");
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
fn external_tree_example_builds_and_traverses_a_tree() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/external-tree");
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
