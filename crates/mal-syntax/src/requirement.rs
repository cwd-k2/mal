use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequirementPathCandidate {
    pub name: String,
    pub is_directory: bool,
}

pub fn resolve_requirement_path(source_path: &Path, requirement: &str) -> Option<PathBuf> {
    let path = relative_requirement_path(source_path, requirement)?;
    Some(fs::canonicalize(&path).unwrap_or(path))
}

pub fn relative_requirement_path(source_path: &Path, requirement: &str) -> Option<PathBuf> {
    let requirement = Path::new(requirement);
    if requirement.as_os_str().is_empty() || requirement.is_absolute() {
        return None;
    }
    Some(
        source_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(requirement),
    )
}

pub fn requirement_path_candidates(
    source_path: &Path,
    fragment: &str,
) -> Vec<RequirementPathCandidate> {
    if Path::new(fragment).is_absolute() {
        return Vec::new();
    }
    let (directory, prefix) = fragment
        .rsplit_once('/')
        .map_or(("", fragment), |(directory, prefix)| (directory, prefix));
    let Some(directory) = resolve_requirement_path(source_path, directory_or_current(directory))
    else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut candidates = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            if !name.starts_with(prefix) {
                return None;
            }
            let path = entry.path();
            let is_directory = path.is_dir();
            let supported_file = path.is_file()
                && matches!(path.extension().and_then(OsStr::to_str), Some("mal" | "c"));
            (is_directory || supported_file).then_some(RequirementPathCandidate {
                name: if is_directory {
                    format!("{name}/")
                } else {
                    name
                },
                is_directory,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.name.cmp(&right.name));
    candidates
}

fn directory_or_current(directory: &str) -> &str {
    if directory.is_empty() { "." } else { directory }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("mal-requirement-test-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir(&path).expect("create test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn lists_only_supported_requirement_paths() {
        let directory = TestDirectory::new();
        let source = directory.path().join("program.mal");
        fs::write(directory.path().join("library.mal"), "").expect("write mal source");
        fs::write(directory.path().join("library.c"), "").expect("write C source");
        fs::write(directory.path().join("library.txt"), "").expect("write unrelated file");
        fs::create_dir(directory.path().join("libdir")).expect("create source directory");

        let candidates = requirement_path_candidates(&source, "lib");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.name.as_str())
                .collect::<Vec<_>>(),
            ["libdir/", "library.c", "library.mal"]
        );
    }
}
