use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use mal_syntax::source::{FileId, SourceFile};

static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

/// Failure to read, format, or replace a source file. The message is ready to print.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Error(String);

impl Error {
    fn io(action: &str, path: &Path, error: std::io::Error) -> Self {
        Self(format!(
            "mal-fmt: cannot {action} '{}': {error}",
            path.display()
        ))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub(crate) fn format_file(path: &Path) -> Result<String, Error> {
    let source = SourceFile::load(FileId::new(0), path)
        .map_err(|error| Error(format!("mal-fmt: {error}")))?;
    crate::format(&source).map_err(|error| Error(error.render(&source)))
}

/// Replaces `path` with its canonical form through a temporary file in the same directory,
/// so a failure leaves the original untouched.
pub(crate) fn write_file(path: &Path) -> Result<(), Error> {
    let formatted = format_file(path)?;
    let permissions = fs::metadata(path)
        .map_err(|error| Error::io("read source metadata", path, error))?
        .permissions();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| Error("mal-fmt: source path has no file name".into()))?;

    for _ in 0..100 {
        let sequence = NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(
            ".{}.mal-fmt-{}-{sequence}",
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
        fs::rename(&temporary.0, path)
            .map_err(|error| Error::io("replace source with formatted text", path, error))?;
        return Ok(());
    }
    Err(Error(
        "mal-fmt: could not allocate a temporary formatted source".into(),
    ))
}

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
