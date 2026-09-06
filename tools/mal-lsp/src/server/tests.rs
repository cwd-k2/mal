use super::*;

#[test]
fn initializes_with_full_sync_utf16_and_formatting() {
    let mut server = Server::new();
    let outcome = server.handle(json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    }));

    let capabilities = &outcome.messages[0]["result"]["capabilities"];
    assert_eq!(capabilities["positionEncoding"], "utf-16");
    assert_eq!(capabilities["textDocumentSync"], 1);
    assert_eq!(capabilities["documentFormattingProvider"], true);
    assert_eq!(capabilities["hoverProvider"], true);
    assert_eq!(capabilities["definitionProvider"], true);
    assert_eq!(
        capabilities["semanticTokensProvider"]["legend"]["tokenTypes"],
        json!(["type", "variable", "parameter", "function"])
    );
}

#[test]
fn publishes_utf16_diagnostics_and_clears_them_after_a_change() {
    let mut server = Server::new();
    let opened = server.handle(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {
            "uri": "file:///unicode.mal", "languageId": "mal", "version": 1, "text": "😀"
        }}
    }));
    assert_eq!(
        opened.messages[0]["params"]["diagnostics"][0]["range"]["end"]["character"],
        2
    );
    assert!(server.documents["file:///unicode.mal"].semantic.is_none());

    let changed = server.handle(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didChange",
        "params": {
            "textDocument": {"uri": "file:///unicode.mal", "version": 2},
            "contentChanges": [{"text": "value :: Int32 := 1;"}]
        }
    }));
    assert_eq!(changed.messages[0]["params"]["version"], 2);
    assert_eq!(changed.messages[0]["params"]["diagnostics"], json!([]));
    assert!(server.documents["file:///unicode.mal"].semantic.is_some());
    request_at(
        &mut server,
        2,
        "textDocument/hover",
        "file:///unicode.mal",
        "value :: Int32 := 1;",
        0,
    );
    assert!(server.documents["file:///unicode.mal"].semantic.is_some());
}

#[test]
fn returns_the_canonical_formatter_edit_for_an_open_document() {
    let mut server = Server::new();
    server.handle(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {"textDocument": {
            "uri": "file:///format.mal", "languageId": "mal", "version": 1,
            "text": "value::Int32:=1;"
        }}
    }));

    let formatted = server.handle(json!({
        "jsonrpc": "2.0", "id": 7, "method": "textDocument/formatting",
        "params": {"textDocument": {"uri": "file:///format.mal"}, "options": {}}
    }));

    assert_eq!(formatted.messages[0]["id"], 7);
    assert_eq!(
        formatted.messages[0]["result"][0]["newText"],
        "value :: Int32 := 1;\n"
    );
}

#[test]
fn exit_succeeds_only_after_shutdown() {
    let mut server = Server::new();
    assert_eq!(
        server
            .handle(json!({"jsonrpc": "2.0", "method": "exit"}))
            .exit,
        Some(false)
    );

    let mut server = Server::new();
    server.handle(json!({"jsonrpc": "2.0", "id": 1, "method": "shutdown"}));
    assert_eq!(
        server
            .handle(json!({"jsonrpc": "2.0", "method": "exit"}))
            .exit,
        Some(true)
    );
}

#[test]
fn serves_hover_navigation_references_and_identity_safe_rename() {
    let text = "make :: Int32 -> Int32 := \\(x :: Int32) {\n  inner :: Unit -> Int32 := \\<x>() { x; };\n  inner();\n};\n";
    let uri = "file:///semantic.mal";
    let mut server = open_document(uri, text);
    let reference = text.find("{ x;").unwrap() + 2;
    let parameter = text.find("x ::").unwrap();

    let hover = request_at(&mut server, 10, "textDocument/hover", uri, text, reference);
    assert_eq!(hover["result"]["contents"]["kind"], "markdown");
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nx :: Int32\n```\n\nparameter"
    );
    assert_eq!(
        hover["result"]["range"]["start"],
        text_position(text, reference)
    );

    let definition = request_at(
        &mut server,
        11,
        "textDocument/definition",
        uri,
        text,
        reference,
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(text, parameter)
    );

    let position = text_position(text, reference);
    let references = server.handle(json!({
            "jsonrpc": "2.0", "id": 12, "method": "textDocument/references",
            "params": {"textDocument": {"uri": uri}, "position": position, "context": {"includeDeclaration": true}}
        }));
    assert_eq!(
        references.messages[0]["result"].as_array().unwrap().len(),
        3
    );

    let rename = server.handle(json!({
        "jsonrpc": "2.0", "id": 13, "method": "textDocument/rename",
        "params": {"textDocument": {"uri": uri}, "position": position, "newName": "renamed"}
    }));
    assert_eq!(
        rename.messages[0]["result"]["changes"][uri]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn serves_typed_hover_for_a_byte_literal_containing_a_closing_parenthesis() {
    let text = "closingParen :: UInt8 := ')';";
    let uri = "file:///byte-hover.mal";
    let mut server = open_document(uri, text);
    let literal = text.find("')'").unwrap();
    let hover = request_at(
        &mut server,
        20,
        "textDocument/hover",
        uri,
        text,
        literal + 1,
    );

    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\n')' :: UInt8\n```"
    );
    assert_eq!(
        hover["result"]["range"],
        json!({
            "start": text_position(text, literal),
            "end": text_position(text, literal + "')'".len())
        })
    );
}

#[test]
fn serves_hover_and_definition_for_a_storage_size_type() {
    let text = "Byte :: UInt8;\nsize :: UInt64 := @Byte;";
    let uri = "file:///storage-size.mal";
    let mut server = open_document(uri, text);
    let reference = text.rfind("Byte").unwrap();
    let declaration = text.find("Byte").unwrap();

    let hover = request_at(&mut server, 21, "textDocument/hover", uri, text, reference);
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nByte :: UInt8\n```\n\ntype"
    );

    let definition = request_at(
        &mut server,
        22,
        "textDocument/definition",
        uri,
        text,
        reference,
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(text, declaration)
    );
}

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
            .any(|item| item["label"] == "loadInt64" && item["kind"] == 3)
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

fn open_document(uri: &str, text: &str) -> Server {
    let mut server = Server::new();
    server.handle(json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "mal", "version": 1, "text": text}}
    }));
    server
}

fn request_at(
    server: &mut Server,
    id: i64,
    method: &str,
    uri: &str,
    text: &str,
    offset: usize,
) -> Value {
    server
        .handle(json!({
            "jsonrpc": "2.0", "id": id, "method": method,
            "params": {"textDocument": {"uri": uri}, "position": text_position(text, offset)}
        }))
        .messages
        .remove(0)
}

fn text_position(text: &str, offset: usize) -> Value {
    let prefix = &text[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix.rsplit('\n').next().unwrap().encode_utf16().count();
    json!({"line": line, "character": character})
}
