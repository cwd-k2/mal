use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::diagnostic::Diagnostic;
use crate::source::{FileId, SourceFile, SourceGraph, SourceRequirement};

use std::fmt;

/// Failure to load a source graph from the file system or an overlay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
    /// A source diagnostic already rendered against the loaded files.
    Rendered(String),
    /// A file system or encoding failure reported without source context.
    Failure(String),
}

impl LoadError {
    fn diagnostic(error: Diagnostic, sources: &impl crate::source::SourceProvider) -> Self {
        Self::Rendered(error.render(sources))
    }

    fn source(error: crate::source::SourceLoadError) -> Self {
        Self::Failure(error.to_string())
    }

    fn io(action: &str, path: &Path, error: std::io::Error) -> Self {
        Self::Failure(format!("cannot {action} '{}': {error}", path.display()))
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rendered(message) | Self::Failure(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Clone, Copy)]
enum State {
    Loading,
    Loaded(FileId),
}

/// Loads `root` and every `.mal` file it requires, transitively, from disk.
///
/// Each canonical path is loaded once. A requirement cycle, an unreadable file, and invalid source are errors.
pub fn load(root: &Path) -> Result<SourceGraph, LoadError> {
    let root_path = canonicalize_root(root)?;
    let overlays = HashMap::new();
    let mut builder = Builder::new(&overlays);
    let root = builder.load_mal(&root_path, None)?;
    Ok(SourceGraph::new(
        root,
        builder.files,
        builder.requirements,
        builder.c_sources,
    ))
}

/// Like `load`, but `root_text` replaces the contents of `root` and `overlays` replace other files, so an editor can
/// analyze unsaved buffers. An overlaid path does not need to exist on disk.
pub fn load_with_overlays(
    root: &Path,
    root_text: &str,
    overlays: &HashMap<PathBuf, String>,
) -> Result<SourceGraph, LoadError> {
    let root_path = canonicalize_or_absolute(root)?;
    let mut normalized = overlays
        .iter()
        .map(|(path, text)| Ok((canonicalize_or_absolute(path)?, text.as_str())))
        .collect::<Result<HashMap<_, _>, LoadError>>()?;
    normalized.insert(root_path.clone(), root_text);
    let mut builder = Builder::new(&normalized);
    let root = builder.load_mal(&root_path, None)?;
    Ok(SourceGraph::new(
        root,
        builder.files,
        builder.requirements,
        builder.c_sources,
    ))
}

struct Builder<'a> {
    files: Vec<SourceFile>,
    requirements: Vec<Vec<SourceRequirement>>,
    states: HashMap<PathBuf, State>,
    c_sources: Vec<PathBuf>,
    seen_c_sources: HashSet<PathBuf>,
    overlays: &'a HashMap<PathBuf, &'a str>,
}

struct PendingFile {
    path: PathBuf,
    id: FileId,
    requirements: std::vec::IntoIter<crate::ast::Node<crate::ast::Requirement>>,
    requested_by: Option<(FileId, crate::source::Span)>,
}

impl<'a> Builder<'a> {
    fn new(overlays: &'a HashMap<PathBuf, &'a str>) -> Self {
        Self {
            files: Vec::new(),
            requirements: Vec::new(),
            states: HashMap::new(),
            c_sources: Vec::new(),
            seen_c_sources: HashSet::new(),
            overlays,
        }
    }

