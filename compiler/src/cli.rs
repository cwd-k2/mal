use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::version_line;

pub const HELP: &str = "malc — reference compiler for mal v0.5

Usage:
  malc --help
  malc --version
  malc check <source.mal>
  malc format <source.mal>
  malc emit-header <source.mal> [--output <program.mal.h>]
  malc emit-host <source.mal>
  malc emit-c <source.mal> --output <program.c>
  malc build <source.mal> --output <program> [--link <input>]...
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
        [command, source] if command == OsStr::new("check") => {
            match crate::driver::check(PathBuf::from(source).as_path()) {
                Ok(()) => Outcome::success(String::new()),
                Err(error) => Outcome::compile_error(error),
            }
        }
        [command, source] if command == OsStr::new("format") => {
            match crate::driver::format(PathBuf::from(source).as_path()) {
                Ok(formatted) => Outcome::success(formatted),
                Err(error) => Outcome::compile_error(error),
            }
        }
        [command, rest @ ..] if command == OsStr::new("emit-header") => execute_emit_header(rest),
        [command, source] if command == OsStr::new("emit-host") => {
            match crate::driver::emit_host(PathBuf::from(source).as_path()) {
                Ok(host) => Outcome::success(host),
                Err(error) => Outcome::compile_error(error),
            }
        }
        [command, rest @ ..] if command == OsStr::new("emit-c") => execute_emit_c(rest),
        [command, rest @ ..] if command == OsStr::new("build") => execute_build(rest),
        _ => usage_error("unknown command or invalid arguments"),
    }
}

fn execute_emit_header(arguments: &[OsString]) -> Outcome {
    let (source, output) = match arguments {
        [source] => {
            let output = PathBuf::from(source).with_file_name(crate::c_emit::GENERATED_HEADER_NAME);
            (source, output)
        }
        [source, option, output] if option == OsStr::new("--output") => {
            (source, PathBuf::from(output))
        }
        [_, option, _] => {
            return usage_error(&format!(
                "unknown emit-header option '{}'",
                option.to_string_lossy()
            ));
        }
        _ => return usage_error("emit-header requires a source path"),
    };
    match crate::driver::emit_header(PathBuf::from(source).as_path(), &output) {
        Ok(()) => Outcome::success(String::new()),
        Err(error) => Outcome::compile_error(error),
    }
}

fn execute_emit_c(arguments: &[OsString]) -> Outcome {
    let [source, option, output] = arguments else {
        return usage_error("emit-c requires a source and --output path");
    };
    if option != OsStr::new("--output") {
        return usage_error("emit-c requires --output after the source path");
    }
    match crate::driver::emit_c(
        PathBuf::from(source).as_path(),
        PathBuf::from(output).as_path(),
    ) {
        Ok(_) => Outcome::success(String::new()),
        Err(error) => Outcome::compile_error(error),
    }
}

fn execute_build(arguments: &[OsString]) -> Outcome {
    let Some(source) = arguments.first() else {
        return usage_error("build requires a source path");
    };
    let mut output = None;
    let mut linker_inputs = Vec::new();
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        let Some(value) = arguments.get(index + 1) else {
            return usage_error(&format!(
                "option '{}' requires a value",
                option.to_string_lossy()
            ));
        };
        if option == OsStr::new("--output") {
            if output.replace(PathBuf::from(value)).is_some() {
                return usage_error("--output may only be specified once");
            }
        } else if option == OsStr::new("--link") {
            linker_inputs.push(PathBuf::from(value));
        } else {
            return usage_error(&format!(
                "unknown build option '{}'",
                option.to_string_lossy()
            ));
        }
        index += 2;
    }
    let Some(output) = output else {
        return usage_error("build requires --output");
    };
    match crate::driver::build(PathBuf::from(source).as_path(), &output, &linker_inputs) {
        Ok(()) => Outcome::success(String::new()),
        Err(error) => Outcome::compile_error(error),
    }
}

fn usage_error(message: &str) -> Outcome {
    Outcome::usage_error(format!("malc: {message}\nTry 'malc --help' for usage.\n"))
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
        assert_eq!(outcome.stdout, "malc 0.5.0-dev (language v0.5)\n");
    }

    #[test]
    fn unknown_command_is_a_usage_error() {
        let outcome = execute(args(&["unknown", "sample.mal"]));
        assert_eq!(outcome.status, ExitStatus::UsageError);
        assert!(outcome.stdout.is_empty());
        assert!(outcome.stderr.contains("unknown command"));
    }

    #[test]
    fn validates_build_options_before_running_the_driver() {
        let outcome = execute(args(&["build", "sample.mal", "--link", "host.c"]));
        assert_eq!(outcome.status, ExitStatus::UsageError);
        assert!(outcome.stderr.contains("requires --output"));

        let outcome = execute(args(&[
            "build",
            "sample.mal",
            "--output",
            "one",
            "--output",
            "two",
        ]));
        assert_eq!(outcome.status, ExitStatus::UsageError);
        assert!(outcome.stderr.contains("only be specified once"));
    }
}
