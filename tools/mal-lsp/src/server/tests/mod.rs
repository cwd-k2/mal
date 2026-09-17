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
    assert!(server.documents["file:///unicode.mal"].semantic.is_none());
    assert!(server.documents["file:///unicode.mal"].analysis.is_none());

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
    assert!(server.documents["file:///unicode.mal"].semantic.is_none());
    assert!(server.documents["file:///unicode.mal"].analysis.is_some());
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
fn returns_no_semantic_result_while_the_current_source_is_invalid() {
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
    assert!(server.documents[uri].analysis_current);
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
    let text = "make :: Int32 -> Int32 := (x) -> {\n  inner :: Unit -> Int32 := () -> { x; };\n  inner();\n};\n";
    let uri = "file:///semantic.mal";
    let mut server = open_document(uri, text);
    let reference = text.find("{ x;").unwrap() + 2;
    let parameter = text.find("(x)").unwrap() + 1;

    let hover = request_at(&mut server, 10, "textDocument/hover", uri, text, reference);
    assert_eq!(hover["result"]["contents"]["kind"], "markdown");
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nx :: Int32\n```\n\nparameter\n\nDefined in `semantic.mal:1:28`"
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
        2
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
        2
    );
}

#[test]
fn preserves_declared_type_aliases_in_hover() {
    let text =
        "Tree :: (Int64, Address, Address);\nf :: (Tree, Int64) -> Int64 := (tree, n) -> { n; };\n";
    let uri = "file:///alias-hover.mal";
    let mut server = open_document(uri, text);

    let alias = request_at(
        &mut server,
        14,
        "textDocument/hover",
        uri,
        text,
        text.find("Tree").unwrap(),
    );
    assert_eq!(
        alias["result"]["contents"]["value"],
        "```mal\nTree :: (Int64, Address, Address)\n```\n\ntype\n\nDefined in `alias-hover.mal:1:1`"
    );

    let function = request_at(
        &mut server,
        15,
        "textDocument/hover",
        uri,
        text,
        text.find("f ::").unwrap(),
    );
    assert_eq!(
        function["result"]["contents"]["value"],
        "```mal\nf :: (Tree, Int64) -> Int64\n```\n\nfunction\n\nDefined in `alias-hover.mal:2:1`"
    );
}

#[test]
fn expands_a_sum_result_type_hover_by_exactly_one_alias_layer() {
    let text = "Payload :: Int32;\nChoice :: [Unit, Payload];\nmake :: Payload -> Choice := (value) -> [none, some] => { some(value) };\nread :: Unit -> Choice := () -> { make(1) };\n";
    let uri = "file:///sum-result-hover.mal";
    let mut server = open_document(uri, text);
    let constructor = text.find("-> Choice").unwrap() + 3;
    let hover = request_at(
        &mut server,
        16,
        "textDocument/hover",
        uri,
        text,
        constructor,
    );

    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nChoice :: [Unit, Payload]\n```\n\ntype\n\nDefined in `sum-result-hover.mal:2:1`"
    );

    let function = request_at(
        &mut server,
        17,
        "textDocument/hover",
        uri,
        text,
        text.find("make ::").unwrap(),
    );
    assert_eq!(
        function["result"]["contents"]["value"],
        "```mal\nmake :: Payload -> Choice\n```\n\nfunction\n\nDefined in `sum-result-hover.mal:3:1`"
    );

    let call = text.rfind("make(1)").unwrap();
    let result = request_at(
        &mut server,
        18,
        "textDocument/hover",
        uri,
        text,
        call + "make".len(),
    );
    assert_eq!(
        result["result"]["contents"]["value"],
        "```mal\nmake(1) :: [Unit, Int32]\n```"
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
fn serves_hover_and_definition_for_an_alias_in_an_indexed_type() {
    let text = "Byte :: UInt8;\nidentity :: Cursor<Byte> -> Cursor<Byte> := (cursor) -> cursor;";
    let uri = "file:///indexed-type.mal";
    let mut server = open_document(uri, text);
    let reference = text.rfind("Byte").unwrap();
    let declaration = text.find("Byte").unwrap();

    let hover = request_at(&mut server, 21, "textDocument/hover", uri, text, reference);
    assert_eq!(
        hover["result"]["contents"]["value"],
        "```mal\nByte :: UInt8\n```\n\ntype\n\nDefined in `indexed-type.mal:1:1`"
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
