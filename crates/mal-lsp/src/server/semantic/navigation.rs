//! Navigation requests: definitions, references, rename, and document symbols.

use super::*;

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
    pub(in crate::server) fn definition(&mut self, id: Value, params: Value) -> Value {
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

    pub(in crate::server) fn references(&mut self, id: Value, params: Value) -> Value {
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

    pub(in crate::server) fn rename(&mut self, id: Value, params: Value) -> Value {
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
        let (Some(spans), Some(occurrence)) = (
            semantic.rename_spans(offset),
            semantic.occurrence_at(offset),
        ) else {
            return success(id, Value::Null);
        };
        if !renames_to_same_kind(&occurrence.name, &request.new_name) {
            return error(
                id,
                -32602,
                "the new name must be one identifier of the same kind as the old one",
            );
        }
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

    pub(in crate::server) fn document_symbols(&mut self, id: Value, params: Value) -> Value {
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

    fn requirement_definition(&self, params: &Value) -> Option<Value> {
        let request = serde_json::from_value::<PositionParams>(params.clone()).ok()?;
        let document = self.documents.get(&request.text_document.uri)?;
        let source = document.source(&request.text_document.uri);
        let offset = source.byte_offset_utf16(Utf16Position {
            line: request.position.line,
            character: request.position.character,
        })?;
        let target =
            super::super::requirement::target_path(&request.text_document.uri, &source, offset)?;
        let target_uri = super::path_to_uri(&target);
        if !target.is_file() && !self.documents.contains_key(&target_uri) {
            return None;
        }
        Some(json!({"uri": target_uri, "range": super::super::zero_range()}))
    }

    fn span_location(&self, root_uri: &str, span: Span) -> Option<(String, Value)> {
        let document = self.documents.get(root_uri)?;
        let source = document.source_for(span, root_uri)?;
        let uri = document.graph().map_or_else(
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

fn symbol_kind(kind: SymbolKind) -> usize {
    match kind {
        SymbolKind::Type => 5,
        SymbolKind::Function => 12,
        SymbolKind::Value | SymbolKind::Parameter | SymbolKind::ResultBinder => 13,
    }
}

/// Whether `new_name` lexes as a single identifier of the same kind as `old_name`, so that the edit keeps the program
/// parseable: a value name stays a value name, a type name a type name, and neither becomes a keyword.
fn renames_to_same_kind(old_name: &str, new_name: &str) -> bool {
    let kind = |text: &str| {
        let source = SourceFile::new(FileId::new(0), "rename", text.to_owned());
        match mal_syntax::lexer::lex(&source).ok()?.as_slice() {
            [identifier, eof]
                if eof.kind == TokenKind::Eof
                    && identifier.span.start() == 0
                    && identifier.span.end() == text.len()
                    && matches!(
                        identifier.kind,
                        TokenKind::ValueIdentifier | TokenKind::TypeIdentifier
                    ) =>
            {
                Some(identifier.kind.clone())
            }
            _ => None,
        }
    };
    kind(new_name).is_some_and(|new| kind(old_name) == Some(new))
}
