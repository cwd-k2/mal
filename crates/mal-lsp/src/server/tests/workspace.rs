use super::*;

#[test]
fn hover_includes_dependency_documentation_and_relative_definition_location() {
    let files = TestFiles::new();
    let root_text =
        "require \"library.mal\";\nanswer :: Unit -> Int32 := () -> { publicValue; };\n";
    let library_text = "// Public answer.\n// Safe to reuse.\npublicValue :: Int32 := 42;\n";
    let root_path = files.write("program.mal", root_text);
    files.write("library.mal", library_text);
    let root_uri = path_to_uri(&root_path);
    let mut server = open_document(&root_uri, root_text);
    let reference = root_text.rfind("publicValue").unwrap();

    let hover = request_at(
        &mut server,
        29,
        "textDocument/hover",
        &root_uri,
        root_text,
        reference,
    );

    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\npublicValue :: Int32\n```\n\nvalue\n\nPublic answer.\nSafe to reuse.\n\nDefined in `library.mal:3:1`"
    );
}

#[test]
fn serves_cross_file_semantics_from_open_dependency_buffers() {
    let files = TestFiles::new();
    let root_path = files.write("program.mal", "not the open buffer");
    let library_path = files.write("library.mal", "diskValue :: Int32 := 0;");
    let root_uri = path_to_uri(&root_path);
    let library_uri = path_to_uri(&library_path);
    let root_text =
        "require \"library.mal\";\nanswer :: Unit -> Int32 := () -> { publicValue; };\n";
    let library_text = "publicValue :: Int32 := 42;\n_privateValue :: Int32 := 7;\n";
    let mut server = Server::new();
    let root_opened = server.handle(did_open(&root_uri, root_text));
    assert!(
        !root_opened.messages[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let library_opened = server.handle(did_open(&library_uri, library_text));
    assert_eq!(library_opened.messages.len(), 2);
    assert_eq!(library_opened.messages[1]["params"]["uri"], root_uri);
    assert_eq!(
        library_opened.messages[1]["params"]["diagnostics"],
        json!([])
    );

    let reference = root_text.rfind("publicValue").unwrap();
    let definition = request_at(
        &mut server,
        30,
        "textDocument/definition",
        &root_uri,
        root_text,
        reference,
    );
    assert_eq!(definition["result"]["uri"], library_uri);
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(library_text, 0)
    );

    let references = server.handle(json!({
        "jsonrpc": "2.0", "id": 31, "method": "textDocument/references",
        "params": {
            "textDocument": {"uri": root_uri},
            "position": text_position(root_text, reference),
            "context": {"includeDeclaration": true}
        }
    }));
    let locations = references.messages[0]["result"].as_array().unwrap();
    assert_eq!(locations.len(), 2);
    assert!(
        locations
            .iter()
            .any(|location| location["uri"] == library_uri)
    );
    assert!(locations.iter().any(|location| location["uri"] == root_uri));

    let rename = server.handle(json!({
        "jsonrpc": "2.0", "id": 32, "method": "textDocument/rename",
        "params": {
            "textDocument": {"uri": root_uri},
            "position": text_position(root_text, reference),
            "newName": "renamed"
        }
    }));
    assert_eq!(
        rename.messages[0]["result"]["changes"][&root_uri]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        rename.messages[0]["result"]["changes"][&library_uri]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let completion = request_at(
        &mut server,
        33,
        "textDocument/completion",
        &root_uri,
        root_text,
        root_text.len(),
    );
    let labels = completion["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"publicValue"));
    assert!(!labels.contains(&"_privateValue"));

    let symbols = server.handle(json!({
        "jsonrpc": "2.0", "id": 34, "method": "textDocument/documentSymbol",
        "params": {"textDocument": {"uri": root_uri}}
    }));
    assert_eq!(symbols.messages[0]["result"].as_array().unwrap().len(), 1);
    assert_eq!(symbols.messages[0]["result"][0]["name"], "answer");

    let tokens = server.handle(json!({
        "jsonrpc": "2.0", "id": 35, "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": {"uri": root_uri}}
    }));
    assert!(
        !tokens.messages[0]["result"]["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn does_not_republish_unchanged_diagnostics_for_open_dependents() {
    let files = TestFiles::new();
    let root_path = files.write(
        "program.mal",
        "require \"library.mal\";\nanswer :: Int32 := publicValue;\n",
    );
    let library_path = files.write("library.mal", "publicValue :: Int32 := 1;\n");
    let root_uri = path_to_uri(&root_path);
    let library_uri = path_to_uri(&library_path);
    let mut server = Server::new();
    server.handle(did_open(
        &root_uri,
        "require \"library.mal\";\nanswer :: Int32 := publicValue;\n",
    ));
    server.handle(did_open(&library_uri, "publicValue :: Int32 := 1;\n"));

    let changed = server.handle(did_change(&library_uri, 2, "publicValue :: Int32 := 2;\n"));

    assert_eq!(changed.messages.len(), 1);
    assert_eq!(changed.messages[0]["params"]["uri"], library_uri);
    assert_eq!(changed.messages[0]["params"]["version"], 2);
    assert_eq!(changed.messages[0]["params"]["diagnostics"], json!([]));
}
