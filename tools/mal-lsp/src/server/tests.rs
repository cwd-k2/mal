use super::*;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

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
    let text =
        "make :: Int32 -> Int32 := (x) {\n  inner :: Unit -> Int32 := () { x; };\n  inner();\n};\n";
    let uri = "file:///semantic.mal";
    let mut server = open_document(uri, text);
    let reference = text.find("{ x;").unwrap() + 2;
    let parameter = text.find("(x)").unwrap() + 1;

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
    let text = "Tree :: (Int64, Ptr, Ptr);\nf :: (Tree, Int64) -> Int64 := (tree, n) { n; };\n";
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
        "```mal\nTree :: (Int64, Ptr, Ptr)\n```\n\ntype"
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
        "```mal\nf :: (Tree, Int64) -> Int64\n```\n\nfunction"
    );
}

#[test]
fn expands_a_sum_constructor_type_hover_by_exactly_one_alias_layer() {
    let text = "Payload :: Int32;\nChoice :: [Unit, Payload];\nmake :: Payload -> Choice := 1[Choice];\nread :: Unit -> Choice := () { make(1) };\n";
    let uri = "file:///sum-constructor-hover.mal";
    let mut server = open_document(uri, text);
    let constructor = text.find("1[Choice]").unwrap() + 2;
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
        "```mal\nChoice :: [Unit, Payload]\n```\n\ntype"
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
        "```mal\nmake :: Payload -> Choice\n```\n\nfunction"
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
fn serves_hover_and_definition_for_a_type_qualified_primitive() {
    let text = "Byte :: UInt8;\nsize :: UInt64 := Byte.size;";
    let uri = "file:///type-qualified-primitive.mal";
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
fn completes_lexical_functions_after_an_incomplete_receiver_suffix() {
    let text = "transform :: (Int32, Int32) -> Int32 := (value, option) { value + option };\n\
                count :: Int32 := 1;\n\
                main :: Unit -> Int32 := () { count. };\n";
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
    let text = "transform :: (Int32, Int32) -> Int32 := (value, option) { value + option };\n\
                count :: Int32 := 1;\n\
                main :: Unit -> Int32 := () { count.transform(1) };\n";
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
                     local :: Int32 -> Int32 := (value) { value };\n\
                     count :: Int32 := 1;\n\
                     main :: Unit -> Int32 := () { count.par };\n";
    let root_path = files.write("program.mal", root_text);
    files.write(
        "library.mal",
        "publicFunction :: Int32 -> Int32 := (value) { value };\n\
         _privateFunction :: Int32 -> Int32 := (value) { value };\n",
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

#[test]
fn completes_requirement_paths_while_the_symbol_is_unclosed() {
    let files = TestFiles::new();
    std::fs::create_dir(files.path.join("library")).expect("create library directory");
    files.write("library/util.mal", "value :: Int32 := 1;\n");
    files.write("library/util.c", "");
    files.write("library/util.txt", "");
    std::fs::create_dir(files.path.join("library/utilities")).expect("create nested directory");
    let text = "require \"library/ut";
    let root_path = files.write("program.mal", text);
    let uri = path_to_uri(&root_path);
    let mut server = open_document(&uri, text);

    let completion = request_at(
        &mut server,
        25,
        "textDocument/completion",
        &uri,
        text,
        text.len(),
    );
    let items = completion["result"].as_array().unwrap();
    let labels = items
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(labels, ["util.c", "util.mal", "utilities/"]);
    assert!(
        items.iter().all(|item| {
            item["textEdit"]["range"]["start"] == json!({"line": 0, "character": 17})
        })
    );
    assert!(items.iter().any(|item| item["kind"] == 19));
}

#[test]
fn navigates_from_a_requirement_path_even_when_semantic_analysis_fails() {
    let files = TestFiles::new();
    let library_path = files.write("library.mal", "value :: Int32 := 1;\n");
    let text = "require \"library.mal\";\nbroken :: Int32 := ;\n";
    let root_path = files.write("program.mal", text);
    let root_uri = path_to_uri(&root_path);
    let mut server = open_document(&root_uri, text);

    let definition = request_at(
        &mut server,
        26,
        "textDocument/definition",
        &root_uri,
        text,
        text.find("library").unwrap(),
    );

    assert_eq!(definition["result"]["uri"], path_to_uri(&library_path));
    assert_eq!(
        definition["result"]["range"],
        json!({
            "start": {"line": 0, "character": 0},
            "end": {"line": 0, "character": 0}
        })
    );
}

#[test]
fn serves_cross_file_semantics_from_open_dependency_buffers() {
    let files = TestFiles::new();
    let root_path = files.write("program.mal", "not the open buffer");
    let library_path = files.write("library.mal", "diskValue :: Int32 := 0;");
    let root_uri = path_to_uri(&root_path);
    let library_uri = path_to_uri(&library_path);
    let root_text = "require \"library.mal\";\nanswer :: Unit -> Int32 := () { publicValue; };\n";
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
