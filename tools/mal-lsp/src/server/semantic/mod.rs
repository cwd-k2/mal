use std::collections::HashMap;

use malc::editor::{Hover, OccurrenceRole, SemanticDocument, SymbolKind};
use malc::source::{SourceFile, Span, Utf16Position};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{Server, TextDocumentIdentifier, error, path_to_uri, position, success};

mod completion;
mod tokens;

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
        if let Some(location) = self.requirement_definition(&params) {
            return success(id, location);
        }
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
        if let Some(position) = request.position
            && let Some(items) = completion::requirement_completions(&uri, &source, position)
        {
            return success(id, json!(items));
        }
        let receiver_context = request.position.is_some_and(|position| {
            source
                .byte_offset_utf16(Utf16Position {
                    line: position.line,
                    character: position.character,
                })
                .and_then(|offset| completion::receiver_suffix_start(source.text(), offset))
                .is_some()
        });
        if self.ensure_analyzed(&uri) {
            let Some(document) = self.documents.get_mut(&uri) else {
                return error(id, -32602, "document is not open");
            };
            let Some(semantic) = document.semantic() else {
                return success(id, json!([]));
            };
            return success(
                id,
                json!(completion::completion_items(semantic, receiver_context)),
            );
        }
        if !receiver_context {
            return success(id, json!([]));
        }
        success(
            id,
            json!(completion::lexical_function_completions(
                self, &uri, &source
            )),
        )
    }

    fn requirement_definition(&self, params: &Value) -> Option<Value> {
        let request = serde_json::from_value::<PositionParams>(params.clone()).ok()?;
        let document = self.documents.get(&request.text_document.uri)?;
        let source = document.source(&request.text_document.uri);
        let offset = source.byte_offset_utf16(Utf16Position {
            line: request.position.line,
            character: request.position.character,
        })?;
        let target = super::requirement::target_path(&request.text_document.uri, &source, offset)?;
        let target_uri = super::path_to_uri(&target);
        if !target.is_file() && !self.documents.contains_key(&target_uri) {
            return None;
        }
        Some(json!({"uri": target_uri, "range": super::zero_range()}))
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
                return success(id, json!({"data": tokens::lexical(&source)}));
            }
            SemanticRequest::Invalid => {
                return error(id, -32602, "invalid parameters or document is not open");
            }
        };
        let data = tokens::encode(
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
