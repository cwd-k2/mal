//! `mal-fmt` argument grammar and process-independent command outcome.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::file;

/// Canonical help text printed by `--help`.
pub const HELP: &str = "mal-fmt — formatter for mal v0.7

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
/// Stable outcome classes used by the formatter binary.
pub enum ExitStatus {
    /// Formatting or the requested write completed successfully.
    Success,
    /// Source admission or filesystem access failed.
    FormatError,
    /// The arguments do not name a supported invocation.
    UsageError,
}

impl ExitStatus {
    /// Returns the stable process status used by the binary entry point.
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::FormatError => 1,
            Self::UsageError => 2,
        }
    }
}

/// Process-independent CLI output; `main` is the only code that writes it to stdio.
#[derive(Debug, Eq, PartialEq)]
pub struct Outcome {
    /// The process status class.
    pub status: ExitStatus,
    /// Bytes intended for standard output.
    pub stdout: String,
    /// Bytes intended for standard error.
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

/// Returns the formatter and language version line used by `--version`.
pub fn version_line() -> String {
    format!(
        "mal-fmt {} (language v{})",
        env!("CARGO_PKG_VERSION"),
        mal_syntax::LANGUAGE_VERSION
    )
}

/// Parses arguments and formats one file without reading process-global arguments or writing stdio.
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
