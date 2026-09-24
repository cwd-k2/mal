use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::file;

pub const HELP: &str = "mal-fmt — formatter for mal v0.6

Usage:
  mal-fmt [--write] <source.mal>
  mal-fmt --help
  mal-fmt --version

Options:
  -w, --write    Atomically replace the source with canonical text instead of printing it
  -h, --help     Print help
  -V, --version  Print version
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitStatus {
    Success,
    FormatError,
    UsageError,
}

impl ExitStatus {
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::FormatError => 1,
            Self::UsageError => 2,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Outcome {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    fn success(stdout: impl Into<String>) -> Self {
        Self {
            status: ExitStatus::Success,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn failure(status: ExitStatus, message: impl Into<String>) -> Self {
        let mut stderr = message.into();
        if !stderr.ends_with('\n') {
            stderr.push('\n');
        }
        Self {
            status,
            stdout: String::new(),
            stderr,
        }
    }
}

pub fn version_line() -> String {
    format!(
        "mal-fmt {} (language v{})",
        env!("CARGO_PKG_VERSION"),
        mal_syntax::LANGUAGE_VERSION
    )
}

pub fn execute(arguments: impl IntoIterator<Item = OsString>) -> Outcome {
    let arguments: Vec<_> = arguments.into_iter().collect();
    let is =
        |argument: &OsString, names: &[&str]| names.iter().any(|name| argument == OsStr::new(name));
    match arguments.as_slice() {
        [] => Outcome::success(HELP),
        [argument] if is(argument, &["-h", "--help"]) => Outcome::success(HELP),
        [argument] if is(argument, &["-V", "--version"]) => {
            Outcome::success(format!("{}\n", version_line()))
        }
        [source] if !source.to_string_lossy().starts_with('-') => {
            match file::format_file(&PathBuf::from(source)) {
                Ok(formatted) => Outcome::success(formatted),
                Err(error) => Outcome::failure(ExitStatus::FormatError, error.to_string()),
            }
        }
        [option, source] if is(option, &["-w", "--write"]) => {
            match file::write_file(&PathBuf::from(source)) {
                Ok(()) => Outcome::success(String::new()),
                Err(error) => Outcome::failure(ExitStatus::FormatError, error.to_string()),
            }
        }
        _ => Outcome::failure(
            ExitStatus::UsageError,
            "mal-fmt: expected [--write] <source.mal>; see mal-fmt --help",
        ),
    }
}
