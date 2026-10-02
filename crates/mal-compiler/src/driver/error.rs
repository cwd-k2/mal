//! Failures of the driver, rendered once into the message that `malc` prints.

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use mal_syntax::graph;

#[derive(Clone, Debug, Eq, PartialEq)]
/// A failure whose message is ready to print; source diagnostics are already rendered against their files.
pub struct Error {
    message: String,
}

impl Error {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(super) fn diagnostic(
        error: mal_syntax::diagnostic::Diagnostic,
        sources: &impl mal_syntax::source::SourceProvider,
    ) -> Self {
        Self::new(error.render(sources))
    }

    pub(super) fn io(action: &str, path: &Path, error: std::io::Error) -> Self {
        Self::new(format!(
            "malc: cannot {action} '{}': {error}",
            path.display()
        ))
    }

    pub(super) fn tool_start(tool: &OsStr, error: std::io::Error) -> Self {
        Self::new(format!(
            "malc: cannot run C compiler '{}': {error}",
            tool.to_string_lossy()
        ))
    }

    pub(super) fn tool_failure(tool: &OsStr, code: Option<i32>, stderr: &[u8]) -> Self {
        let status = code.map_or_else(|| "signal".into(), |code| format!("exit status {code}"));
        let details = String::from_utf8_lossy(stderr);
        Self::new(format!(
            "malc: C compiler '{}' failed with {status}\n{details}",
            tool.to_string_lossy()
        ))
    }

    pub(super) fn backend(
        error: mal_backend::pipeline::GenerateError,
        sources: &impl mal_syntax::source::SourceProvider,
    ) -> Self {
        match error {
            mal_backend::pipeline::GenerateError::Diagnostic(diagnostic)
            | mal_backend::pipeline::GenerateError::Backend(
                mal_backend::pipeline::BackendError::Diagnostic(diagnostic),
            ) => Self::diagnostic(diagnostic, sources),
            mal_backend::pipeline::GenerateError::Backend(error) => {
                Self::new(format!("malc: LLVM backend failure: {error}"))
            }
        }
    }
}

impl From<graph::LoadError> for Error {
    fn from(error: graph::LoadError) -> Self {
        match error {
            graph::LoadError::Rendered(message) => Self::new(message),
            graph::LoadError::Failure(message) => Self::new(format!("malc: {message}")),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
