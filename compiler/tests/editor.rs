use malc::editor::{OccurrenceRole, SymbolKind};
use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(111), "editor-test.mal", text.into())
}

#[test]
fn reports_canonical_types_symbols_and_predefined_completions() {
    let text = "Count :: Int32;\nvalue :: Count := 1;\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    let value_hover = document.hover_at(text.find("value").unwrap()).unwrap();
    assert_eq!(value_hover.ty, "Int32");
    assert_eq!(value_hover.occurrence.unwrap().name, "value");
    let alias_hover = document.hover_at(text.rfind("Count").unwrap()).unwrap();
    assert_eq!(alias_hover.ty, "Int32");
    assert_eq!(alias_hover.occurrence.unwrap().name, "Count");
    assert!(
        document
            .document_symbols()
            .iter()
            .any(|symbol| { symbol.name == "Count" && symbol.kind == SymbolKind::Type })
    );
    assert!(
        document
            .completions()
            .iter()
            .any(|symbol| { symbol.name == "loadInt64" && symbol.kind == SymbolKind::Function })
    );
    assert!(
        document
            .completions()
            .iter()
            .any(|symbol| { symbol.name == "Symbol" && symbol.kind == SymbolKind::Type })
    );
}

#[test]
fn byte_literal_hover_preserves_a_closing_parenthesis_as_literal_content() {
    let text = "closingParen :: UInt8 := ')';";
    let literal = text.find("')'").unwrap();
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let hover = document.hover_at(literal + 1).expect("byte literal hover");

    assert_eq!(hover.ty, "UInt8");
    assert_eq!(&text[hover.span.start()..hover.span.end()], "')'");
    assert!(hover.occurrence.is_none());
}

#[test]
fn storage_size_types_support_hover_and_definition() {
    let text = "Byte :: UInt8;\nsize :: UInt64 := @Byte;";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let reference_offset = text.rfind("Byte").unwrap();
    let declaration_offset = text.find("Byte").unwrap();

    let hover = document.hover_at(reference_offset).expect("type hover");
    assert_eq!(hover.ty, "UInt8");
    assert_eq!(hover.occurrence.unwrap().name, "Byte");

    let reference = document.occurrence_at(reference_offset).unwrap();
    assert_eq!(reference.kind, SymbolKind::Type);
    assert_eq!(reference.role, OccurrenceRole::Reference);
    assert_eq!(
        document.definition(reference.id).unwrap().span.start(),
        declaration_offset
    );
}

#[test]
fn symbol_operators_report_their_result_types() {
    let text =
        "inspect :: Symbol -> UInt64 := \\(value :: Symbol) { #value + UInt64(value # 0); };";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let length_operator = text.find('#').unwrap();
    let access_operator = text.rfind('#').unwrap();

    assert_eq!(document.hover_at(length_operator).unwrap().ty, "UInt64");
    assert_eq!(document.hover_at(access_operator).unwrap().ty, "UInt8");
}

#[test]
fn definition_references_and_rename_follow_capture_identity() {
    let text = "make :: Int32 -> Int32 := \\(x :: Int32) {\n  inner :: Unit -> Int32 := \\<x>() { x; };\n  inner();\n};\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let parameter_offset = text.find("x ::").unwrap();
    let captured_offset = text.find("<x>").unwrap() + 1;
    let inner_reference_offset = text.find("{ x;").unwrap() + 2;

    let parameter = document.occurrence_at(parameter_offset).unwrap();
    assert_eq!(parameter.kind, SymbolKind::Parameter);
    assert_eq!(parameter.role, OccurrenceRole::Declaration);
    assert_eq!(
        document.occurrence_at(captured_offset).unwrap().id,
        parameter.id
    );
    assert_eq!(
        document.occurrence_at(inner_reference_offset).unwrap().id,
        parameter.id
    );
    assert_eq!(
        document.definition(parameter.id).unwrap().span,
        parameter.span
    );
    assert_eq!(document.references(parameter.id, true).len(), 3);
    assert_eq!(
        document.rename_spans(inner_reference_offset).unwrap().len(),
        3
    );
}

#[test]
fn resolved_identity_keeps_shadowed_names_separate() {
    let text = "first :: Int32 -> Int32 := \\(x :: Int32) { x; };\nsecond :: Int32 -> Int32 := \\(x :: Int32) { x; };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let first = document.occurrence_at(text.find("x ::").unwrap()).unwrap();
    let second = document.occurrence_at(text.rfind("x ::").unwrap()).unwrap();

    assert_ne!(first.id, second.id);
    assert_eq!(document.references(first.id, true).len(), 2);
    assert_eq!(document.references(second.id, true).len(), 2);
}

#[test]
fn predefined_references_have_no_source_definition_or_rename_target() {
    let text = "value :: Bool := false;";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let offset = text.find("false").unwrap();
    let occurrence = document.occurrence_at(offset).unwrap();

    assert_eq!(occurrence.role, OccurrenceRole::Reference);
    assert!(document.definition(occurrence.id).is_none());
    assert!(document.rename_spans(offset).is_none());
}

#[test]
fn reports_the_function_type_of_a_first_class_memory_function() {
    let text = "reader :: Ptr -> Int64 := loadInt64;";
    let offset = text.find("loadInt64").unwrap();
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let hover = document.hover_at(offset).expect("memory function hover");

    assert_eq!(hover.ty, "Ptr -> Int64");
    assert_eq!(hover.occurrence.unwrap().name, "loadInt64");
}
