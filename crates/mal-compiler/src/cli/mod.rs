//! `malc` argument grammar, use-case selection, and process-independent command outcome.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::driver::{OptimizationMode, ToolchainOptions};
use crate::version_line;

mod options;
#[cfg(test)]
mod tests;

use options::{Flag, TOOLCHAIN_FLAGS, parse};

/// Canonical help text printed by an empty invocation and `--help`.
pub const HELP: &str = "malc — compiler for mal v0.7

Usage:
  malc <command> [options]
  malc --help
  malc --version

Commands:
  check <source.mal>            Check a program without producing artifacts
  build <source.mal>            Build an executable (requires -o)
  emit header <source.mal>      Write the C host header
  emit host <source.mal>        Write a C host implementation template
  emit atcoder <source.mal>     Write one C++ source for an AtCoder submission

Options:
  -o, --output <path>                        Output path; emit commands print to stdout without it
  --optimization <baseline|production>       Optimization mode [default: production] (build, emit atcoder)
  --artifact-dir <directory>                 Keep generated build artifacts here (build, emit atcoder)
  --clang-arg <argument>                     Add a Clang argument; may be repeated (build, emit atcoder)
  --header <header-name>                     Include this generated header name (emit host)

Global options:
  -h, --help     Print help
  -V, --version  Print version
";

/// Stable outcome classes used by the compiler binary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitStatus {
    /// The requested use case completed successfully.
    Success = 0,
    /// Source admission, artifact generation, filesystem access, or the toolchain failed.
    CompileError = 1,
    /// The arguments do not name a supported invocation.
    UsageError = 2,
}

impl ExitStatus {
    /// Returns the stable process status used by the binary entry point.
    pub const fn code(self) -> u8 {
        self as u8
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

    fn usage_error(stderr: impl Into<String>) -> Self {
        Self {
            status: ExitStatus::UsageError,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    fn compile_error(error: crate::driver::Error) -> Self {
        let mut stderr = error.to_string();
        if !stderr.ends_with('\n') {
            stderr.push('\n');
        }
        Self {
            status: ExitStatus::CompileError,
            stdout: String::new(),
            stderr,
        }
    }
}

/// Parses arguments and runs one compiler use case without reading process-global arguments or writing stdio.
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
        [command, rest @ ..] if command == OsStr::new("check") => execute_check(rest),
        [command, rest @ ..] if command == OsStr::new("build") => execute_build(rest),
        [command, subcommand, rest @ ..] if command == OsStr::new("emit") => {
            match subcommand.to_str() {
                Some("header") => execute_emit_header(rest),
                Some("host") => execute_emit_host(rest),
                Some("atcoder") => execute_emit_atcoder(rest),
                _ => usage_error("emit requires 'header', 'host', or 'atcoder'"),
            }
        }
        _ => usage_error("unknown command or invalid arguments"),
    }
}

fn execute_check(arguments: &[OsString]) -> Outcome {
    let parsed = match parse("check", arguments, &[]) {
        Ok(parsed) => parsed,
        Err(outcome) => return outcome,
    };
    match crate::driver::check(&parsed.source) {
        Ok(()) => Outcome::success(String::new()),
        Err(error) => Outcome::compile_error(error),
    }
}

fn execute_build(arguments: &[OsString]) -> Outcome {
    let parsed = match parse("build", arguments, TOOLCHAIN_FLAGS) {
        Ok(parsed) => parsed,
        Err(outcome) => return outcome,
    };
    let Some(output) = &parsed.output else {
        return usage_error("build requires --output");
    };
    match crate::driver::build(&parsed.source, output, &parsed.toolchain()) {
        Ok(()) => Outcome::success(String::new()),
        Err(error) => Outcome::compile_error(error),
    }
}

fn execute_emit_header(arguments: &[OsString]) -> Outcome {
    let parsed = match parse("emit header", arguments, &[Flag::Output]) {
        Ok(parsed) => parsed,
        Err(outcome) => return outcome,
    };
    deliver(
        crate::driver::emit_header(&parsed.source),
        parsed.output,
        "write generated header",
    )
}

fn execute_emit_host(arguments: &[OsString]) -> Outcome {
    let parsed = match parse("emit host", arguments, &[Flag::Output, Flag::Header]) {
        Ok(parsed) => parsed,
        Err(outcome) => return outcome,
    };
    let header = match parsed.header {
        Some(header) => header,
        None => {
            let Some(name) = parsed.source.file_name().and_then(OsStr::to_str) else {
                return usage_error("emit host source file name must be valid UTF-8");
            };
            format!("{name}.h")
        }
    };
    deliver(
        crate::driver::emit_host(&parsed.source, &header),
        parsed.output,
        "write generated host template",
    )
}

fn execute_emit_atcoder(arguments: &[OsString]) -> Outcome {
    let parsed = match parse("emit atcoder", arguments, TOOLCHAIN_FLAGS) {
        Ok(parsed) => parsed,
        Err(outcome) => return outcome,
    };
    deliver(
        crate::driver::emit_atcoder(&parsed.source, &parsed.toolchain()),
        parsed.output,
        "write AtCoder submission",
    )
}

/// Prints generated text to stdout, or writes it to `output` when one was given.
fn deliver(
    generated: Result<String, crate::driver::Error>,
    output: Option<PathBuf>,
    action: &str,
) -> Outcome {
    match (generated, output) {
        (Err(error), _) => Outcome::compile_error(error),
        (Ok(text), None) => Outcome::success(text),
        (Ok(text), Some(path)) => match crate::driver::write_output(&path, action, &text) {
            Ok(()) => Outcome::success(String::new()),
            Err(error) => Outcome::compile_error(error),
        },
    }
}

fn usage(message: String) -> Outcome {
    usage_error(&message)
}

fn usage_error(message: &str) -> Outcome {
    Outcome::usage_error(format!("malc: {message}\nTry 'malc --help' for usage.\n"))
}
