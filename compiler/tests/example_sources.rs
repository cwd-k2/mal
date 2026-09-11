use std::path::{Path, PathBuf};

use malc::lexer::TokenKind;
use malc::source::{FileId, SourceFile};

fn example_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a repository parent")
        .join("examples");
    let mut pending = vec![root];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("example directory") {
            let path = entry.expect("example entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mal") {
                sources.push(path);
            }
        }
    }
    sources.sort();
    sources
}

fn source(path: &Path) -> SourceFile {
    SourceFile::new(
        FileId::new(0),
        path.display().to_string(),
        std::fs::read_to_string(path).expect("example source"),
    )
}

#[test]
fn every_example_source_is_canonical_and_idempotent() {
    for path in example_sources() {
        let source = source(&path);
        let formatted = malc::formatter::format(&source).expect("formatted example");
        assert_eq!(formatted, source.text(), "{}", path.display());
        let formatted_source = SourceFile::new(FileId::new(0), path, formatted.clone());
        assert_eq!(
            malc::formatter::format(&formatted_source).expect("reformatted example"),
            formatted
        );
    }
}

#[test]
fn examples_construct_sum_variants_through_return_binders() {
    for path in example_sources() {
        let source = source(&path);
        let tokens = malc::lexer::lex(&source).expect("lexed example");
        let indexed_constructor = tokens.windows(5).any(|tokens| {
            matches!(tokens[0].kind, TokenKind::Integer(_))
                && matches!(tokens[1].kind, TokenKind::LeftBracket)
                && matches!(tokens[2].kind, TokenKind::TypeIdentifier)
                && matches!(tokens[3].kind, TokenKind::RightBracket)
                && matches!(tokens[4].kind, TokenKind::LeftParen)
        });
        assert!(
            !indexed_constructor,
            "{} contains an indexed sum constructor",
            path.display()
        );
    }
}
