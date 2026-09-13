use std::collections::HashMap;

use malc::editor::{Hover, OccurrenceRole, SemanticDocument, SymbolKind};
use malc::source::{SourceFile, Span, Utf16Position};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{Server, TextDocumentIdentifier, error, path_to_uri, position, success};

#[derive(Clone, Copy, Deserialize)]
struct Position {
    line: usize,
    character: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PositionParams {
    text_document: TextDocumentIdentifier,
    position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionParams {
    text_document: TextDocumentIdentifier,
    position: Option<Position>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceParams {
    text_document: TextDocumentIdentifier,
    position: Position,
    context: ReferenceContext,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceContext {
    include_declaration: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameParams {
    text_document: TextDocumentIdentifier,
    position: Position,
    new_name: String,
}

impl Server {
    pub(super) fn hover(&mut self, id: Value, params: Value) -> Value {
        let (source, semantic, offset) = match self.position_request(&params) {
            SemanticRequest::Ready(value) => value,
            SemanticRequest::Unavailable => return success(id, Value::Null),
            SemanticRequest::Invalid => {
                return error(id, -32602, "invalid position or document is not open");
            }
        };
        let Some(hover) = semantic.hover_at(offset) else {
            return success(id, Value::Null);
        };
        success(
            id,
            json!({
                "contents": {"kind": "markdown", "value": hover_contents(&source, hover)},
                "range": span_range(&source, hover.span)
            }),
        )
    }

    pub(super) fn definition(&mut self, id: Value, params: Value) -> Value {
        let (_, semantic, offset) = match self.position_request(&params) {
            SemanticRequest::Ready(value) => value,
            SemanticRequest::Unavailable => return success(id, Value::Null),
            SemanticRequest::Invalid => {
                return error(id, -32602, "invalid position or document is not open");
            }
        };
        let Some(span) = semantic
            .occurrence_at(offset)
            .and_then(|occurrence| semantic.definition(occurrence.id))
            .map(|definition| definition.span)
        else {
            return success(id, Value::Null);
        };
        let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
        let Some((target_uri, range)) = self.span_location(uri, span) else {
            return success(id, Value::Null);
        };
        success(id, json!({"uri": target_uri, "range": range}))
    }

    pub(super) fn references(&mut self, id: Value, params: Value) -> Value {
        let Ok(request) = serde_json::from_value::<ReferenceParams>(params) else {
            return error(id, -32602, "invalid reference parameters");
        };
        let (_, semantic, offset) =
            match self.semantic_at(&request.text_document.uri, request.position) {
                SemanticRequest::Ready(value) => value,
                SemanticRequest::Unavailable => return success(id, json!([])),
                SemanticRequest::Invalid => {
                    return error(id, -32602, "invalid position or document is not open");
                }
            };
        let Some(occurrence) = semantic.occurrence_at(offset) else {
            return success(id, json!([]));
        };
        let spans = semantic
            .references(occurrence.id, request.context.include_declaration)
            .into_iter()
            .map(|occurrence| occurrence.span)
            .collect::<Vec<_>>();
        let locations = spans
            .into_iter()
            .filter_map(|span| {
                let (uri, range) = self.span_location(&request.text_document.uri, span)?;
                Some(json!({"uri": uri, "range": range}))
            })
            .collect::<Vec<_>>();
        success(id, json!(locations))
    }

    pub(super) fn rename(&mut self, id: Value, params: Value) -> Value {
        let Ok(request) = serde_json::from_value::<RenameParams>(params) else {
            return error(id, -32602, "invalid rename parameters");
        };
        let (_, semantic, offset) =
            match self.semantic_at(&request.text_document.uri, request.position) {
                SemanticRequest::Ready(value) => value,
                SemanticRequest::Unavailable => return success(id, Value::Null),
                SemanticRequest::Invalid => {
                    return error(id, -32602, "invalid position or document is not open");
                }
            };
        let Some(spans) = semantic.rename_spans(offset) else {
            return success(id, Value::Null);
        };
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();
        for span in spans {
            let Some((uri, range)) = self.span_location(&request.text_document.uri, span) else {
                continue;
            };
            changes
                .entry(uri)
                .or_default()
                .push(json!({"range": range, "newText": request.new_name}));
        }
        success(id, json!({"changes": changes}))
    }

    pub(super) fn document_symbols(&mut self, id: Value, params: Value) -> Value {
        let (source, semantic) = match self.document_request(&params) {
            SemanticRequest::Ready(value) => value,
            SemanticRequest::Unavailable => return success(id, json!([])),
            SemanticRequest::Invalid => {
                return error(id, -32602, "invalid parameters or document is not open");
            }
        };
        let symbols = semantic
            .document_symbols()
            .iter()
            .filter_map(|symbol| {
                let span = symbol.span?;
                Some(json!({
                    "name": symbol.name,
                    "detail": symbol.detail,
                    "kind": symbol_kind(symbol.kind),
                    "range": span_range(&source, span),
                    "selectionRange": span_range(&source, span)
                }))
            })
            .collect::<Vec<_>>();
        success(id, json!(symbols))
    }

    pub(super) fn completion(&mut self, id: Value, params: Value) -> Value {
        let Ok(request) = serde_json::from_value::<CompletionParams>(params) else {
            return error(id, -32602, "invalid completion parameters");
        };
        let uri = request.text_document.uri;
        if !self.documents.contains_key(&uri) {
            return error(id, -32602, "document is not open");
        }
        let source = self.documents[&uri].source(&uri);
        let receiver_context = request.position.is_some_and(|position| {
            source
                .byte_offset_utf16(Utf16Position {
                    line: position.line,
                    character: position.character,
                })
                .and_then(|offset| receiver_suffix_start(source.text(), offset))
                .is_some()
        });
        if self.ensure_analyzed(&uri) {
            let Some(document) = self.documents.get_mut(&uri) else {
                return error(id, -32602, "document is not open");
            };
            let Some(semantic) = document.semantic() else {
                return success(id, json!([]));
            };
            return success(id, json!(completion_items(semantic, receiver_context)));
        }
        if !receiver_context {
            return success(id, json!([]));
        }
        success(id, json!(lexical_function_completions(self, &uri, &source)))
    }

    pub(super) fn semantic_tokens(&mut self, id: Value, params: Value) -> Value {
        let (source, semantic) = match self.document_request(&params) {
            SemanticRequest::Ready(value) => value,
            SemanticRequest::Unavailable => {
                let Ok(request) = serde_json::from_value::<DocumentRequest>(params) else {
                    return error(id, -32602, "invalid semantic token parameters");
                };
                let Some(document) = self.documents.get(&request.text_document.uri) else {
                    return error(id, -32602, "document is not open");
                };
                let source = document.source(&request.text_document.uri);
                return success(id, json!({"data": lexical_semantic_tokens(&source)}));
            }
            SemanticRequest::Invalid => {
                return error(id, -32602, "invalid parameters or document is not open");
            }
        };
        let data = encode_semantic_tokens(
            &source,
            semantic.document_occurrences().map(|occurrence| {
                (
                    occurrence.span,
                    occurrence.kind,
                    occurrence.role == OccurrenceRole::Declaration,
                )
            }),
        );
        success(id, json!({"data": data}))
    }

    fn position_request(
        &mut self,
        params: &Value,
    ) -> SemanticRequest<(SourceFile, &SemanticDocument, usize)> {
        let Ok(request) = serde_json::from_value::<PositionParams>(params.clone()) else {
            return SemanticRequest::Invalid;
        };
        self.semantic_at(&request.text_document.uri, request.position)
    }

    fn semantic_at(
        &mut self,
        uri: &str,
        position: Position,
    ) -> SemanticRequest<(SourceFile, &SemanticDocument, usize)> {
        let Some(document) = self.documents.get(uri) else {
            return SemanticRequest::Invalid;
        };
        let source = document.source(uri);
        let Some(offset) = source.byte_offset_utf16(Utf16Position {
            line: position.line,
            character: position.character,
        }) else {
            return SemanticRequest::Invalid;
        };
        if !self.ensure_analyzed(uri) {
            return SemanticRequest::Unavailable;
        }
        let Some(document) = self.documents.get_mut(uri) else {
            return SemanticRequest::Invalid;
        };
        let source = document.source(uri);
        let Some(semantic) = document.semantic() else {
            return SemanticRequest::Unavailable;
        };
        SemanticRequest::Ready((source, semantic, offset))
    }

    fn document_request(
        &mut self,
        params: &Value,
    ) -> SemanticRequest<(SourceFile, &SemanticDocument)> {
        let Ok(identifier) = serde_json::from_value::<DocumentRequest>(params.clone()) else {
            return SemanticRequest::Invalid;
        };
        let uri = identifier.text_document.uri;
        if !self.documents.contains_key(&uri) {
            return SemanticRequest::Invalid;
        }
        if !self.ensure_analyzed(&uri) {
            return SemanticRequest::Unavailable;
        }
        let Some(document) = self.documents.get_mut(&uri) else {
            return SemanticRequest::Invalid;
        };
        let source = document.source(&uri);
        let Some(semantic) = document.semantic() else {
            return SemanticRequest::Unavailable;
        };
        SemanticRequest::Ready((source, semantic))
    }

    fn span_location(&self, root_uri: &str, span: Span) -> Option<(String, Value)> {
        let document = self.documents.get(root_uri)?;
        let source = document.source_for(span, root_uri)?;
        let uri = document.graph.as_ref().map_or_else(
            || root_uri.to_owned(),
            |graph| {
                if span.file() == graph.root() {
                    root_uri.to_owned()
                } else {
                    path_to_uri(source.path())
                }
            },
        );
        Some((uri, span_range(&source, span)))
    }
}

enum SemanticRequest<T> {
    Ready(T),
    Unavailable,
    Invalid,
}

fn hover_contents(source: &SourceFile, hover: Hover<'_>) -> String {
    let (declaration, label) = if let Some(occurrence) = hover.occurrence {
        let declaration = if occurrence.kind == SymbolKind::Type && occurrence.name == hover.ty {
            occurrence.name.clone()
        } else {
            format!("{} :: {}", occurrence.name, hover.ty)
        };
        let label = match (occurrence.id, occurrence.kind) {
            (_, SymbolKind::Type) => "type",
            (_, SymbolKind::Function) => "function",
            (_, SymbolKind::Parameter) => "parameter",
            (_, SymbolKind::Value) => "value",
        };
        (declaration, Some(label))
    } else {
        let expression = &source.text()[hover.span.start()..hover.span.end()];
        (format!("{expression} :: {}", hover.ty), None)
    };
    let mut contents = format!("```mal\n{declaration}\n```");
    if let Some(label) = label {
        contents.push_str("\n\n");
        contents.push_str(label);
    }
    contents
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentRequest {
    text_document: TextDocumentIdentifier,
}
fn span_range(source: &SourceFile, span: Span) -> Value {
    let start = source
        .utf16_position(span.start())
        .expect("semantic span start must have a position");
    let end = source
        .utf16_position(span.end())
        .expect("semantic span end must have a position");
    json!({"start": position(start), "end": position(end)})
}

fn symbol_kind(kind: SymbolKind) -> usize {
    match kind {
        SymbolKind::Type => 5,
        SymbolKind::Function => 12,
        SymbolKind::Value | SymbolKind::Parameter => 13,
    }
}

fn completion_kind(kind: SymbolKind) -> usize {
    match kind {
        SymbolKind::Type => 7,
        SymbolKind::Function => 3,
        SymbolKind::Value | SymbolKind::Parameter => 6,
    }
}

fn completion_items(semantic: &SemanticDocument, functions_only: bool) -> Vec<Value> {
    semantic
        .completions()
        .iter()
        .filter(|symbol| !functions_only || symbol.kind == SymbolKind::Function)
        .map(|symbol| {
            json!({
                "label": symbol.name,
                "kind": completion_kind(symbol.kind),
                "detail": symbol.detail
            })
        })
        .collect()
}

fn lexical_function_completions(server: &Server, uri: &str, source: &SourceFile) -> Vec<Value> {
    let syntax = malc::editor::analyze_syntax(source).ok();
    let mut names = syntax
        .as_ref()
        .map(|syntax| syntax.functions().to_vec())
        .unwrap_or_default();
    let document = &server.documents[uri];
    let recovered_graph;
    let graph = if let Some(graph) = &document.graph {
        Some(graph)
    } else {
        recovered_graph = syntax.and_then(|syntax| {
            let path = super::uri_to_path(uri)?;
            let mut requirements = String::new();
            for span in syntax.requirements() {
                requirements.push_str(&source.text()[span.start()..span.end()]);
                requirements.push('\n');
            }
            let overlays = server
                .documents
                .iter()
                .filter_map(|(uri, document)| {
                    Some((super::uri_to_path(uri)?, document.text.clone()))
                })
                .collect::<HashMap<_, _>>();
            malc::driver::load_source_graph_with_overlays(&path, &requirements, &overlays).ok()
        });
        recovered_graph.as_ref()
    };
    if let Some(graph) = graph {
        for requirement in graph.requirements(graph.root()) {
            let Some(source) = graph.source(requirement.target) else {
                continue;
            };
            let Ok(syntax) = malc::editor::analyze_syntax(source) else {
                continue;
            };
            names.extend(
                syntax
                    .functions()
                    .iter()
                    .filter(|name| !name.starts_with('_'))
                    .cloned(),
            );
        }
    }
    names.sort();
    names.dedup();
    names
        .into_iter()
        .map(|name| json!({"label": name, "kind": completion_kind(SymbolKind::Function)}))
        .collect()
}

fn receiver_suffix_start(text: &str, offset: usize) -> Option<usize> {
    if offset > text.len() || !text.is_char_boundary(offset) {
        return None;
    }
    let bytes = text.as_bytes();
    let mut start = offset;
    while start > 0 && bytes[start - 1].is_ascii_alphanumeric() {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'_' {
        start -= 1;
    }
    let partial = &bytes[start..offset];
    if !partial.is_empty()
        && !(partial[0].is_ascii_lowercase()
            || (partial[0] == b'_' && partial.get(1).is_none_or(|byte| byte.is_ascii_alphabetic())))
    {
        return None;
    }
    let dot = start.checked_sub(1)?;
    (bytes[dot] == b'.').then_some(dot)
}

fn lexical_semantic_tokens(source: &SourceFile) -> Vec<usize> {
    let Ok(syntax) = malc::editor::analyze_syntax(source) else {
        return Vec::new();
    };
    encode_semantic_tokens(
        source,
        syntax
            .tokens()
            .iter()
            .map(|token| (token.span, token.kind, token.declaration)),
    )
}

fn encode_semantic_tokens(
    source: &SourceFile,
    tokens: impl IntoIterator<Item = (Span, SymbolKind, bool)>,
) -> Vec<usize> {
    let tokens = tokens.into_iter();
    let mut data = Vec::with_capacity(tokens.size_hint().0 * 5);
    let mut previous = Utf16Position {
        line: 0,
        character: 0,
    };
    for (span, kind, declaration) in tokens {
        let Some(start) = source.utf16_position(span.start()) else {
            continue;
        };
        let Some(end) = source.utf16_position(span.end()) else {
            continue;
        };
        if start.line != end.line {
            continue;
        }
        let delta_line = start.line - previous.line;
        let delta_start = if delta_line == 0 {
            start.character - previous.character
        } else {
            start.character
        };
        data.extend([
            delta_line,
            delta_start,
            end.character - start.character,
            semantic_token_kind(kind),
            usize::from(declaration),
        ]);
        previous = start;
    }
    data
}

fn semantic_token_kind(kind: SymbolKind) -> usize {
    match kind {
        SymbolKind::Type => 0,
        SymbolKind::Value => 1,
        SymbolKind::Parameter => 2,
        SymbolKind::Function => 3,
    }
}
