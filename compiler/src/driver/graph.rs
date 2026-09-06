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
    let mut builder = Builder::default();
    let root = builder.load_mal(&root_path, None)?;
    Ok(SourceGraph::new(
        root,
        builder.files,
        builder.requirements,
        builder.c_sources,
    ))
}

#[derive(Default)]
struct Builder {
    files: Vec<SourceFile>,
    requirements: Vec<Vec<SourceRequirement>>,
    states: HashMap<PathBuf, State>,
    c_sources: Vec<PathBuf>,
    seen_c_sources: HashSet<PathBuf>,
}

impl Builder {
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
        let source = SourceFile::load(id, path).map_err(Error::source)?;
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
                    let diagnostic = Diagnostic::error("unsupported requirement type")
                        .with_primary(required.kind.path_span, "expected a `.mal` or `.c` path");
                    return Err(Error::diagnostic(diagnostic, &self.files));
                }
            };
            let canonical = canonicalize_requirement(&required_path, required.kind.path_span)
                .map_err(|error| Error::diagnostic(error, &self.files))?;
            match kind {
                RequirementKind::Mal => {
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

fn requirement_path(
    source: &SourceFile,
    bytes: &[u8],
    span: crate::source::Span,
) -> Result<PathBuf, Diagnostic> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        Diagnostic::error("invalid requirement path")
            .with_primary(span, "requirement paths must be UTF-8")
    })?;
    let path = Path::new(text);
    if text.is_empty() || path.is_absolute() {
        return Err(Diagnostic::error("invalid requirement path")
            .with_primary(span, "expected a non-empty relative path"));
    }
    Ok(source
        .path()
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(path))
}

fn canonicalize_requirement(path: &Path, span: crate::source::Span) -> Result<PathBuf, Diagnostic> {
    std::fs::canonicalize(path).map_err(|error| {
        Diagnostic::error("cannot load requirement")
            .with_primary(span, format!("cannot read `{}`", path.display()))
            .with_note(error.to_string())
    })
}
