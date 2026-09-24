use std::path::PathBuf;

use mal_compiler::source::{SourceFile, Span, Utf16Position};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{Server, TextDocumentIdentifier, error, path_to_uri, span_range, success, uri_to_path};

pub(super) struct CompletionCandidate {
    pub name: String,
    pub is_directory: bool,
    pub replacement: Span,
}

pub(super) fn completion_candidates(
    uri: &str,
    source: &SourceFile,
    position: Utf16Position,
) -> Option<Vec<CompletionCandidate>> {
    let offset = source.byte_offset_utf16(position)?;
    let context = path_context(source.text(), offset)?;
    let fragment = &source.text()[context.start..offset];
    if fragment.contains('\\') {
        return Some(Vec::new());
    }
    let source_path = uri_to_path(uri)?;
    let replacement_start = fragment
        .rfind('/')
        .map_or(context.start, |slash| context.start + slash + 1);
    let replacement = Span::new(source.id(), replacement_start, offset);
    Some(
        mal_compiler::driver::requirement_path_candidates(&source_path, fragment)
            .into_iter()
            .map(|candidate| CompletionCandidate {
                name: candidate.name,
                is_directory: candidate.is_directory,
                replacement,
            })
            .collect(),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentLinkParams {
    text_document: TextDocumentIdentifier,
}

impl Server {
    pub(super) fn document_links(&self, id: Value, params: Value) -> Value {
        let Ok(params) = serde_json::from_value::<DocumentLinkParams>(params) else {
            return error(id, -32602, "invalid document link parameters");
        };
        let uri = params.text_document.uri;
        let Some(document) = self.documents.get(&uri) else {
            return error(id, -32602, "document is not open");
        };
        let source = document.source(&uri);
        let Ok(syntax) = mal_compiler::editor::analyze_syntax(&source) else {
            return success(id, json!([]));
        };
        let links = syntax
            .requirements()
            .iter()
            .filter_map(|requirement| {
                let span = requirement.path_span();
                let path = std::str::from_utf8(requirement.path()).ok()?;
                let target = mal_compiler::driver::resolve_requirement_path(&uri_to_path(&uri)?, path)?;
                let target_uri = path_to_uri(&target);
                if !target.is_file() && !self.documents.contains_key(&target_uri) {
                    return None;
                }
                Some(json!({
                    "range": span_range(&source, span),
                    "target": target_uri
                }))
            })
            .collect::<Vec<_>>();
        success(id, json!(links))
    }
}

pub(super) fn target_path(uri: &str, source: &SourceFile, offset: usize) -> Option<PathBuf> {
    let syntax = mal_compiler::editor::analyze_syntax(source).ok()?;
    let requirement = syntax.requirements().iter().find(|requirement| {
        let span = requirement.path_span();
        span.start() <= offset && offset < span.end()
    })?;
    let path = std::str::from_utf8(requirement.path()).ok()?;
    mal_compiler::driver::resolve_requirement_path(&uri_to_path(uri)?, path)
}

#[derive(Clone, Copy)]
struct PathContext {
    start: usize,
}

fn path_context(text: &str, offset: usize) -> Option<PathContext> {
    if offset > text.len() || !text.is_char_boundary(offset) {
        return None;
    }
    let line_start = text[..offset]
        .rfind(['\n', '\r'])
        .map_or(0, |newline| newline + 1);
    let line_end = text[offset..]
        .find(['\n', '\r'])
        .map_or(text.len(), |newline| offset + newline);
    let bytes = text.as_bytes();
    let mut cursor = line_start;
    while cursor < line_end && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if !text[cursor..line_end].starts_with("require") {
        return None;
    }
    cursor += "require".len();
    if cursor >= line_end || !bytes[cursor].is_ascii_whitespace() {
        return None;
    }
    while cursor < line_end && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'\"') {
        return None;
    }
    let start = cursor + 1;
    let mut escaped = false;
    let mut end = None;
    cursor = start;
    while cursor < line_end {
        match (bytes[cursor], escaped) {
            (_, true) => escaped = false,
            (b'\\', false) => escaped = true,
            (b'\"', false) => {
                end = Some(cursor);
                break;
            }
            _ => {}
        }
        cursor += 1;
    }
    let content_end = end.unwrap_or(line_end);
    (offset >= start && offset <= content_end).then_some(PathContext { start })
}
