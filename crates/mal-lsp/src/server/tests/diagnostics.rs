use super::super::*;
use super::*;

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
    assert!(!server.documents["file:///unicode.mal"].has_semantic());
    assert!(!server.documents["file:///unicode.mal"].has_analysis());

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
    assert!(!server.documents["file:///unicode.mal"].has_semantic());
    assert!(server.documents["file:///unicode.mal"].has_analysis());
    request_at(
        &mut server,
        2,
        "textDocument/hover",
        "file:///unicode.mal",
        "value :: Int32 := 1;",
        0,
    );
    assert!(server.documents["file:///unicode.mal"].has_semantic());
}

#[test]
fn publishes_diagnostics_once_per_document_version() {
    let uri = "file:///diagnostic-version.mal";
    let invalid = "value :: Int32 := ;";
    let mut server = Server::new();
    let opened = server.handle(did_open(uri, invalid));
    assert_eq!(opened.messages[0]["params"]["version"], 1);

    let changed = server.handle(did_change(uri, 2, invalid));
    assert_eq!(changed.messages.len(), 1);
    assert_eq!(changed.messages[0]["params"]["version"], 2);

    let duplicate = server.handle(did_change(uri, 2, invalid));
    assert!(duplicate.messages.is_empty());

    let fixed = server.handle(did_change(uri, 3, "value :: Int32 := 1;"));
    assert_eq!(fixed.messages.len(), 1);
    assert_eq!(fixed.messages[0]["params"]["version"], 3);
    assert_eq!(fixed.messages[0]["params"]["diagnostics"], json!([]));

    let duplicate_fix = server.handle(did_change(uri, 3, "value :: Int32 := 1;"));
    assert!(duplicate_fix.messages.is_empty());

    let stale = server.handle(did_change(uri, 2, invalid));
    assert!(stale.messages.is_empty());
    assert_eq!(server.documents[uri].version, 3);
    assert_eq!(server.documents[uri].text, "value :: Int32 := 1;");
}

#[test]
fn reports_specialization_errors_and_keeps_semantic_queries() {
    let text = "zero<A> :: A;\nzero<Int32> :: Int32 := 0i32;\nmain :: Unit -> Int32 := () -> (zero<Int64>).i32;\n";
    let uri = "file:///missing-implementation.mal";
    let mut server = Server::new();
    let opened = server.handle(did_open(uri, text));

    assert!(
        opened.messages[0]["params"]["diagnostics"][0]["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("missing operation implementation"))
    );
    let hover = request_at(
        &mut server,
        7,
        "textDocument/hover",
        uri,
        text,
        text.find("main").unwrap(),
    );
    assert_ne!(hover["result"], Value::Null);
}

#[test]
fn parse_error_keeps_semantic_requests_unavailable() {
    let text = "good :: Int32 := 1;\nbad :: Int32 := ;\n";
    let uri = "file:///invalid-semantic.mal";
    let mut server = open_document(uri, text);

    let hover = request_at(
        &mut server,
        8,
        "textDocument/hover",
        uri,
        text,
        text.find("good").unwrap(),
    );

    assert_eq!(hover["result"], Value::Null);
    assert!(hover.get("error").is_none());

    let definition = request_at(
        &mut server,
        9,
        "textDocument/definition",
        uri,
        text,
        text.find("good").unwrap(),
    );
    assert_eq!(definition["result"], Value::Null);

    let references = server.handle(json!({
        "jsonrpc": "2.0", "id": 10, "method": "textDocument/references",
        "params": {
            "textDocument": {"uri": uri}, "position": {"line": 0, "character": 1},
            "context": {"includeDeclaration": true}
        }
    }));
    assert_eq!(references.messages[0]["result"], json!([]));

    let rename = server.handle(json!({
        "jsonrpc": "2.0", "id": 11, "method": "textDocument/rename",
        "params": {
            "textDocument": {"uri": uri}, "position": {"line": 0, "character": 1},
            "newName": "renamed"
        }
    }));
    assert_eq!(rename.messages[0]["result"], Value::Null);

    let symbols = server.handle(json!({
        "jsonrpc": "2.0", "id": 12, "method": "textDocument/documentSymbol",
        "params": {"textDocument": {"uri": uri}}
    }));
    assert_eq!(symbols.messages[0]["result"], json!([]));
    assert!(symbols.messages[0].get("error").is_none());

    let completion = request_at(
        &mut server,
        13,
        "textDocument/completion",
        uri,
        text,
        text.find("good").unwrap(),
    );
    assert_eq!(completion["result"], json!([]));
    assert!(completion.get("error").is_none());
    let tokens = server.handle(json!({
        "jsonrpc": "2.0", "id": 14, "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": {"uri": uri}}
    }));
    assert_eq!(
        tokens.messages[0]["result"]["data"]
            .as_array()
            .unwrap()
            .len(),
        20
    );
    assert!(server.documents[uri].analysis_is_current());
}

