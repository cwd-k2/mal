//! Conversion of compiler diagnostics to LSP diagnostics.

use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{SourceFile, SourceGraph};
use serde_json::{Value, json};

use super::{position, zero_range};

pub(super) fn lsp_diagnostic(source: &SourceFile, diagnostic: Diagnostic) -> Value {
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

/// Converts a graph diagnostic for the root's URI: at its own range in the root, otherwise at the root's zero range.
pub(super) fn graph_diagnostic(graph: &SourceGraph, diagnostic: Diagnostic) -> Value {
    let source = diagnostic
        .primary
        .as_ref()
        .and_then(|label| graph.source(label.span.file()))
        .unwrap_or_else(|| graph.root_source());
    if source.id() == graph.root() {
        lsp_diagnostic(source, diagnostic)
    } else {
        lsp_diagnostic_at_root(source, diagnostic)
    }
}

fn lsp_diagnostic_at_root(source: &SourceFile, diagnostic: Diagnostic) -> Value {
    let mut value = lsp_diagnostic(source, diagnostic);
    value["range"] = zero_range();
    if let Some(message) = value["message"].as_str() {
        value["message"] = Value::String(format!("{}: {message}", source.path().display()));
    }
    value
}
