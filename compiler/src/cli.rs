use std::ffi::{OsStr, OsString};

use crate::version_line;

pub const HELP: &str = "malc — reference compiler for mal v0.4

Usage:
  malc --help
  malc --version

Compilation commands will be added milestone-by-milestone; see docs/implementation/m0.md.
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitStatus {
    Success = 0,
    CompileError = 1,
    UsageError = 2,
}

impl ExitStatus {
    pub const fn code(self) -> u8 {
        self as u8
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

    fn usage_error(stderr: impl Into<String>) -> Self {
        Self {
            status: ExitStatus::UsageError,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
}

pub fn execute(arguments: impl IntoIterator<Item = OsString>) -> Outcome {
    let arguments: Vec<_> = arguments.into_iter().collect();
    match arguments.as_slice() {
        [] => Outcome::success(HELP),
        [argument] if argument == OsStr::new("--help") || argument == OsStr::new("-h") => {
            Outcome::success(HELP)
        }
        [argument] if argument == OsStr::new("--version") || argument == OsStr::new("-V") => {
            Outcome::success(format!("{}\n", version_line()))
        }
        _ => Outcome::usage_error(
            "malc: compilation commands are not implemented yet\n\
             Try 'malc --help' for the current interface.\n",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_arguments_prints_help_successfully() {
        let outcome = execute(args(&[]));
        assert_eq!(outcome.status, ExitStatus::Success);
        assert_eq!(outcome.stdout, HELP);
        assert!(outcome.stderr.is_empty());
    }

    #[test]
    fn version_uses_the_version_contract() {
        let outcome = execute(args(&["--version"]));
        assert_eq!(outcome.status, ExitStatus::Success);
        assert_eq!(outcome.stdout, "malc 0.0.0 (language v0.4)\n");
    }

    #[test]
    fn unavailable_compilation_command_is_a_usage_error() {
        let outcome = execute(args(&["check", "sample.mal"]));
        assert_eq!(outcome.status, ExitStatus::UsageError);
        assert!(outcome.stdout.is_empty());
        assert!(outcome.stderr.contains("not implemented"));
    }
}
