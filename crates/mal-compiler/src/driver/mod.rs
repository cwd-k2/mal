use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use mal_syntax::graph;

mod build;
mod toolchain;

pub use build::{BuildOptions, OptimizationMode, build, emit_atcoder};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

pub fn check(source_path: &Path) -> Result<(), Error> {
    let graph = graph::load(source_path)?;
    mal_frontend::analysis::check_graph(&graph)
        .map(|_| ())
        .map_err(|error| Error::diagnostic(error, &graph))
}

pub fn emit_header(source_path: &Path, output_path: &Path) -> Result<(), Error> {
    let graph = graph::load(source_path)?;
    let header = crate::pipeline::emit_header_graph(&graph)
        .map_err(|error| Error::diagnostic(error, &graph))?;
    create_parent(output_path)?;
    fs::write(output_path, header)
        .map_err(|error| Error::io("write generated header", output_path, error))
}

pub fn emit_host(source_path: &Path, header_name: &str) -> Result<String, Error> {
    let graph = graph::load(source_path)?;
    crate::pipeline::emit_host_graph(&graph, header_name)
        .map_err(|error| Error::diagnostic(error, &graph))
}

fn create_parent(path: &Path) -> Result<(), Error> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| Error::io("create output directory", parent, error))
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Result<Self, Error> {
        for _ in 0..100 {
            let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("malc-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(Error::io("create temporary directory", &path, error)),
            }
        }
        Err(Error::new(
            "could not allocate a unique temporary directory",
        ))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn diagnostic(
        error: mal_syntax::diagnostic::Diagnostic,
        sources: &impl mal_syntax::source::SourceProvider,
    ) -> Self {
        Self::new(error.render(sources))
    }

    fn io(action: &str, path: &Path, error: std::io::Error) -> Self {
        Self::new(format!(
            "malc: cannot {action} '{}': {error}",
            path.display()
        ))
    }

    fn tool_start(tool: &OsStr, error: std::io::Error) -> Self {
        Self::new(format!(
            "malc: cannot run C compiler '{}': {error}",
            tool.to_string_lossy()
        ))
    }

    fn tool_failure(tool: &OsStr, code: Option<i32>, stderr: &[u8]) -> Self {
        let status = code.map_or_else(|| "signal".into(), |code| format!("exit status {code}"));
        let details = String::from_utf8_lossy(stderr);
        Self::new(format!(
            "malc: C compiler '{}' failed with {status}\n{details}",
            tool.to_string_lossy()
        ))
    }

    fn backend(
        error: crate::backend::llvm::Error,
        sources: &impl mal_syntax::source::SourceProvider,
    ) -> Self {
        match error {
            crate::backend::llvm::Error::Diagnostic(diagnostic) => {
                Self::diagnostic(diagnostic, sources)
            }
            error => Self::new(format!("malc: LLVM backend failure: {error}")),
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
