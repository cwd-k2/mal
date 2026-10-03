use super::*;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

mod completion;
mod requirement;
mod workspace;

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

    for (id, method, empty) in [
        (12, "textDocument/documentSymbol", json!([])),
        (13, "textDocument/completion", json!([])),
    ] {
        let outcome = server.handle(json!({
            "jsonrpc": "2.0", "id": id, "method": method,
            "params": {"textDocument": {"uri": uri}}
        }));
        assert_eq!(outcome.messages[0]["result"], empty);
        assert!(outcome.messages[0].get("error").is_none());
    }
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

mod semantic;

fn open_document(uri: &str, text: &str) -> Server {
    let mut server = Server::new();
    server.handle(did_open(uri, text));
    server
}

fn did_open(uri: &str, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": {"textDocument": {"uri": uri, "languageId": "mal", "version": 1, "text": text}}
    })
}

fn did_change(uri: &str, version: i64, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {
            "textDocument": {"uri": uri, "version": version},
            "contentChanges": [{"text": text}]
        }
    })
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

struct TestFiles {
    path: PathBuf,
}

impl TestFiles {
    fn new() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("mal-lsp-test-{}-{sequence}", std::process::id()));
        std::fs::create_dir(&path).expect("create LSP test directory");
        Self { path }
    }

    fn write(&self, name: impl AsRef<Path>, text: &str) -> PathBuf {
        let path = self.path.join(name);
        std::fs::write(&path, text).expect("write LSP test file");
        path
    }
}

impl Drop for TestFiles {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove LSP test directory");
    }
}
