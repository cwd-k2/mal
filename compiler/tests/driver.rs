use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("mal-driver-test-{}-{sequence}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }

    fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test directory");
    }
}

fn malc(arguments: impl IntoIterator<Item = impl AsRef<OsStr>>) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_malc"))
        .args(arguments)
        .output()
        .expect("run malc")
}

#[test]
fn check_reports_frontend_success_and_failure_through_exit_status() {
    let directory = TestDirectory::new();
    let valid = directory.join("valid.mal");
    fs::write(&valid, "value :: Int32 := 1;").expect("write valid source");
    let output = malc([OsStr::new("check"), valid.as_os_str()]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    let invalid = directory.join("invalid.mal");
    fs::write(&invalid, "value :: Unit := 1;").expect("write invalid source");
    let output = malc([OsStr::new("check"), invalid.as_os_str()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("type mismatch"));
}

#[test]
fn emit_c_writes_the_translation_unit_and_paired_header() {
    let directory = TestDirectory::new();
    let source = directory.join("program.mal");
    let output_path = directory.join("generated/program.c");
    fs::write(&source, "main :: Unit -> Int32 := \\() { return 0; };").expect("write source");
    let output = malc([
        OsStr::new("emit-c"),
        source.as_os_str(),
        OsStr::new("--output"),
        output_path.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(output_path)
            .unwrap()
            .contains("int main(void)")
    );
    assert!(
        fs::read_to_string(directory.join("generated/program.mal.h"))
            .unwrap()
            .contains("MAL_C_ABI_VERSION")
    );
}

#[test]
fn build_links_multiple_host_inputs_and_produces_an_executable() {
    let directory = TestDirectory::new();
    let source = directory.join("program.mal");
    let host = directory.join("host.c");
    let helper = directory.join("helper.c");
    let executable = directory.join("out/program");
    fs::write(
        &source,
        "extern adjust :: Int32 -> Int32;\n\
         main :: Unit -> Int32 := \\() { return extern adjust(40) - 42; };",
    )
    .expect("write source");
    fs::write(
        &host,
        "#include \"program.mal.h\"\n\
         int32_t host_increment(int32_t value);\n\
         int32_t mal_ext_adjust(MalContext *context, int32_t value) {\n\
             (void)context;\n\
             return host_increment(value);\n\
         }\n",
    )
    .expect("write host");
    fs::write(
        &helper,
        "#include <stdint.h>\n\
         int32_t host_increment(int32_t value) { return value + 2; }\n",
    )
    .expect("write helper");

    let output = malc([
        OsStr::new("build"),
        source.as_os_str(),
        OsStr::new("--output"),
        executable.as_os_str(),
        OsStr::new("--link"),
        host.as_os_str(),
        OsStr::new("--link"),
        helper.as_os_str(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        Command::new(executable)
            .status()
            .expect("run executable")
            .success()
    );
}
