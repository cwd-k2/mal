use super::*;

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

    std::fs::write(&database, vec![0]).unwrap();
    let output = run_session("quit\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid database file"));
}

#[test]
fn mini_database_output_chunks_symbols_larger_than_the_host_buffer() {
    let directory = NativeFixture::new("mini-database-output");
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples/mini-database");
    for name in ["host.mal", "host.c", "program.mal.h"] {
        directory.write(
            name,
            std::fs::read(example.join(name)).expect("read mini database host fixture"),
        );
    }

    let response = "response".repeat(100);
    let error = "error".repeat(140);
    let program = directory.write(
        "program.mal",
        format!(
            "require \"./host.mal\";\n\nmain :: Unit -> Int32 := () -> {{\n    writeSymbol(\"{response}\");\n    0;\n}};\n"
        ),
    );
    let executable = directory.join("response-example");
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
    assert_eq!(output.stdout, response.as_bytes());
    assert!(output.stderr.is_empty());

    directory.write(
        "program.mal",
        format!(
            "require \"./host.mal\";\n\nmain :: Unit -> Int32 := () -> {{\n    fail(\"{error}\");\n    1;\n}};\n"
        ),
    );
    let executable = directory.join("error-example");
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
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.starts_with(format!("{error}\n").as_bytes()));
    assert!(String::from_utf8_lossy(&output.stderr).contains("host rejected the database"));
}
