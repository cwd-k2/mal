use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::driver::{OptimizationMode, ToolchainOptions};
use crate::version_line;

pub const HELP: &str = "malc — compiler for mal v0.6

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

#[derive(Clone, Copy, Eq, PartialEq)]
enum Flag {
    Output,
    ArtifactDirectory,
    ClangArgument,
    Optimization,
    Header,
}

impl Flag {
    fn parse(argument: &OsStr) -> Option<Self> {
        Some(match argument.to_str()? {
            "-o" | "--output" => Self::Output,
            "--artifact-dir" => Self::ArtifactDirectory,
            "--clang-arg" => Self::ClangArgument,
            "--optimization" => Self::Optimization,
            "--header" => Self::Header,
            _ => return None,
        })
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Output => "--output",
            Self::ArtifactDirectory => "--artifact-dir",
            Self::ClangArgument => "--clang-arg",
            Self::Optimization => "--optimization",
            Self::Header => "--header",
        }
    }
}

const TOOLCHAIN_FLAGS: &[Flag] = &[
    Flag::Output,
    Flag::ArtifactDirectory,
    Flag::ClangArgument,
    Flag::Optimization,
];

#[derive(Default)]
struct Parsed {
    source: PathBuf,
    output: Option<PathBuf>,
    artifact_directory: Option<PathBuf>,
    clang_arguments: Vec<OsString>,
    optimization: Option<OptimizationMode>,
    header: Option<String>,
}

impl Parsed {
    fn toolchain(&self) -> ToolchainOptions<'_> {
        ToolchainOptions {
            artifact_directory: self.artifact_directory.as_deref(),
            clang_arguments: &self.clang_arguments,
            optimization: self.optimization.unwrap_or(OptimizationMode::Production),
        }
    }
}

/// Parses one source path and the options `allowed` for `command`, in any order.
fn parse(command: &str, arguments: &[OsString], allowed: &[Flag]) -> Result<Parsed, Outcome> {
    let mut parsed = Parsed::default();
    let mut source = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        index += 1;
        if !argument.to_string_lossy().starts_with('-') {
            if source.replace(PathBuf::from(argument)).is_some() {
                return Err(usage(format!("{command} takes one source path")));
            }
            continue;
        }
        let Some(flag) = Flag::parse(argument).filter(|flag| allowed.contains(flag)) else {
            return Err(usage(format!(
                "unknown {command} option '{}'",
                argument.to_string_lossy()
            )));
        };
        let Some(value) = arguments.get(index) else {
            return Err(usage(format!(
                "option '{}' requires a value",
                argument.to_string_lossy()
            )));
        };
        index += 1;
        let duplicate = match flag {
            Flag::Output => parsed.output.replace(PathBuf::from(value)).is_some(),
            Flag::ArtifactDirectory => parsed
                .artifact_directory
                .replace(PathBuf::from(value))
                .is_some(),
            Flag::ClangArgument => {
                parsed.clang_arguments.push(value.clone());
                false
            }
            Flag::Optimization => {
                let mode = match value.to_str() {
                    Some("baseline") => OptimizationMode::Baseline,
                    Some("production") => OptimizationMode::Production,
                    _ => {
                        return Err(usage(
                            "--optimization must be 'baseline' or 'production'".to_owned(),
                        ));
                    }
                };
                parsed.optimization.replace(mode).is_some()
            }
            Flag::Header => {
                let Some(name) = value.to_str() else {
                    return Err(usage(format!("{command} header name must be valid UTF-8")));
                };
                if !mal_backend::pipeline::is_valid_header_name(name) {
                    return Err(usage(format!(
                        "{command} header name is not valid in a quoted C include"
                    )));
                }
                parsed.header.replace(name.to_owned()).is_some()
            }
        };
        if duplicate {
            return Err(usage(format!("{} may only be specified once", flag.name())));
        }
    }
    let Some(source) = source else {
        return Err(usage(format!("{command} requires a source path")));
    };
    parsed.source = source;
    Ok(parsed)
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
    let header = parsed
        .header
        .as_deref()
        .unwrap_or(mal_backend::pipeline::GENERATED_HEADER_NAME);
    deliver(
        crate::driver::emit_host(&parsed.source, header),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    fn usage_message(values: &[&str]) -> String {
        let outcome = execute(args(values));
        assert_eq!(outcome.status, ExitStatus::UsageError, "{values:?}");
        assert!(outcome.stdout.is_empty());
        outcome.stderr
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
        assert_eq!(outcome.stdout, "malc 0.6.0-dev (language v0.6)\n");
    }

    #[test]
    fn unknown_commands_are_usage_errors() {
        assert!(usage_message(&["unknown", "sample.mal"]).contains("unknown command"));
        assert!(usage_message(&["format", "sample.mal"]).contains("unknown command"));
        assert!(usage_message(&["emit", "bogus", "sample.mal"]).contains("emit requires"));
    }

    #[test]
    fn validates_build_options_before_running_the_driver() {
        assert!(usage_message(&["build", "sample.mal"]).contains("build requires --output"));
        assert!(usage_message(&["build"]).contains("build requires a source path"));
        assert!(
            usage_message(&["build", "sample.mal", "-o", "program", "--link", "host.c"])
                .contains("unknown build option '--link'")
        );
        assert!(
            usage_message(&["build", "sample.mal", "-o", "one", "--output", "two"])
                .contains("--output may only be specified once")
        );
        assert!(
            usage_message(&[
                "build",
                "sample.mal",
                "-o",
                "program",
                "--artifact-dir",
                "one",
                "--artifact-dir",
                "two",
            ])
            .contains("--artifact-dir may only be specified once")
        );
        assert!(
            usage_message(&[
                "build",
                "sample.mal",
                "-o",
                "program",
                "--optimization",
                "fast",
            ])
            .contains("must be 'baseline' or 'production'")
        );
        assert!(
            usage_message(&["build", "one.mal", "two.mal", "-o", "program"])
                .contains("build takes one source path")
        );
        assert!(usage_message(&["build", "sample.mal", "-o"]).contains("requires a value"));
    }

    #[test]
    fn emit_commands_accept_only_their_own_options() {
        assert!(
            usage_message(&["emit", "header", "sample.mal", "--optimization", "baseline"])
                .contains("unknown emit header option '--optimization'")
        );
        assert!(
            usage_message(&["emit", "atcoder", "sample.mal", "--target", "other"])
                .contains("unknown emit atcoder option '--target'")
        );
        assert!(
            usage_message(&["emit", "host", "sample.mal", "--header", "invalid\"name.h"])
                .contains("not valid in a quoted C include")
        );
    }
}
