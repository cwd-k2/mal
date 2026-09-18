use std::collections::HashMap;

use malc::editor::{SemanticDocument, SymbolKind};
use malc::source::{SourceFile, Utf16Position};
use serde_json::{Value, json};

use super::{Position, Server};
use crate::server::span_range;

pub(super) fn completion_items(semantic: &SemanticDocument, functions_only: bool) -> Vec<Value> {
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

pub(super) fn requirement_completions(
    uri: &str,
    source: &SourceFile,
    position: Position,
) -> Option<Vec<Value>> {
    let candidates = super::super::requirement::completion_candidates(
        uri,
        source,
        Utf16Position {
            line: position.line,
            character: position.character,
        },
    )?;
    Some(
        candidates
            .into_iter()
            .map(|candidate| {
                json!({
                    "label": candidate.name,
                    "kind": if candidate.is_directory { 19 } else { 17 },
                    "detail": if candidate.is_directory { "directory" } else { "requirement" },
                    "textEdit": {
                        "range": span_range(source, candidate.replacement),
                        "newText": candidate.name
                    }
                })
            })
            .collect(),
    )
}

pub(super) fn lexical_function_completions(
    server: &Server,
    uri: &str,
    source: &SourceFile,
) -> Vec<Value> {
    let syntax = malc::editor::analyze_syntax(source).ok();
    let mut names = syntax
        .as_ref()
        .map(|syntax| syntax.functions().to_vec())
        .unwrap_or_default();
    let document = &server.documents[uri];
    let recovered_graph;
    let graph = if let Some(graph) = document.graph() {
        Some(graph)
    } else {
        recovered_graph = syntax.and_then(|syntax| {
            let path = super::super::uri_to_path(uri)?;
            let mut requirements = String::new();
            for requirement in syntax.requirements() {
                let span = requirement.declaration_span();
                requirements.push_str(&source.text()[span.start()..span.end()]);
                requirements.push('\n');
            }
            let overlays = server
                .documents
                .iter()
                .filter_map(|(uri, document)| {
                    Some((super::super::uri_to_path(uri)?, document.text.clone()))
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

pub(super) fn receiver_suffix_start(text: &str, offset: usize) -> Option<usize> {
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
            || (partial[0] == b'_' && partial.get(1).is_none_or(u8::is_ascii_lowercase)))
    {
        return None;
    }
    let dot = start.checked_sub(1)?;
    (bytes[dot] == b'.').then_some(dot)
}

fn completion_kind(kind: SymbolKind) -> usize {
    match kind {
        SymbolKind::Type => 7,
        SymbolKind::Function => 3,
        SymbolKind::Value | SymbolKind::Parameter => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::receiver_suffix_start;

    #[test]
    fn recognizes_only_value_identifier_receiver_suffixes() {
        for text in ["value.", "value.partial", "value._partial"] {
            assert_eq!(receiver_suffix_start(text, text.len()), Some(5));
        }
        for text in ["value.Partial", "value._Partial"] {
            assert_eq!(receiver_suffix_start(text, text.len()), None);
        }
    }
}
