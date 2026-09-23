use super::*;

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

    let deeply_nested = format!("{}0{}", "[".repeat(64), "]".repeat(64));
    let output = run_query("depth", &deeply_nested);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{\"ok\":true,\"depth\":65}\n");

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
fn buffer_tree_example_builds_mutates_and_traverses_a_tree() {
    let directory = NativeFixture::new("driver");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/buffer-tree");
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
fn relation_modeling_examples_build_and_validate_their_results() {
    for name in ["relation-views", "csr-dijkstra", "spreadsheet"] {
        let directory = NativeFixture::new(name);
        let example = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("compiler has a repository parent")
            .join("examples")
            .join(name);
        let executable = directory.join("example");
        let output = directory.malc([
            OsStr::new("build"),
            example.join("program.mal").as_os_str(),
            OsStr::new("--output"),
            executable.as_os_str(),
        ]);
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = directory.run(executable);
        assert!(output.status.success(), "{name}: {}", output.status);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
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
