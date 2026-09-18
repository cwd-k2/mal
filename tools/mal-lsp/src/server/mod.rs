use std::collections::HashMap;
use std::path::{Path, PathBuf};

use malc::source::{FileId, SourceFile, SourceGraph, Span, Utf16Position};
use serde::Deserialize;
use serde_json::{Value, json};

mod analysis;
mod requirement;
mod semantic;

pub struct Outcome {
    pub messages: Vec<Value>,
    pub exit: Option<bool>,
}

pub struct Server {
    documents: HashMap<String, Document>,
    next_file_id: u32,
    shutdown: bool,
}

struct Document {
    id: FileId,
    version: i64,
    text: String,
    analysis: Option<malc::pipeline::Analysis>,
    graph: Option<SourceGraph>,
    semantic: Option<malc::editor::SemanticDocument>,
    analysis_current: bool,
    published_diagnostics: Option<PublishedDiagnostics>,
}

struct PublishedDiagnostics {
    version: i64,
    diagnostics: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DidOpenParams {
    text_document: TextDocumentItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextDocumentItem {
    uri: String,
    version: i64,
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DidChangeParams {
    text_document: VersionedTextDocumentIdentifier,
    content_changes: Vec<ContentChange>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionedTextDocumentIdentifier {
    uri: String,
    version: i64,
}

#[derive(Deserialize)]
struct ContentChange {
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DidCloseParams {
    text_document: TextDocumentIdentifier,
}

#[derive(Deserialize)]
struct FormattingParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,
}

#[derive(Deserialize)]
struct TextDocumentIdentifier {
    uri: String,
}

impl Server {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            next_file_id: 0,
            shutdown: false,
        }
    }

    pub fn handle(&mut self, message: Value) -> Outcome {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let mut messages = Vec::new();
        let mut exit = None;

        match (method, id) {
            (Some("initialize"), Some(id)) => messages.push(success(
                id,
                json!({
                    "capabilities": {
                        "positionEncoding": "utf-16",
                        "textDocumentSync": 1,
                        "documentFormattingProvider": true,
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "documentLinkProvider": {},
                        "referencesProvider": true,
                        "renameProvider": true,
                        "documentSymbolProvider": true,
                        "completionProvider": {"triggerCharacters": [".", "\"", "/"]},
                        "semanticTokensProvider": {
                            "legend": {
                                "tokenTypes": ["type", "variable", "parameter", "function"],
                                "tokenModifiers": ["declaration"]
                            },
                            "full": true
                        }
                    },
                    "serverInfo": {"name": "mal-lsp", "version": env!("CARGO_PKG_VERSION")}
                }),
            )),
            (Some("shutdown"), Some(id)) => {
                self.shutdown = true;
                messages.push(success(id, Value::Null));
            }
            (Some("textDocument/formatting"), Some(id)) => {
                messages.push(self.formatting(id, params));
            }
            (Some("textDocument/hover"), Some(id)) => {
                messages.push(self.hover(id, params));
            }
            (Some("textDocument/definition"), Some(id)) => {
                messages.push(self.definition(id, params));
            }
            (Some("textDocument/documentLink"), Some(id)) => {
                messages.push(self.document_links(id, params));
            }
            (Some("textDocument/references"), Some(id)) => {
                messages.push(self.references(id, params));
            }
            (Some("textDocument/rename"), Some(id)) => {
                messages.push(self.rename(id, params));
            }
            (Some("textDocument/documentSymbol"), Some(id)) => {
                messages.push(self.document_symbols(id, params));
            }
            (Some("textDocument/completion"), Some(id)) => {
                messages.push(self.completion(id, params));
            }
            (Some("textDocument/semanticTokens/full"), Some(id)) => {
                messages.push(self.semantic_tokens(id, params));
            }
            (Some(_), Some(id)) => messages.push(error(id, -32601, "method not found")),
            (Some("initialized"), None) => {}
            (Some("exit"), None) => exit = Some(self.shutdown),
            (Some("textDocument/didOpen"), None) => {
                if let Ok(params) = serde_json::from_value::<DidOpenParams>(params) {
                    let item = params.text_document;
                    let id = FileId::new(self.next_file_id);
                    self.next_file_id = self.next_file_id.wrapping_add(1);
                    self.documents.insert(
                        item.uri.clone(),
                        Document {
                            id,
                            version: item.version,
                            text: item.text,
                            analysis: None,
                            graph: None,
                            semantic: None,
                            analysis_current: false,
                            published_diagnostics: None,
                        },
                    );
                    self.invalidate_analyses();
                    self.publish_workspace_diagnostics(&item.uri, &mut messages);
                }
            }
            (Some("textDocument/didChange"), None) => {
                if let Ok(params) = serde_json::from_value::<DidChangeParams>(params)
                    && let Some(text) = params.content_changes.last()
                    && let Some(document) = self.documents.get_mut(&params.text_document.uri)
                    && params.text_document.version > document.version
                {
                    document.version = params.text_document.version;
                    document.text.clone_from(&text.text);
                    self.invalidate_analyses();
                    self.publish_workspace_diagnostics(&params.text_document.uri, &mut messages);
                }
            }
            (Some("textDocument/didClose"), None) => {
                if let Ok(params) = serde_json::from_value::<DidCloseParams>(params) {
                    self.documents.remove(&params.text_document.uri);
                    messages.push(publish_diagnostics(
                        &params.text_document.uri,
                        None,
                        Vec::new(),
                    ));
                    self.invalidate_analyses();
                    let remaining = self.documents.keys().cloned().collect::<Vec<_>>();
                    for uri in remaining {
                        if let Some(message) = self.diagnostics(&uri) {
                            messages.push(message);
                        }
                    }
                }
            }
            _ => {}
        }
        Outcome { messages, exit }
    }

    fn formatting(&self, id: Value, params: Value) -> Value {
        let Ok(params) = serde_json::from_value::<FormattingParams>(params) else {
            return error(id, -32602, "invalid formatting parameters");
        };
        let Some(document) = self.documents.get(&params.text_document.uri) else {
            return error(id, -32602, "document is not open");
        };
        let source = document.source(&params.text_document.uri);
        let Ok(formatted) = malc::formatter::format(&source) else {
            return success(id, Value::Null);
        };
        if formatted == document.text {
            return success(id, json!([]));
        }
        let end = source
            .utf16_position(source.text().len())
            .expect("source end must have a position");
        success(
            id,
            json!([{
                "range": {"start": position(Utf16Position { line: 0, character: 0 }), "end": position(end)},
                "newText": formatted
            }]),
        )
    }
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?;
    let encoded = encoded.strip_prefix("localhost").unwrap_or(encoded);
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex(*bytes.get(index + 1)?)?;
            let low = hex(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok().map(PathBuf::from)
}

fn path_to_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            uri.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(uri, "%{byte:02X}").expect("writing to a string cannot fail");
        }
    }
    uri
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn publish_diagnostics(uri: &str, version: Option<i64>, diagnostics: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "version": version, "diagnostics": diagnostics}
    })
}

fn success(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn position(position: Utf16Position) -> Value {
    json!({"line": position.line, "character": position.character})
}

fn span_range(source: &SourceFile, span: Span) -> Value {
    let start = source
        .utf16_position(span.start())
        .expect("source span start must have a position");
    let end = source
        .utf16_position(span.end())
        .expect("source span end must have a position");
    json!({"start": position(start), "end": position(end)})
}

fn zero_range() -> Value {
    json!({
        "start": position(Utf16Position { line: 0, character: 0 }),
        "end": position(Utf16Position { line: 0, character: 0 })
    })
}

#[cfg(test)]
mod tests;
