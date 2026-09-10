use malc::editor::{OccurrenceRole, SymbolKind};
use malc::source::{FileId, SourceFile, SourceGraph, SourceRequirement, Span};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(111), "editor-test.mal", text.into())
}

#[test]
fn preserves_declared_aliases_in_symbol_types() {
    let text = "Count :: Int32;\nvalue :: Count := 1;\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    let value_hover = document.hover_at(text.find("value").unwrap()).unwrap();
    assert_eq!(value_hover.ty, "Count");
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
            .any(|symbol| { symbol.name == "false" && symbol.kind == SymbolKind::Value })
    );
    assert!(
        document
            .completions()
            .iter()
            .any(|symbol| { symbol.name == "Symbol" && symbol.kind == SymbolKind::Type })
    );
}

#[test]
fn expands_only_the_hovered_alias() {
    let text = "Tree :: (Int64, Ptr, Ptr);\nForest :: (Tree, Tree);\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    let tree = document.hover_at(text.find("Tree").unwrap()).unwrap();
    assert_eq!(tree.ty, "(Int64, Ptr, Ptr)");
    let forest = document.hover_at(text.find("Forest").unwrap()).unwrap();
    assert_eq!(forest.ty, "(Tree, Tree)");
}

#[test]
fn function_and_parameter_hovers_preserve_declared_aliases() {
    let text = "Tree :: (Int64, Ptr, Ptr);\nf :: (Tree, Int64) -> Int64 := (tree, n) { n; };\nHandler :: Tree -> Int64;\ng :: Handler := (tree) { 0; };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    let function = document.hover_at(text.find("f ::").unwrap()).unwrap();
    assert_eq!(function.ty, "(Tree, Int64) -> Int64");
    assert_eq!(function.occurrence.unwrap().kind, SymbolKind::Function);
    let tree_parameter = document.hover_at(text.find("tree").unwrap()).unwrap();
    assert_eq!(tree_parameter.ty, "Tree");
    let aliased_function = document.hover_at(text.find("g ::").unwrap()).unwrap();
    assert_eq!(aliased_function.ty, "Handler");
    assert_eq!(
        aliased_function.occurrence.unwrap().kind,
        SymbolKind::Function
    );
    let aliased_parameter = document.hover_at(text.rfind("tree").unwrap()).unwrap();
    assert_eq!(aliased_parameter.ty, "Tree");
}

