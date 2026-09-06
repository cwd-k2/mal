use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::source::{FileId, SourceFile, SourceGraph};

mod graph;

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

const C_COMPILER_OPTIONS: &[&str] = &[
    "-std=c11",
    "-Wall",
    "-Wextra",
    "-Werror",
    "-pedantic",
    "-O2",
    "-fno-fast-math",
    "-ffp-contract=off",
    "-frounding-math",
    "-fexcess-precision=standard",
];

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

pub fn emit_c(source_path: &Path, output_path: &Path) -> Result<PathBuf, Error> {
    let graph = graph::load(source_path)?;
    let generated =
        crate::pipeline::emit_c_graph(&graph).map_err(|error| Error::diagnostic(error, &graph))?;
    write_generated(output_path, generated)
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

pub fn build(source_path: &Path, output_path: &Path) -> Result<(), Error> {
    let graph = graph::load(source_path)?;
    let generated =
        crate::pipeline::emit_c_graph(&graph).map_err(|error| Error::diagnostic(error, &graph))?;
    let temporary = TemporaryDirectory::new()?;
    let generated_path = temporary.path().join("program.c");
    write_generated(&generated_path, generated)?;
    create_parent(output_path)?;

    let compiler = std::env::var_os("CC").unwrap_or_else(|| OsString::from("clang"));
    let result = Command::new(&compiler)
        .args(C_COMPILER_OPTIONS)
        .arg("-I")
        .arg(temporary.path())
        .arg(&generated_path)
        .args(graph.c_sources())
        .arg("-o")
        .arg(output_path)
        .output()
        .map_err(|error| Error::tool_start(&compiler, error))?;
    if !result.status.success() {
        return Err(Error::tool_failure(
            &compiler,
            result.status.code(),
            &result.stderr,
        ));
    }
    Ok(())
}

fn write_generated(output_path: &Path, generated: crate::c_emit::Output) -> Result<PathBuf, Error> {
    create_parent(output_path)?;
    fs::write(output_path, generated.source)
        .map_err(|error| Error::io("write generated C", output_path, error))?;
    let header_path = output_path.with_file_name(crate::c_emit::GENERATED_HEADER_NAME);
    fs::write(&header_path, generated.header)
        .map_err(|error| Error::io("write generated header", &header_path, error))?;
    Ok(header_path)
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
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::C_COMPILER_OPTIONS;

    #[test]
    fn public_build_uses_optimization_with_the_strict_float_profile() {
        assert_eq!(
            C_COMPILER_OPTIONS,
            [
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-pedantic",
                "-O2",
                "-fno-fast-math",
                "-ffp-contract=off",
                "-frounding-math",
                "-fexcess-precision=standard",
            ]
        );
    }
}
