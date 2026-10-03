use super::*;

#[test]
fn associates_only_adjacent_standalone_comments_with_declarations() {
    let text = "// First line\n// Second line  \nextern output :: Int32 -> Unit;\n\n// Detached\n\nvalue :: Int32 := 1; // Trailing\nnext :: Int32 := 2;\n";
    let source = source(text);
    let document = mal_frontend::editor::analyze(&source).expect("semantic document");
    let documentation = |name: &str| {
        let occurrence = document
            .occurrence_at(text.find(name).unwrap())
            .expect("declaration occurrence");
        mal_frontend::editor::declaration_documentation(
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
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

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
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

    let tree = document.hover_at(text.find("Tree").unwrap()).unwrap();
    assert_eq!(tree.ty, "(Int64, Address, Address)");
    let forest = document.hover_at(text.find("Forest").unwrap()).unwrap();
    assert_eq!(forest.ty, "(Tree, Tree)");
}

#[test]
fn function_and_parameter_hovers_preserve_declared_aliases() {
    let text = "Tree :: (Int64, Address, Address);\nf :: (Tree, Int64) -> Int64 := (tree, n) -> n;\nHandler :: Tree -> Int64;\ng :: Handler := (tree) -> 0;\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

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
fn inferred_call_results_and_callback_parameters_preserve_specialized_aliases() {
    let text = "View<A> :: (Buffer<A>, USize);\n\
                MaybeView<A> :: [Unit, View<A>];\n\
                view<A> :: Buffer<A> -> View<A> := (buffer) -> (buffer, 0usize);\n\
                choose<A> :: View<A> -> MaybeView<A> := (source) -> [none, some] => some(source);\n\
                inspect<A> :: (View<A>, View<A> -> USize) -> USize := (source, callback) -> callback(source);\n\
                main :: Unit -> Int32 := () -> {\n\
                    writable := view<Int32>(make<Int32>(0usize));\n\
                    inspected := inspect<Int32>(writable, (current) -> 0usize);\n\
                    selected := choose<Int32>(writable);\n\
                    selected[() -> 0usize, (selectedView) -> 0usize];\n\
                    0i32\n\
                };\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

    let writable = document
        .hover_at(text.find("writable :=").unwrap())
        .unwrap();
    assert_eq!(writable.ty, "View<Int32>");
    let current = document.hover_at(text.find("current").unwrap()).unwrap();
    assert_eq!(current.ty, "View<Int32>");
    let selected_view = document
        .hover_at(text.find("selectedView").unwrap())
        .unwrap();
    assert_eq!(selected_view.ty, "View<Int32>");
}

#[test]
fn nested_opaque_carriers_keep_the_nominal_type_without_exposing_representation() {
    let text = "opaque Focus<A> :: (Buffer<A>, USize);\n\
                inspect :: Focus<Focus<Int32>> -> USize := (nested) -> 0usize;\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

    let declaration = document.hover_at(text.find("Focus").unwrap()).unwrap();
    assert_eq!(declaration.ty, "Focus");

    let parameter = document.hover_at(text.find("nested").unwrap()).unwrap();
    assert_eq!(parameter.ty, "Focus<Focus<Int32>>");
    assert!(!parameter.ty.contains("Buffer"));
}

#[test]
fn external_function_references_share_the_declaration_identity() {
    let text = "extern output :: UInt8 -> Unit;\nrun :: Unit -> Unit := () -> { selected := output; selected(1u8) };\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
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
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
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
fn indexes_buffer_intrinsics_and_types_as_predefined_symbols() {
    let text = "build :: Unit -> Buffer<Int32> := () -> make<Int32>(1usize);\n\
                admit :: Address -> Buffer<Int32> := (address) -> from<Int32>(address, 0usize, 1usize);\n\
                publish :: (Buffer<Int32>, Address) -> Unit := (values, address) -> values.into(address, 0usize, #values);\n";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");

    for (name, offset) in [
        ("from", text.find("from<Int32>").unwrap()),
        ("into", text.find("into(address").unwrap()),
    ] {
        let occurrence = document
            .occurrence_at(offset)
            .expect("intrinsic occurrence");
        assert_eq!(occurrence.name, name);
        assert_eq!(occurrence.kind, SymbolKind::Function);
        assert!(document.hover_at(offset).is_some());
    }
    for name in [
        "Buffer", "from", "into", "make", "new", "get", "put", "fill", "copy",
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
            .is_some_and(|documentation| documentation.contains("initial capacity"))
    );
}

#[test]
fn byte_literal_hover_preserves_a_closing_parenthesis_as_literal_content() {
    let text = "closingParen :: UInt8 := ')';";
    let literal = text.find("')'").unwrap();
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    let hover = document.hover_at(literal + 1).expect("byte literal hover");

    assert_eq!(hover.ty, "UInt8");
    assert_eq!(&text[hover.span.start()..hover.span.end()], "')'");
    assert!(hover.occurrence.is_none());
}

#[test]
fn numeric_conversion_suffixes_have_value_hover_without_type_navigation() {
    let text = "value :: UInt8 := 1i8.u8;";
    let document = mal_frontend::editor::analyze(&source(text)).expect("semantic document");
    let suffix_offset = text.rfind("u8").unwrap();

    let hover = document.hover_at(suffix_offset).expect("conversion hover");
    assert_eq!(hover.ty, "UInt8");
    assert!(hover.occurrence.is_none());
    assert!(document.occurrence_at(suffix_offset).is_none());
}
