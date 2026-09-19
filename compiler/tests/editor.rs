use malc::editor::{OccurrenceRole, SymbolKind};
use malc::source::{FileId, SourceFile, SourceGraph, SourceRequirement, Span};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(111), "editor-test.mal", text.into())
}

#[test]
fn associates_only_adjacent_standalone_comments_with_declarations() {
    let text = "// First line\n// Second line  \nextern output :: Int32 -> Unit;\n\n// Detached\n\nvalue :: Int32 := 1; // Trailing\nnext :: Int32 := 2;\n";
    let source = source(text);
    let document = malc::editor::analyze(&source).expect("semantic document");
    let documentation = |name: &str| {
        let occurrence = document
            .occurrence_at(text.find(name).unwrap())
            .expect("declaration occurrence");
        malc::editor::declaration_documentation(
            &source,
            occurrence.declaration_span.expect("declaration span"),
        )
    };

    assert_eq!(
        documentation("output").as_deref(),
        Some("First line\nSecond line")
    );
    assert_eq!(documentation("value"), None);
    assert_eq!(documentation("next"), None);
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
    let text = "Tree :: (Int64, Address, Address);\nForest :: (Tree, Tree);\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    let tree = document.hover_at(text.find("Tree").unwrap()).unwrap();
    assert_eq!(tree.ty, "(Int64, Address, Address)");
    let forest = document.hover_at(text.find("Forest").unwrap()).unwrap();
    assert_eq!(forest.ty, "(Tree, Tree)");
}

#[test]
fn function_and_parameter_hovers_preserve_declared_aliases() {
    let text = "Tree :: (Int64, Address, Address);\nf :: (Tree, Int64) -> Int64 := (tree, n) -> n;\nHandler :: Tree -> Int64;\ng :: Handler := (tree) -> 0;\n";
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
    let text = "extern output :: UInt8 -> Unit;\nrun :: Unit -> Unit := () -> { selected := output; selected(1u8) };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("output").unwrap();
    let reference_offset = text.rfind("output").unwrap();
    let declaration = document.occurrence_at(declaration_offset).unwrap();
    let reference = document.occurrence_at(reference_offset).unwrap();

    assert_eq!(declaration.id, reference.id);
    assert_eq!(reference.kind, SymbolKind::Function);
    assert_eq!(reference.detail.as_deref(), Some("UInt8 -> Unit"));
    assert_eq!(
        document.definition(reference.id).unwrap().span,
        declaration.span
    );
}

#[test]
fn receiver_first_callees_support_function_editor_features() {
    let text = "add :: (Int32, Int32) -> Int32 := (left, right) -> { left + right };\n\
                main :: Unit -> Int32 := () -> { 40i32.add(2) };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("add ::").unwrap();
    let reference_offset = text.rfind(".add(").unwrap() + 1;
    let reference = document
        .occurrence_at(reference_offset)
        .expect("receiver-first callee reference");

    assert_eq!(reference.kind, SymbolKind::Function);
    assert_eq!(reference.role, OccurrenceRole::Reference);
    assert_eq!(reference.detail.as_deref(), Some("(Int32, Int32) -> Int32"));
    assert_eq!(
        document.hover_at(reference_offset).unwrap().span,
        reference.span
    );
    assert_eq!(
        document.definition(reference.id).unwrap().span.start(),
        declaration_offset
    );
    assert_eq!(document.references(reference.id, true).len(), 2);
    assert_eq!(document.rename_spans(reference_offset).unwrap().len(), 2);
}

