use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{FileId, SourceFile};

fn source_file(id: u32, text: String) -> SourceFile {
    SourceFile::new(FileId::new(id), "robustness-test.mal", text)
}

fn assert_valid_diagnostic(source: &SourceFile, diagnostic: &Diagnostic) {
    if let Some(primary) = &diagnostic.primary {
        assert!(source.contains(primary.span), "{}", diagnostic.message);
    }
    assert!(!diagnostic.render(source).contains("<invalid span>"));
}

fn exercise(id: u32, text: String) {
    let source = source_file(id, text);
    match mal_fmt::format(&source) {
        Ok(formatted) => {
            let formatted_source = source_file(id, formatted.clone());
            assert_eq!(
                mal_fmt::format(&formatted_source).expect("reformat accepted source"),
                formatted
            );
        }
        Err(diagnostic) => assert_valid_diagnostic(&source, &diagnostic),
    }
    if let Err(diagnostic) = mal_frontend::analysis::check(&source) {
        assert_valid_diagnostic(&source, &diagnostic);
    }
}

#[test]
fn source_mutations_preserve_frontend_totality_and_formatter_idempotence() {
    let seeds = [
        "main::Unit->Int32:=() -> {40+2;};",
        "choose::Bool->Int32:=(condition) -> {if(condition)then{1}else{2};};",
        "Result::[Int32,Symbol];create::Bool->Result:=(ok) -> [yes,no] => {when(ok){yes(1)};no(\"失敗\")};",
        "pair::(UInt8,UInt64):=(1u8,2u64);",
        "extern Handle;extern consume::Handle->Unit;",
    ];
    let replacements = [';', '(', '}', '"', '#', '\n', 'é'];
    let mut id = 0;

    for seed in seeds {
        let characters = seed.chars().collect::<Vec<_>>();
        exercise(id, seed.into());
        id += 1;
        for index in 0..=characters.len() {
            let replacement = replacements[index % replacements.len()];
            let mut inserted = characters.clone();
            inserted.insert(index, replacement);
            exercise(id, inserted.iter().collect());
            id += 1;

            if index < characters.len() {
                let mut deleted = characters.clone();
                deleted.remove(index);
                exercise(id, deleted.iter().collect());
                id += 1;

                let mut replaced = characters.clone();
                replaced[index] = replacement;
                exercise(id, replaced.iter().collect());
                id += 1;
            }
        }
    }
}

#[test]
fn accepts_ordinary_nesting() {
    let ordinary = format!(
        "main :: Unit -> Int32 := () -> {{ {}0i32{}; }};",
        "(".repeat(32),
        ")".repeat(32)
    );
    let source = source_file(900, ordinary);
    mal_frontend::analysis::check(&source).expect("ordinary nesting must check");
    mal_fmt::format(&source).expect("ordinary nesting must format");
}
