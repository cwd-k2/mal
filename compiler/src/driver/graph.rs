use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::diagnostic::Diagnostic;
use crate::source::{FileId, SourceFile, SourceGraph, SourceRequirement};

use super::Error;

#[derive(Clone, Copy)]
enum State {
    Loading,
    Loaded(FileId),
}

pub(super) fn load(root: &Path) -> Result<SourceGraph, Error> {
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

pub(super) fn load_with_overlays(
    root: &Path,
    root_text: &str,
    overlays: &HashMap<PathBuf, String>,
) -> Result<SourceGraph, Error> {
    let root_path = canonicalize_or_absolute(root)?;
    let mut normalized = overlays
        .iter()
        .map(|(path, text)| Ok((canonicalize_or_absolute(path)?, text.as_str())))
        .collect::<Result<HashMap<_, _>, Error>>()?;
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
    ) -> Result<FileId, Error> {
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
                return Err(Error::diagnostic(diagnostic, &self.files));
            }
            None => {}
        }

        let id = FileId::new(self.files.len() as u32);
        let source = self.overlays.get(path).map_or_else(
            || SourceFile::load(id, path).map_err(Error::source),
            |text| Ok(SourceFile::new(id, path, (*text).to_owned())),
        )?;
        let parsed =
            crate::parser::parse(&source).map_err(|error| Error::diagnostic(error, &source))?;
        self.states.insert(path.to_owned(), State::Loading);
        self.files.push(source);
        self.requirements.push(Vec::new());

        for required in parsed.requirements {
            let source = &self.files[id.index() as usize];
            let required_path =
                requirement_path(source, &required.kind.path, required.kind.path_span)
                    .map_err(|error| Error::diagnostic(error, &self.files))?;
            let kind = match required_path
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("mal") => RequirementKind::Mal,
                Some("c") => RequirementKind::C,
                _ => {
                    let diagnostic = Diagnostic::error("invalid requirement path extension")
                        .with_primary(required.kind.path_span, "expected a `.mal` or `.c` path");
                    return Err(Error::diagnostic(diagnostic, &self.files));
                }
            };
            match kind {
                RequirementKind::Mal => {
                    let canonical = canonicalize_mal_requirement(
                        &required_path,
                        required.kind.path_span,
                        self.overlays,
                    )
                    .map_err(|error| Error::diagnostic(error, &self.files))?;
                    let dependency = self.load_mal(&canonical, Some(required.kind.path_span))?;
                    let requirements = &mut self.requirements[id.index() as usize];
                    if !requirements
                        .iter()
                        .any(|requirement| requirement.target == dependency)
                    {
                        requirements.push(SourceRequirement {
                            target: dependency,
                            span: required.kind.path_span,
                        });
                    }
                }
                RequirementKind::C => {
                    let canonical =
                        canonicalize_requirement(&required_path, required.kind.path_span)
                            .map_err(|error| Error::diagnostic(error, &self.files))?;
                    if self.seen_c_sources.insert(canonical.clone()) {
                        self.c_sources.push(canonical);
                    }
                }
            }
        }
        self.states.insert(path.to_owned(), State::Loaded(id));
        Ok(id)
    }
}

#[derive(Clone, Copy)]
enum RequirementKind {
    Mal,
    C,
}

fn canonicalize_root(path: &Path) -> Result<PathBuf, Error> {
    std::fs::canonicalize(path).map_err(|error| Error::io("read source", path, error))
}

fn canonicalize_or_absolute(path: &Path) -> Result<PathBuf, Error> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) if path.is_absolute() => Ok(path.to_owned()),
        Err(_) => std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| Error::io("resolve source path", path, error)),
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
