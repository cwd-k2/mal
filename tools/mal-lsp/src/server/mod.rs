use std::collections::HashMap;
use std::path::{Path, PathBuf};

use malc::source::{FileId, SourceFile, SourceGraph, Span, Utf16Position};
use serde::Deserialize;
use serde_json::{Value, json};

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

    fn diagnostics(&mut self, uri: &str) -> Option<Value> {
        let diagnostics = self.analyze_document(uri);
        let document = self.documents.get_mut(uri).expect("open document");
        let unchanged = document
            .published_diagnostics
            .as_ref()
            .is_some_and(|published| {
                published.version == document.version && published.diagnostics == diagnostics
            });
        if unchanged {
            return None;
        }
        document.published_diagnostics = Some(PublishedDiagnostics {
            version: document.version,
            diagnostics: diagnostics.clone(),
        });
        Some(publish_diagnostics(
            uri,
            Some(document.version),
            diagnostics,
        ))
    }

    fn invalidate_analyses(&mut self) {
        for document in self.documents.values_mut() {
            document.analysis = None;
            document.graph = None;
            document.semantic = None;
            document.analysis_current = false;
        }
    }

    fn publish_workspace_diagnostics(&mut self, primary: &str, messages: &mut Vec<Value>) {
        if let Some(message) = self.diagnostics(primary) {
            messages.push(message);
        }
        let mut remaining = self
            .documents
            .keys()
            .filter(|uri| uri.as_str() != primary)
            .cloned()
            .collect::<Vec<_>>();
        remaining.sort();
        for uri in remaining {
            if let Some(message) = self.diagnostics(&uri) {
                messages.push(message);
            }
        }
    }

    fn analyze_document(&mut self, uri: &str) -> Vec<Value> {
        let overlays = self
            .documents
            .iter()
            .filter_map(|(uri, document)| Some((uri_to_path(uri)?, document.text.clone())))
            .collect::<HashMap<_, _>>();
        let document = self.documents.get_mut(uri).expect("open document");
        document.analysis_current = true;
        let Some(path) = uri_to_path(uri) else {
            return document.analyze_single(uri);
        };
        match malc::driver::load_source_graph_with_overlays(&path, &document.text, &overlays) {
            Ok(graph) => match malc::pipeline::analyze_graph(&graph) {
                Ok(analysis) => {
                    document.graph = Some(graph);
                    document.analysis = Some(analysis);
                    Vec::new()
                }
                Err(diagnostic) => {
                    let source = diagnostic
                        .primary
                        .as_ref()
                        .and_then(|label| graph.source(label.span.file()))
                        .unwrap_or_else(|| graph.root_source());
                    let diagnostic = if source.id() == graph.root() {
                        lsp_diagnostic(source, diagnostic)
                    } else {
                        lsp_diagnostic_at_root(source, diagnostic)
                    };
                    document.graph = Some(graph);
                    vec![diagnostic]
                }
            },
            Err(load_error) => {
                let source = document.source(uri);
                match malc::pipeline::analyze(&source) {
                    Err(diagnostic) => vec![lsp_diagnostic(&source, diagnostic)],
                    Ok(_) => vec![json!({
                        "range": zero_range(), "severity": 1, "source": "malc",
                        "message": load_error.to_string()
                    })],
                }
            }
        }
    }

    fn ensure_analyzed(&mut self, uri: &str) -> bool {
        if let Some(document) = self.documents.get(uri)
            && document.analysis_current
        {
            return document.analysis.is_some();
        }
        self.analyze_document(uri);
        self.documents
            .get(uri)
            .is_some_and(|document| document.analysis.is_some())
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

impl Document {
    fn source(&self, uri: &str) -> SourceFile {
        self.graph.as_ref().map_or_else(
            || SourceFile::new(self.id, uri, self.text.clone()),
            |graph| SourceFile::new(graph.root(), graph.root_source().path(), self.text.clone()),
        )
    }

    fn semantic(&mut self) -> Option<&malc::editor::SemanticDocument> {
        if self.semantic.is_none() {
            let analysis = self.analysis.as_ref()?;
            self.semantic = Some(self.graph.as_ref().map_or_else(
                || malc::editor::from_analysis_for_file(analysis, self.id),
                |graph| malc::editor::from_graph_analysis(graph, analysis, graph.root()),
            ));
        }
        self.semantic.as_ref()
    }

    fn analyze_single(&mut self, uri: &str) -> Vec<Value> {
        let source = self.source(uri);
        match malc::pipeline::analyze(&source) {
            Ok(analysis) => {
                self.analysis = Some(analysis);
                Vec::new()
            }
            Err(diagnostic) => vec![lsp_diagnostic(&source, diagnostic)],
        }
    }

    fn source_for(&self, span: malc::source::Span, uri: &str) -> Option<SourceFile> {
        if let Some(graph) = &self.graph {
            let source = graph.source(span.file())?;
            return Some(SourceFile::new(
                source.id(),
                source.path(),
                source.text().to_owned(),
            ));
        }
        let source = self.source(uri);
        source.contains(span).then_some(source)
    }
}

fn lsp_diagnostic(source: &SourceFile, diagnostic: malc::diagnostic::Diagnostic) -> Value {
    let (range, label) = diagnostic.primary.map_or_else(
        || (zero_range(), None),
        |label| {
            let start = source.utf16_position(label.span.start());
            let end = source.utf16_position(label.span.end());
            let range = match (start, end) {
                (Some(start), Some(end)) => json!({"start": position(start), "end": position(end)}),
                _ => zero_range(),
            };
            (range, Some(label.message))
        },
    );
    let mut message = diagnostic.message;
    if let Some(label) = label {
        message.push_str(": ");
        message.push_str(&label);
    }
    for note in diagnostic.notes {
        message.push_str("\nnote: ");
        message.push_str(&note);
    }
    json!({"range": range, "severity": 1, "source": "malc", "message": message})
}

fn lsp_diagnostic_at_root(source: &SourceFile, diagnostic: malc::diagnostic::Diagnostic) -> Value {
    let mut value = lsp_diagnostic(source, diagnostic);
    value["range"] = zero_range();
    if let Some(message) = value["message"].as_str() {
        value["message"] = Value::String(format!("{}: {message}", source.path().display()));
    }
    value
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