#[test]
fn indexes_packed_intrinsics_and_indexed_types_as_predefined_symbols() {
    let text = "build :: Unit -> Packed<Int32> := () -> make<Int32>(1usize, (buffer) -> { buffer.new(1i32); () });\n\
                revise :: Packed<Int32> -> Packed<Int32> := (source) -> source.edit<Int32>((_) -> ());\n\
                admit :: Address -> Packed<Int32> := (address) -> address.pack<Int32>(0usize, 1usize);\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");

    for (name, offset) in [
        ("pack", text.find("pack<Int32>").unwrap()),
        ("edit", text.find("edit<Int32>").unwrap()),
    ] {
        let occurrence = document
            .occurrence_at(offset)
            .expect("intrinsic occurrence");
        assert_eq!(occurrence.name, name);
        assert_eq!(occurrence.kind, SymbolKind::Function);
        assert!(document.hover_at(offset).is_some());
    }
    for name in [
        "Region", "Packed", "Buffer", "pack", "make", "edit", "view", "new", "get", "put", "set",
    ] {
        assert!(
            document
                .completions()
                .iter()
                .any(|symbol| symbol.name == name),
            "missing `{name}` completion"
        );
    }

    let make = document
        .occurrence_at(text.find("make<Int32>").unwrap())
        .expect("make occurrence");
    assert!(
        make.documentation
            .as_deref()
            .is_some_and(|documentation| documentation.contains("initial allocation"))
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
fn numeric_conversion_suffixes_have_value_hover_without_type_navigation() {
    let text = "value :: UInt8 := 1i8.u8;";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let suffix_offset = text.rfind("u8").unwrap();

    let hover = document.hover_at(suffix_offset).expect("conversion hover");
    assert_eq!(hover.ty, "UInt8");
    assert!(hover.occurrence.is_none());
    assert!(document.occurrence_at(suffix_offset).is_none());
}

#[test]
fn sum_result_annotations_navigate_to_the_alias() {
    let text = "Payload :: Int32;\nChoice :: [Unit, Payload];\ncreate :: Payload -> Choice := (value) -> [none, some] => { some(value) };\nread :: Unit -> Choice := () -> { create(1) };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let declaration_offset = text.find("Choice").unwrap();
    let constructor_offset = text.find("-> Choice").unwrap() + 3;
    let reference = document
        .occurrence_at(constructor_offset)
        .expect("result type reference");

    assert_eq!(reference.kind, SymbolKind::Type);
    assert_eq!(reference.role, OccurrenceRole::Reference);
    assert_eq!(
        document.definition(reference.id).unwrap().span.start(),
        declaration_offset
    );
    assert_eq!(document.references(reference.id, true).len(), 3);
    assert_eq!(document.rename_spans(constructor_offset).unwrap().len(), 3);
    assert_eq!(
        document.hover_at(constructor_offset).unwrap().ty,
        "[Unit, Payload]"
    );
    assert_eq!(
        document
            .hover_at(text.find("create ::").unwrap())
            .unwrap()
            .ty,
        "Payload -> Choice"
    );
    assert_eq!(
        document
            .hover_at(text.rfind("create(1)").unwrap() + "create".len())
            .unwrap()
            .ty,
        "[Unit, Int32]"
    );
}

#[test]
fn sum_continuation_parameters_keep_declaration_identity() {
    let text = "Choice :: [Unit, Int32];\nread :: Choice -> Int32 := (choice) -> { choice[\n() -> { 0 },\n(payload) -> { payload }\n] };\n";
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
fn result_binders_support_hover_definition_references_and_rename() {
    let text = "Payload :: Int32;\nResult :: [Payload, Symbol];\ncompute :: Bool -> Result := (enabled) -> [ok, err] => { when (enabled) { ok(42) }; err(\"disabled\") };\nfinish :: Result -> Result := (result) -> [return] => { return(result) };\n";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let ok_declaration_offset = text.find("[ok").unwrap() + 1;
    let ok_reference_offset = text.rfind("ok(42)").unwrap();
    let err_declaration_offset = text.find("err]").unwrap();
    let err_reference_offset = text.rfind("err(\"").unwrap();

    let ok = document.occurrence_at(ok_declaration_offset).unwrap();
    assert_eq!(ok.kind, SymbolKind::Parameter);
    assert_eq!(ok.role, OccurrenceRole::Declaration);
    assert_eq!(
        document.hover_at(ok_declaration_offset).unwrap().ty,
        "Payload"
    );
    assert_eq!(
        document.occurrence_at(ok_reference_offset).unwrap().id,
        ok.id
    );
    assert_eq!(document.definition(ok.id).unwrap().span, ok.span);
    assert_eq!(document.references(ok.id, true).len(), 2);
    assert_eq!(document.rename_spans(ok_reference_offset).unwrap().len(), 2);

    let err = document.occurrence_at(err_declaration_offset).unwrap();
    assert_eq!(err.kind, SymbolKind::Parameter);
    assert_eq!(
        document.hover_at(err_reference_offset).unwrap().ty,
        "Symbol"
    );
    assert_eq!(
        document.occurrence_at(err_reference_offset).unwrap().id,
        err.id
    );

    let return_declaration_offset = text.find("[return]").unwrap() + 1;
    let return_reference_offset = text.rfind("return(result)").unwrap();
    let result_binder = document.occurrence_at(return_declaration_offset).unwrap();
    assert_eq!(result_binder.kind, SymbolKind::Parameter);
    assert_eq!(
        document.hover_at(return_reference_offset).unwrap().ty,
        "Result"
    );
    assert_eq!(
        document.occurrence_at(return_reference_offset).unwrap().id,
        result_binder.id
    );
}

#[test]
fn symbol_operators_report_their_result_types() {
    let text = "inspect :: Symbol -> USize := (value) -> { #value + (value # 0usize).usize; };";
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let length_operator = text.find('#').unwrap();
    let access_operator = text.rfind('#').unwrap();

    assert_eq!(document.hover_at(length_operator).unwrap().ty, "USize");
    assert_eq!(document.hover_at(access_operator).unwrap().ty, "UInt8");
}

#[test]
fn definition_references_and_rename_follow_capture_identity() {
    let text = "create :: Int32 -> Int32 := (x) -> {\n  inner :: Unit -> Int32 := () -> { x; };\n  inner();\n};\n";
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
    let text =
        "first :: Int32 -> Int32 := (x) -> { x; };\nsecond :: Int32 -> Int32 := (x) -> { x; };\n";
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
fn reports_the_type_of_a_region_method() {
    let text = "read :: Region<Int64> -> Int64 := (region) -> region.get(0usize);";
    let offset = text.rfind("get").unwrap();
    let document = malc::editor::analyze(&source(text)).expect("semantic document");
    let hover = document.hover_at(offset).expect("region get hover");

    assert_eq!(hover.ty, "(Buffer<T>, USize) -> T");
    assert_eq!(hover.occurrence.unwrap().name, "get");
    assert!(
        hover
            .occurrence
            .unwrap()
            .documentation
            .as_deref()
            .is_some_and(|documentation| documentation.contains("Region<T>"))
    );
}

#[test]
fn graph_analysis_keeps_navigation_global_and_document_features_local() {
    let root_text =
        "require \"library.mal\";\nanswer :: Unit -> Int32 := () -> { publicValue; };\n";
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

#[test]
fn indexes_long_left_associative_expressions_without_host_recursion() {
    let expression = std::iter::repeat_n("0i32", 4_096)
        .collect::<Vec<_>>()
        .join(" + ");
    let text = format!("main :: Unit -> Int32 := () -> {{ {expression}; }};");

    let document = malc::editor::analyze(&source(&text)).expect("semantic document");

    assert_eq!(
        document.hover_at(text.rfind("0i32").unwrap()).unwrap().ty,
        "Int32"
    );
}

#[test]
fn indexes_many_top_level_symbols_from_declarations_once() {
    let text = (0..4_096)
        .map(|index| format!("value{index} :: Int32 := 0i32;\n"))
        .collect::<String>();

    let document = malc::editor::analyze(&source(&text)).expect("semantic document");

    assert_eq!(document.document_symbols().len(), 4_096);
}