#[test]
fn external_function_references_share_the_declaration_identity() {
    let text = "extern output :: Symbol -> Unit;\nmain :: Unit -> Unit := () { selected := output; selected(\"x\") };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("output").unwrap();
    let reference_offset = text.rfind("output").unwrap();
    let declaration = document.occurrence_at(declaration_offset).unwrap();
    let reference = document.occurrence_at(reference_offset).unwrap();

    assert_eq!(declaration.id, reference.id);
    assert_eq!(reference.kind, SymbolKind::Function);
    assert_eq!(reference.detail.as_deref(), Some("Symbol -> Unit"));
    assert_eq!(
        document.definition(reference.id).unwrap().span,
        declaration.span
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
fn type_qualified_primitives_support_type_hover_and_definition() {
    let text = "Byte :: UInt8;\nsize :: UInt64 := Byte.size;";
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
fn sum_constructor_type_references_navigate_to_the_alias() {
    let text = "Payload :: Int32;\nChoice :: [Unit, Payload];\nmake :: Payload -> Choice := 1[Choice];\nread :: Unit -> Choice := () { make(1) };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("Choice").unwrap();
    let constructor_offset = text.find("1[Choice]").unwrap() + 2;
    let reference = document
        .occurrence_at(constructor_offset)
        .expect("constructor type reference");

    assert_eq!(reference.kind, SymbolKind::Type);
    assert_eq!(reference.role, OccurrenceRole::Reference);
    assert_eq!(
        document.definition(reference.id).unwrap().span.start(),
        declaration_offset
    );
    assert_eq!(document.references(reference.id, true).len(), 4);
    assert_eq!(document.rename_spans(constructor_offset).unwrap().len(), 4);
    assert_eq!(
        document.hover_at(constructor_offset).unwrap().ty,
        "[Unit, Payload]"
    );
    assert_eq!(
        document.hover_at(text.find("make ::").unwrap()).unwrap().ty,
        "Payload -> Choice"
    );
    assert_eq!(
        document
            .hover_at(text.rfind("make(1)").unwrap() + 4)
            .unwrap()
            .ty,
        "[Unit, Int32]"
    );
}

#[test]
fn sum_continuation_parameters_keep_declaration_identity() {
    let text = "Choice :: [Unit, Int32];\nread :: Choice -> Int32 := (choice) { choice[\n() { 0 },\n(payload) { payload }\n] };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("(payload)").unwrap() + 1;
    let reference_offset = text.rfind("payload").unwrap();
    let declaration = document
        .occurrence_at(declaration_offset)
        .expect("continuation parameter");
    let reference = document
        .occurrence_at(reference_offset)
        .expect("continuation parameter reference");

    assert_eq!(declaration.kind, SymbolKind::Parameter);
    assert_eq!(declaration.role, OccurrenceRole::Declaration);
    assert_eq!(reference.id, declaration.id);
    assert_eq!(document.references(declaration.id, true).len(), 2);
    assert_eq!(document.rename_spans(reference_offset).unwrap().len(), 2);
}

#[test]
fn symbol_operators_report_their_result_types() {
    let text = "inspect :: Symbol -> UInt64 := (value) { #value + UInt64(value # 0); };";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let length_operator = text.find('#').unwrap();
    let access_operator = text.rfind('#').unwrap();

    assert_eq!(document.hover_at(length_operator).unwrap().ty, "UInt64");
    assert_eq!(document.hover_at(access_operator).unwrap().ty, "UInt8");
}

#[test]
fn definition_references_and_rename_follow_capture_identity() {
    let text =
        "make :: Int32 -> Int32 := (x) {\n  inner :: Unit -> Int32 := () { x; };\n  inner();\n};\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let parameter_offset = text.find("(x)").unwrap() + 1;
    let inner_reference_offset = text.find("{ x;").unwrap() + 2;

    let parameter = document.occurrence_at(parameter_offset).unwrap();
    assert_eq!(parameter.kind, SymbolKind::Parameter);
    assert_eq!(parameter.role, OccurrenceRole::Declaration);
    assert_eq!(
        document.occurrence_at(inner_reference_offset).unwrap().id,
        parameter.id
    );
    assert_eq!(
        document.definition(parameter.id).unwrap().span,
        parameter.span
    );
    assert_eq!(document.references(parameter.id, true).len(), 2);
    assert_eq!(
        document.rename_spans(inner_reference_offset).unwrap().len(),
        2
    );
}

#[test]
fn resolved_identity_keeps_shadowed_names_separate() {
    let text = "first :: Int32 -> Int32 := (x) { x; };\nsecond :: Int32 -> Int32 := (x) { x; };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let first = document
        .occurrence_at(text.find("(x)").unwrap() + 1)
        .unwrap();
    let second = document
        .occurrence_at(text.rfind("(x)").unwrap() + 1)
        .unwrap();

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
    let text = "reader :: Ptr -> Int64 := Int64.load;";
    let offset = text.find("load").unwrap();
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let hover = document.hover_at(offset).expect("memory function hover");

    assert_eq!(hover.ty, "Ptr -> Int64");
    assert!(hover.occurrence.is_none());
}

#[test]
fn graph_analysis_keeps_navigation_global_and_document_features_local() {
    let root_text = "require \"library.mal\";\nanswer :: Unit -> Int32 := () { publicValue; };\n";
    let library_text = "publicValue :: Int32 := 42;\n_privateValue :: Int32 := 7;\n";
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
    let analysis = malc::pipeline::analyze_graph(&graph).expect("graph analysis");
    let document = malc::editor::from_graph_analysis(&graph, &analysis, FileId::new(0));

    assert_eq!(
        document
            .document_symbols()
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>(),
        ["answer"]
    );
    assert!(
        document
            .completions()
            .iter()
            .any(|symbol| symbol.name == "publicValue")
    );
    assert!(
        !document
            .completions()
            .iter()
            .any(|symbol| symbol.name == "_privateValue")
    );

    let reference = document
        .occurrence_at(root_text.rfind("publicValue").unwrap())
        .expect("root reference");
    let definition = document
        .definition(reference.id)
        .expect("library definition");
    assert_eq!(definition.span.file(), FileId::new(1));
    assert_eq!(document.references(reference.id, true).len(), 2);
    assert!(
        document
            .document_occurrences()
            .all(|occurrence| occurrence.span.file() == FileId::new(0))
    );
}
