use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::source::{FileId, SourceFile, SourceGraph};

mod build;
mod graph;
mod requirement;
mod toolchain;

pub use build::{BuildOptions, OptimizationMode, build, emit_atcoder};
pub use requirement::{
    RequirementPathCandidate, requirement_path_candidates, resolve_requirement_path,
};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

pub fn check(source_path: &Path) -> Result<(), Error> {
    let graph = graph::load(source_path)?;
    crate::pipeline::check_graph(&graph)
        .map(|_| ())
        .map_err(|error| Error::diagnostic(error, &graph))
}

pub fn load_source_graph_with_overlays(
    root_path: &Path,
    root_text: &str,
    overlays: &HashMap<PathBuf, String>,
) -> Result<SourceGraph, Error> {
    graph::load_with_overlays(root_path, root_text, overlays)
}

pub fn format(source_path: &Path) -> Result<String, Error> {
    let source = SourceFile::load(FileId::new(0), source_path).map_err(Error::source)?;
    crate::formatter::format(&source).map_err(|error| Error::diagnostic(error, &source))
}

pub fn format_in_place(source_path: &Path) -> Result<(), Error> {
    let formatted = format(source_path)?;
    let permissions = fs::metadata(source_path)
        .map_err(|error| Error::io("read source metadata", source_path, error))?
        .permissions();
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = source_path
        .file_name()
        .ok_or_else(|| Error::new("malc: format source path has no file name"))?;

    for _ in 0..100 {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(
            ".{}.malc-format-{}-{sequence}",
            file_name.to_string_lossy(),
            std::process::id()
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(Error::io(
                    "create temporary formatted source",
                    &temporary_path,
                    error,
                ));
            }
        };
        let temporary = TemporaryFile(temporary_path);
        file.write_all(formatted.as_bytes())
            .map_err(|error| Error::io("write formatted source", &temporary.0, error))?;
        file.sync_all()
            .map_err(|error| Error::io("sync formatted source", &temporary.0, error))?;
        file.set_permissions(permissions)
            .map_err(|error| Error::io("preserve source permissions", &temporary.0, error))?;
        drop(file);
        fs::rename(&temporary.0, source_path)
            .map_err(|error| Error::io("replace source with formatted text", source_path, error))?;
        return Ok(());
    }
    Err(Error::new(
        "malc: could not allocate a temporary formatted source",
    ))
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

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

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

    fn source(error: crate::source::SourceLoadError) -> Self {
        Self::new(format!("malc: {error}"))
    }

    fn diagnostic(
        error: crate::diagnostic::Diagnostic,
        sources: &impl crate::source::SourceProvider,
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
        sources: &impl crate::source::SourceProvider,
    ) -> Self {
        match error {
            crate::backend::llvm::Error::Diagnostic(diagnostic) => {
                Self::diagnostic(diagnostic, sources)
            }
            error => Self::new(format!("malc: LLVM backend failure: {error}")),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
