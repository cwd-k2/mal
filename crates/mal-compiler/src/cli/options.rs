//! The options each command accepts, parsed in any order around one source path.

use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Flag {
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

pub(super) const TOOLCHAIN_FLAGS: &[Flag] = &[
    Flag::Output,
    Flag::ArtifactDirectory,
    Flag::ClangArgument,
    Flag::Optimization,
];

#[derive(Default)]
pub(super) struct Parsed {
    pub(super) source: PathBuf,
    pub(super) output: Option<PathBuf>,
    pub(super) artifact_directory: Option<PathBuf>,
    pub(super) clang_arguments: Vec<OsString>,
    pub(super) optimization: Option<OptimizationMode>,
    pub(super) header: Option<String>,
}

impl Parsed {
    pub(super) fn toolchain(&self) -> ToolchainOptions<'_> {
        ToolchainOptions {
            artifact_directory: self.artifact_directory.as_deref(),
            clang_arguments: &self.clang_arguments,
            optimization: self.optimization.unwrap_or(OptimizationMode::Production),
        }
    }
}

/// Parses one source path and the options `allowed` for `command`, in any order.
pub(super) fn parse(
    command: &str,
    arguments: &[OsString],
    allowed: &[Flag],
) -> Result<Parsed, Outcome> {
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