    fn load_mal(
        &mut self,
        path: &Path,
        requirement_span: Option<crate::source::Span>,
    ) -> Result<FileId, LoadError> {
        match self.states.get(path).copied() {
            Some(State::Loaded(id)) => return Ok(id),
            Some(State::Loading) => {
                let diagnostic = Diagnostic::error("cyclic `.mal` requirement").with_primary(
                    requirement_span.expect("only a dependency can form a cycle"),
                    format!(
                        "this reaches `{}` while it is still loading",
                        path.display()
                    ),
                );
                return Err(LoadError::diagnostic(diagnostic, &self.files));
            }
            None => {}
        }

        let root = self.begin_mal(path, None)?;
        let root_id = root.id;
        let mut pending = vec![root];

        while let Some(frame) = pending.last_mut() {
            let Some(required) = frame.requirements.next() else {
                let completed = pending.pop().expect("pending file exists");
                self.states
                    .insert(completed.path, State::Loaded(completed.id));
                if let Some((importer, span)) = completed.requested_by {
                    self.add_mal_requirement(importer, completed.id, span);
                }
                continue;
            };
            let importer = frame.id;
            let source = &self.files[importer.index() as usize];
            let required_path =
                requirement_path(source, &required.kind.path, required.kind.path_span)
                    .map_err(|error| LoadError::diagnostic(error, &self.files))?;
            let kind = match required_path
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("mal") => RequirementKind::Mal,
                Some("c") => RequirementKind::C,
                _ => {
                    let diagnostic = Diagnostic::error("invalid requirement path extension")
                        .with_primary(required.kind.path_span, "expected a `.mal` or `.c` path");
                    return Err(LoadError::diagnostic(diagnostic, &self.files));
                }
            };
            match kind {
                RequirementKind::Mal => {
                    let canonical = canonicalize_mal_requirement(
                        &required_path,
                        required.kind.path_span,
                        self.overlays,
                    )
                    .map_err(|error| LoadError::diagnostic(error, &self.files))?;
                    match self.states.get(&canonical).copied() {
                        Some(State::Loaded(dependency)) => {
                            self.add_mal_requirement(importer, dependency, required.kind.path_span)
                        }
                        Some(State::Loading) => {
                            let diagnostic = Diagnostic::error("cyclic `.mal` requirement")
                                .with_primary(
                                    required.kind.path_span,
                                    format!(
                                        "this reaches `{}` while it is still loading",
                                        canonical.display()
                                    ),
                                );
                            return Err(LoadError::diagnostic(diagnostic, &self.files));
                        }
                        None => pending.push(
                            self.begin_mal(&canonical, Some((importer, required.kind.path_span)))?,
                        ),
                    }
                }
                RequirementKind::C => {
                    let canonical =
                        canonicalize_requirement(&required_path, required.kind.path_span)
                            .map_err(|error| LoadError::diagnostic(error, &self.files))?;
                    if self.seen_c_sources.insert(canonical.clone()) {
                        self.c_sources.push(canonical);
                    }
                }
            }
        }
        Ok(root_id)
    }

    fn begin_mal(
        &mut self,
        path: &Path,
        requested_by: Option<(FileId, crate::source::Span)>,
    ) -> Result<PendingFile, LoadError> {
        let id = FileId::new(self.files.len() as u32);
        let source = self.overlays.get(path).map_or_else(
            || SourceFile::load(id, path).map_err(LoadError::source),
            |text| Ok(SourceFile::new(id, path, (*text).to_owned())),
        )?;
        let parsed =
            crate::parser::parse(&source).map_err(|error| LoadError::diagnostic(error, &source))?;
        self.states.insert(path.to_owned(), State::Loading);
        self.files.push(source);
        self.requirements.push(Vec::new());
        Ok(PendingFile {
            path: path.to_owned(),
            id,
            requirements: parsed.requirements.into_iter(),
            requested_by,
        })
    }

    fn add_mal_requirement(
        &mut self,
        importer: FileId,
        dependency: FileId,
        span: crate::source::Span,
    ) {
        let requirements = &mut self.requirements[importer.index() as usize];
        if !requirements
            .iter()
            .any(|requirement| requirement.target == dependency)
        {
            requirements.push(SourceRequirement {
                target: dependency,
                span,
            });
        }
    }
}

#[derive(Clone, Copy)]
enum RequirementKind {
    Mal,
    C,
}

fn canonicalize_root(path: &Path) -> Result<PathBuf, LoadError> {
    std::fs::canonicalize(path).map_err(|error| LoadError::io("read source", path, error))
}

fn canonicalize_or_absolute(path: &Path) -> Result<PathBuf, LoadError> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) if path.is_absolute() => Ok(path.to_owned()),
        Err(_) => std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| LoadError::io("resolve source path", path, error)),
    }
}

fn requirement_path(
    source: &SourceFile,
    bytes: &[u8],
    span: crate::source::Span,
) -> Result<PathBuf, Diagnostic> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        Diagnostic::error("invalid requirement path")
            .with_primary(span, "requirement paths must be UTF-8")
    })?;
    let Some(path) = super::requirement::relative_requirement_path(source.path(), text) else {
        return Err(Diagnostic::error("invalid requirement path")
            .with_primary(span, "expected a non-empty relative path"));
    };
    Ok(path)
}

fn canonicalize_requirement(path: &Path, span: crate::source::Span) -> Result<PathBuf, Diagnostic> {
    std::fs::canonicalize(path).map_err(|error| {
        Diagnostic::error("cannot load requirement")
            .with_primary(span, format!("cannot read `{}`", path.display()))
            .with_note(error.to_string())
    })
}

fn canonicalize_mal_requirement(
    path: &Path,
    span: crate::source::Span,
    overlays: &HashMap<PathBuf, &str>,
) -> Result<PathBuf, Diagnostic> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) if overlays.contains_key(path) => Ok(path.to_owned()),
        Err(_) => canonicalize_requirement(path, span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_deep_requirement_chains_without_host_recursion() {
        let directory = Path::new("/tmp/malc-deep-source-graph");
        let depth = 4096;
        let overlays = (0..depth)
            .map(|index| {
                let path = directory.join(format!("file{index}.mal"));
                let text = if index + 1 == depth {
                    "value := 0;".to_owned()
                } else {
                    format!("require \"./file{}.mal\"; value{index} := 0;", index + 1)
                };
                (path, text)
            })
            .collect::<HashMap<_, _>>();
        let root = directory.join("file0.mal");

        let graph = load_with_overlays(&root, &overlays[&root], &overlays).unwrap();

        assert_eq!(graph.files().len(), depth);
    }
}
