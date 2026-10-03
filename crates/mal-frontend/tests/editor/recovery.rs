use super::*;

#[test]
fn keeps_resolved_identities_and_independent_checked_items_after_a_type_error() {
    let text = "before :: Unit -> Int32 := () -> 1;\n\
                bad :: Unit -> Bool := () -> 2;\n\
                dependent :: Unit -> Int32 := () -> { bad(); 3 };\n\
                after :: Unit -> Int32 := () -> 4;\n";
    let source = source(text);
    let analysis = mal_frontend::analysis::analyze_for_editor(&source)
        .expect("parse, resolution, and checker prepasses succeed");
    assert_eq!(
        analysis
            .check_diagnostic()
            .expect("the bad top-level item is diagnosed")
            .message,
        "type mismatch"
    );
    let strict_diagnostic = mal_frontend::analysis::analyze(&source)
        .err()
        .expect("strict analysis rejects the document");
    assert_eq!(
        strict_diagnostic.message,
        analysis.check_diagnostic().unwrap().message
    );
    assert!(analysis.specialization_error().is_none());

    let document = mal_frontend::editor::from_editor_analysis_for_file(&analysis, source.id());
    let bad_declaration = document
        .occurrence_at(text.find("bad ::").unwrap())
        .expect("bad declaration remains resolved");
    let bad_reference = document
        .occurrence_at(text.find("bad();").unwrap())
        .expect("dependent reference remains resolved");
    assert_eq!(bad_declaration.id, bad_reference.id);
    assert_eq!(document.references(bad_declaration.id, true).len(), 2);
    assert_eq!(
        document
            .rename_spans(bad_reference.span.start())
            .unwrap()
            .len(),
        2
    );

    assert!(document.hover_at(text.find("1;").unwrap()).is_some());
    assert!(document.hover_at(text.find("2;").unwrap()).is_none());
    assert!(document.hover_at(text.find("3 }").unwrap()).is_none());
    assert!(document.hover_at(text.find("4;").unwrap()).is_some());
}

#[test]
fn rejects_checker_wide_declaration_failures_in_editor_analysis() {
    let source = source("Loop :: Loop;\nafter :: Int32 := 1;\n");
    let diagnostic = mal_frontend::analysis::analyze_for_editor(&source)
        .err()
        .expect("recursive alias is a checker-wide failure");

    assert_eq!(diagnostic.message, "recursive type alias");
}

#[test]
fn graph_recovery_keeps_independent_root_facts_after_a_dependency_error() {
    let root_text = "require \"library.mal\";\nroot :: Unit -> Int32 := () -> 7;\nuseBad :: Unit -> Int32 := () -> bad();\n";
    let library_text =
        "bad :: Unit -> Int32 := () -> false;\npublic :: Unit -> Int32 := () -> 5;\n";
    let root = SourceFile::new(FileId::new(0), "root.mal", root_text.into());
    let library = SourceFile::new(FileId::new(1), "library.mal", library_text.into());
    let graph = SourceGraph::new(
        FileId::new(0),
        vec![root, library],
        vec![
            vec![SourceRequirement {
                target: FileId::new(1),
                span: Span::new(FileId::new(0), 0, 22),
            }],
            vec![],
        ],
        vec![],
    );
    let analysis = mal_frontend::analysis::analyze_graph_for_editor(&graph)
        .expect("graph resolves and recovers from checking");
    assert!(analysis.check_diagnostic().is_some());
    let document =
        mal_frontend::editor::from_graph_editor_analysis(&graph, &analysis, FileId::new(0));

    assert!(document.hover_at(root_text.find('7').unwrap()).is_some());
    let bad_reference = document
        .occurrence_at(root_text.find("bad()").unwrap())
        .expect("dependency reference remains resolved");
    let definition = document
        .definition(bad_reference.id)
        .expect("dependency declaration remains available");
    assert_eq!(definition.span.file(), FileId::new(1));
}
