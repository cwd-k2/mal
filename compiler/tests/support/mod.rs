#![allow(dead_code)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use malc::c_emit;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub struct NativeFixture {
    directory: PathBuf,
}

impl NativeFixture {
    pub fn new(suite: &str) -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "mal-{suite}-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("create native fixture directory");
        Self { directory }
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.directory.join(path)
    }

    pub fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture output directory");
        }
        fs::write(&path, contents).expect("write fixture file");
        path
    }

    pub fn compile_generated(&self, generated: c_emit::Output, host: &str) -> PathBuf {
        self.write(c_emit::GENERATED_HEADER_NAME, generated.header);
        self.write("program.c", generated.source);
        let executable = self.join("program");
        let mut compiler = Command::new("clang");
        compiler.current_dir(&self.directory).args([
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pedantic",
            "program.c",
        ]);
        if !host.is_empty() {
            self.write("host.c", host);
            compiler.arg("host.c");
        }
        let compilation = compiler
            .arg("-o")
            .arg(&executable)
            .output()
            .expect("run Clang");
        assert!(
            compilation.status.success(),
            "Clang failed:\n{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        executable
    }

    pub fn malc(&self, arguments: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Output {
        Command::new(env!("CARGO_BIN_EXE_malc"))
            .args(arguments)
            .output()
            .expect("run malc")
    }

    pub fn run(&self, executable: impl AsRef<Path>) -> Output {
        Command::new(executable.as_ref())
            .output()
            .expect("run native fixture executable")
    }
}

impl Drop for NativeFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).expect("remove native fixture directory");
    }
}