#[test]
fn check_error_keeps_independent_semantics_and_navigation_available() {
    let text = "before :: Int32 := 1;\n\
                broken :: Unit -> Int32 := () -> {\n\
                  bad := (value) -> value;\n\
                  0\n\
                };\n\
                after :: Int32 := before;\n";
    let uri = "file:///partial-semantic.mal";
    let mut server = open_document(uri, text);
    let declaration = text.find("before").unwrap();
    let reference = text.rfind("before").unwrap();

    for (id, offset) in [
        (40, declaration),
        (41, reference),
        (42, text.find("after").unwrap()),
    ] {
        let hover = request_at(&mut server, id, "textDocument/hover", uri, text, offset);
        assert!(hover.get("error").is_none());
        assert!(
            hover["result"]["contents"]["value"]
                .as_str()
                .is_some_and(|contents| contents.contains(":: Int32")),
            "unexpected hover: {hover}"
        );
    }

    let unknown = request_at(
        &mut server,
        43,
        "textDocument/hover",
        uri,
        text,
        text.find("bad :=").unwrap(),
    );
    assert_eq!(unknown["result"], Value::Null);
    assert!(unknown.get("error").is_none());

    let definition = request_at(
        &mut server,
        44,
        "textDocument/definition",
        uri,
        text,
        reference,
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(text, declaration)
    );
    assert!(definition.get("error").is_none());

    let position = text_position(text, reference);
    let references = server.handle(json!({
        "jsonrpc": "2.0", "id": 45, "method": "textDocument/references",
        "params": {
            "textDocument": {"uri": uri}, "position": position,
            "context": {"includeDeclaration": true}
        }
    }));
    assert_eq!(
        references.messages[0]["result"].as_array().unwrap().len(),
        2
    );
    assert!(references.messages[0].get("error").is_none());

    let rename = server.handle(json!({
        "jsonrpc": "2.0", "id": 46, "method": "textDocument/rename",
        "params": {
            "textDocument": {"uri": uri}, "position": position,
            "newName": "renamed"
        }
    }));
    assert_eq!(
        rename.messages[0]["result"]["changes"][uri]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(rename.messages[0].get("error").is_none());
}

#[test]
fn check_error_uses_current_spans_and_recovers_after_the_fix() {
    let uri = "file:///partial-semantic-version.mal";
    let initial = "stable :: Int32 := 1;\nuse :: Unit -> Int32 := () -> stable;\n";
    let invalid = "padding :: Int32 := 0;\n\
                   stable :: Int32 := 1;\n\
                   broken :: Unit -> Int32 := () -> {\n\
                     bad := (value) -> value;\n\
                     0\n\
                   };\n\
                   use :: Unit -> Int32 := () -> stable;\n";
    let fixed = "padding :: Int32 := 0;\n\
                 stable :: Int32 := 1;\n\
                 broken :: Unit -> Int32 := () -> {\n\
                   bad :: Int32 -> Int32 := (value) -> value;\n\
                   0\n\
                 };\n\
                 use :: Unit -> Int32 := () -> stable;\n";
    let mut server = open_document(uri, initial);
    request_at(
        &mut server,
        50,
        "textDocument/hover",
        uri,
        initial,
        initial.rfind("stable").unwrap(),
    );

    let changed = server.handle(did_change(uri, 2, invalid));
    assert!(
        !changed.messages[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let definition = request_at(
        &mut server,
        51,
        "textDocument/definition",
        uri,
        invalid,
        invalid.rfind("stable").unwrap(),
    );
    assert_eq!(
        definition["result"]["range"]["start"],
        text_position(invalid, invalid.find("stable").unwrap())
    );
    assert!(definition.get("error").is_none());

    let repaired = server.handle(did_change(uri, 3, fixed));
    assert_eq!(repaired.messages[0]["params"]["diagnostics"], json!([]));
    let hover = request_at(
        &mut server,
        52,
        "textDocument/hover",
        uri,
        fixed,
        fixed.find("bad ::").unwrap(),
    );
    assert!(
        hover["result"]["contents"]["value"]
            .as_str()
            .is_some_and(|contents| contents.contains("bad :: Int32 -> Int32")),
        "unexpected hover after repair: {hover}"
    );
    assert!(hover.get("error").is_none());
}
