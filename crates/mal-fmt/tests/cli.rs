use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("mal-fmt-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture directory");
        Self(path)
    }

    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, text).expect("write fixture file");
        path
    }

    fn run(&self, arguments: &[&OsStr]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mal-fmt"))
            .args(arguments)
            .output()
            .expect("run mal-fmt")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ORIGINAL: &str = "value::Int32:=40+2;// answer\n";
const FORMATTED: &str = "value :: Int32 := 40 + 2; // answer\n";

#[test]
fn prints_canonical_source_without_touching_the_file() {
    let fixture = Fixture::new("print");
    let source = fixture.write("program.mal", ORIGINAL);

    let output = fixture.run(&[source.as_os_str()]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), FORMATTED);
    assert_eq!(fs::read_to_string(source).unwrap(), ORIGINAL);
}

#[test]
fn write_replaces_the_file_and_prints_nothing() {
    let fixture = Fixture::new("write");
    let source = fixture.write("program.mal", ORIGINAL);

    let output = fixture.run(&[OsStr::new("--write"), source.as_os_str()]);

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read_to_string(source).unwrap(), FORMATTED);
}

#[test]
fn syntax_errors_fail_and_keep_the_original() {
    let fixture = Fixture::new("invalid");
    let source = fixture.write("invalid.mal", "value := ;\n");

    let output = fixture.run(&[OsStr::new("-w"), source.as_os_str()]);

    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.is_empty());
    assert_eq!(fs::read_to_string(source).unwrap(), "value := ;\n");
}

#[test]
fn unknown_options_are_usage_errors() {
    let fixture = Fixture::new("usage");

    let output = fixture.run(&[OsStr::new("--bogus"), OsStr::new("x.mal")]);

    assert_eq!(output.status.code(), Some(2));
}
