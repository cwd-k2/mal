#![allow(dead_code)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(crate) struct NativeFixture {
    directory: PathBuf,
}

impl NativeFixture {
    pub(crate) fn new(suite: &str) -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "mal-{suite}-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("create native fixture directory");
        Self { directory }
    }

    pub(crate) fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.directory.join(path)
    }

    pub(crate) fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture output directory");
        }
        fs::write(&path, contents).expect("write fixture file");
        path
    }

    pub(crate) fn malc(&self, arguments: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Output {
        Command::new(env!("CARGO_BIN_EXE_malc"))
            .args(arguments)
            .output()
            .expect("run malc")
    }

    pub(crate) fn malc_with_env(
        &self,
        arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
        key: impl AsRef<OsStr>,
        value: impl AsRef<OsStr>,
    ) -> Output {
        Command::new(env!("CARGO_BIN_EXE_malc"))
            .args(arguments)
            .env(key, value)
            .output()
            .expect("run malc")
    }

    pub(crate) fn run(&self, executable: impl AsRef<Path>) -> Output {
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
