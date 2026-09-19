use super::*;

#[test]
fn serves_symbols_completion_and_semantic_tokens() {
    let text = "Count :: Int32;\nvalue :: Count := 1;\n";
    let uri = "file:///symbols.mal";
    let mut server = open_document(uri, text);

    let symbols = server.handle(json!({
        "jsonrpc": "2.0", "id": 20, "method": "textDocument/documentSymbol",
        "params": {"textDocument": {"uri": uri}}
    }));
    assert_eq!(symbols.messages[0]["result"].as_array().unwrap().len(), 2);

    let completion = request_at(
        &mut server,
        21,
        "textDocument/completion",
        uri,
        text,
        text.len(),
    );
    assert!(
        completion["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "false" && item["kind"] == 6)
    );
    let tokens = server.handle(json!({
        "jsonrpc": "2.0", "id": 22, "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": {"uri": uri}}
    }));
    let data = tokens.messages[0]["result"]["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(data.len() % 5, 0);
    assert!(data.chunks(5).any(|token| token[3] == 0 && token[4] == 1));
}

#[test]
fn serves_packed_intrinsics_as_functions_and_indexed_type_completions() {
    let text = "build :: Unit -> Packed<Int32> := () -> make<Int32>(1usize, (buffer) -> { buffer.new(1i32); () });\n";
    let uri = "file:///packed-editor.mal";
    let mut server = open_document(uri, text);
    let completion = request_at(
        &mut server,
        25,
        "textDocument/completion",
        uri,
        text,
        text.len(),
    );
    let items = completion["result"].as_array().unwrap();

    for name in ["Region", "Packed", "Buffer"] {
        assert!(
            items
                .iter()
                .any(|item| item["label"] == name && item["kind"] == 7)
        );
    }
    assert!(items.iter().any(|item| {
        item["label"] == "pack"
            && item["kind"] == 3
            && item["documentation"]["value"]
                .as_str()
                .is_some_and(|documentation| documentation.contains("half-open element range"))
    }));
    for name in ["make", "edit", "view", "new", "get", "put", "set"] {
        assert!(
            items
                .iter()
                .any(|item| item["label"] == name && item["kind"] == 3)
        );
    }

    let tokens = server.handle(json!({
        "jsonrpc": "2.0", "id": 26, "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": {"uri": uri}}
    }));
    let data = tokens.messages[0]["result"]["data"].as_array().unwrap();
    assert!(data.chunks(5).any(|token| token[2] == 4 && token[3] == 3));
}

#[test]
fn completes_lexical_functions_after_an_incomplete_receiver_suffix() {
    let text = "transform :: (Int32, Int32) -> Int32 := (value, option) -> value + option;\n\
                count :: Int32 := 1;\n\
                main :: Unit -> Int32 := () -> count.;\n";
    let uri = "file:///receiver-completion.mal";
    let mut server = open_document(uri, text);
    let completion = request_at(
        &mut server,
        23,
        "textDocument/completion",
        uri,
        text,
        text.rfind('.').unwrap() + 1,
    );
    let items = completion["result"].as_array().unwrap();

    assert!(items.iter().any(|item| item["label"] == "transform"));
    assert!(!items.iter().any(|item| item["label"] == "count"));
}

#[test]
fn limits_receiver_completion_to_functions_in_a_valid_document() {
    let text = "transform :: (Int32, Int32) -> Int32 := (value, option) -> value + option;\n\
                count :: Int32 := 1;\n\
                main :: Unit -> Int32 := () -> count.transform(1);\n";
    let uri = "file:///valid-receiver-completion.mal";
    let mut server = open_document(uri, text);
    let completion = request_at(
        &mut server,
        24,
        "textDocument/completion",
        uri,
        text,
        text.rfind(".transform").unwrap() + 1,
    );
    let items = completion["result"].as_array().unwrap();

    assert!(items.iter().any(|item| item["label"] == "transform"));
    assert!(!items.iter().any(|item| item["label"] == "count"));
    assert!(!items.iter().any(|item| item["label"] == "false"));
}

#[test]
fn completes_visible_dependency_functions_from_an_invalid_root() {
    let files = TestFiles::new();
    let root_text = "require \"library.mal\";\n\
                     local :: Int32 -> Int32 := (value) -> value;\n\
                     count :: Int32 := 1;\n\
                     main :: Unit -> Int32 := () -> count.par;\n";
    let root_path = files.write("program.mal", root_text);
    files.write(
        "library.mal",
        "publicFunction :: Int32 -> Int32 := (value) -> value;\n\
         _privateFunction :: Int32 -> Int32 := (value) -> value;\n",
    );
    let root_uri = path_to_uri(&root_path);
    let mut server = open_document(&root_uri, root_text);
    let completion = request_at(
        &mut server,
        24,
        "textDocument/completion",
        &root_uri,
        root_text,
        root_text.rfind(".par").unwrap() + 4,
    );
    let labels = completion["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(labels.contains(&"local"));
    assert!(labels.contains(&"publicFunction"), "{labels:?}");
    assert!(!labels.contains(&"_privateFunction"));
    assert!(!labels.contains(&"count"));
}
