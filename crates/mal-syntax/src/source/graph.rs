//! The files of one program in load order, with their requirements.

use super::*;

/// The files of one program in load order, with the `.mal` requirements of each file and the `.c` files to link.
///
/// A `FileId` is the position of its file in `files`, and `requirements[i]` lists the requirements of file `i` in source
/// order. `SourceGraph::new` panics when these invariants do not hold.
#[derive(Debug)]
pub struct SourceGraph {
    root: FileId,
    files: Vec<SourceFile>,
    requirements: Vec<Vec<SourceRequirement>>,
    c_sources: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A resolved mal-source requirement edge.
pub struct SourceRequirement {
    /// The required file's graph identity.
    pub target: FileId,
    /// The requirement path span in the requiring file.
    pub span: Span,
}

impl SourceGraph {
    /// Builds a graph from files whose ids equal their positions; `root` must be one of them.
    pub fn new(
        root: FileId,
        files: Vec<SourceFile>,
        requirements: Vec<Vec<SourceRequirement>>,
        c_sources: Vec<PathBuf>,
    ) -> Self {
        assert_eq!(files.len(), requirements.len());
        assert!(root.index() < files.len() as u32);
        for (index, file) in files.iter().enumerate() {
            assert_eq!(file.id().index(), index as u32);
            for (requirement_index, requirement) in requirements[index].iter().enumerate() {
                assert!(requirement.target.index() < files.len() as u32);
                assert!(file.contains(requirement.span));
                assert!(
                    requirements[index][..requirement_index]
                        .iter()
                        .all(|previous| previous.target != requirement.target)
                );
            }
        }
        for (index, path) in c_sources.iter().enumerate() {
            assert!(!c_sources[..index].contains(path));
        }
        Self {
            root,
            files,
            requirements,
            c_sources,
        }
    }

    /// Returns the entry source selected by the caller.
    pub const fn root(&self) -> FileId {
        self.root
    }

    /// Borrows the entry source; construction guarantees that it exists.
    pub fn root_source(&self) -> &SourceFile {
        self.source(self.root)
            .expect("a source graph always contains its root")
    }

    /// Returns files in stable identity and admission order.
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Looks up a file by its graph identity.
    pub fn source(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.index() as usize)
    }

    /// Returns direct mal source requirements in source order, or an empty slice for an unknown identity.
    pub fn requirements(&self, id: FileId) -> &[SourceRequirement] {
        self.requirements
            .get(id.index() as usize)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Returns deduplicated C build inputs reached while loading the requirement graph.
    pub fn c_sources(&self) -> &[PathBuf] {
        &self.c_sources
    }
}
