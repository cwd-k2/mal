use super::super::*;
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
    assert_eq!(capabilities["documentLinkProvider"], json!({}));
    assert_eq!(
        capabilities["completionProvider"]["triggerCharacters"],
        json!([".", "\"", "/"])
    );
    assert_eq!(
        capabilities["semanticTokensProvider"]["legend"]["tokenTypes"],
        json!(["type", "variable", "parameter", "function"])
    );
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
fn rejects_completion_without_a_position() {
    let uri = "file:///completion-parameters.mal";
    let mut server = open_document(uri, "value :: Int32 := 1;\n");

    let outcome = server.handle(json!({
        "jsonrpc": "2.0", "id": 1, "method": "textDocument/completion",
        "params": {"textDocument": {"uri": uri}}
    }));

    assert_eq!(outcome.messages[0]["error"]["code"], -32602);
}

#[test]
fn accepts_only_local_file_uris() {
    assert_eq!(
        uri_to_path("file:///work/main.mal"),
        Some(PathBuf::from("/work/main.mal"))
    );
    assert_eq!(
        uri_to_path("file://localhost/work/main.mal"),
        Some(PathBuf::from("/work/main.mal"))
    );
    assert_eq!(uri_to_path("file://localhostevil/work/main.mal"), None);
    assert_eq!(uri_to_path("file://server/work/main.mal"), None);
    assert_eq!(uri_to_path("file://relative.mal"), None);
}
