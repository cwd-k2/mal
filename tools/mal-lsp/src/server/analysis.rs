use std::collections::HashMap;

use malc::source::SourceFile;
use serde_json::{Value, json};

use super::{
    AnalysisState, Document, PublishedDiagnostics, Server, position, publish_diagnostics,
    uri_to_path, zero_range,
};

impl Server {
    pub(super) fn diagnostics(&mut self, uri: &str) -> Option<Value> {
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

    pub(super) fn invalidate_analyses(&mut self) {
        for document in self.documents.values_mut() {
            document.analysis = AnalysisState::Stale;
        }
    }

    pub(super) fn publish_workspace_diagnostics(
        &mut self,
        primary: &str,
        messages: &mut Vec<Value>,
    ) {
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
        let Some(path) = uri_to_path(uri) else {
            return document.analyze_single(uri);
        };
        match malc::driver::load_source_graph_with_overlays(&path, &document.text, &overlays) {
            Ok(graph) => match malc::pipeline::analyze_graph(&graph) {
                Ok(analysis) => {
                    document.analysis = AnalysisState::Ready {
                        graph: Some(graph),
                        analysis,
                        semantic: None,
                    };
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
                    document.analysis = AnalysisState::Failed { graph: Some(graph) };
                    vec![diagnostic]
                }
            },
            Err(load_error) => {
                let source = document.source(uri);
                let diagnostics = match malc::pipeline::analyze(&source) {
                    Err(diagnostic) => vec![lsp_diagnostic(&source, diagnostic)],
                    Ok(_) => vec![json!({
                        "range": zero_range(), "severity": 1, "source": "malc",
                        "message": load_error.to_string()
                    })],
                };
                document.analysis = AnalysisState::Failed { graph: None };
                diagnostics
            }
        }
    }

    pub(super) fn ensure_analyzed(&mut self, uri: &str) -> bool {
        if let Some(document) = self.documents.get(uri) {
            match &document.analysis {
                AnalysisState::Ready { .. } => return true,
                AnalysisState::Failed { .. } => return false,
                AnalysisState::Stale => {}
            }
        }
        self.analyze_document(uri);
        self.documents.get(uri).is_some_and(Document::has_analysis)
    }
}

impl Document {
    pub(super) fn source(&self, uri: &str) -> SourceFile {
        self.graph().map_or_else(
            || SourceFile::new(self.id, uri, self.text.clone()),
            |graph| SourceFile::new(graph.root(), graph.root_source().path(), self.text.clone()),
        )
    }

    pub(super) fn semantic(&mut self) -> Option<&malc::editor::SemanticDocument> {
        let AnalysisState::Ready {
            graph,
            analysis,
            semantic,
        } = &mut self.analysis
        else {
            return None;
        };
        if semantic.is_none() {
            *semantic = Some(graph.as_ref().map_or_else(
                || malc::editor::from_analysis_for_file(analysis, self.id),
                |graph| malc::editor::from_graph_analysis(graph, analysis, graph.root()),
            ));
        }
        semantic.as_ref()
    }

    fn analyze_single(&mut self, uri: &str) -> Vec<Value> {
        let source = self.source(uri);
        match malc::pipeline::analyze(&source) {
            Ok(analysis) => {
                self.analysis = AnalysisState::Ready {
                    graph: None,
                    analysis,
                    semantic: None,
                };
                Vec::new()
            }
            Err(diagnostic) => {
                self.analysis = AnalysisState::Failed { graph: None };
                vec![lsp_diagnostic(&source, diagnostic)]
            }
        }
    }

    pub(super) fn source_for(&self, span: malc::source::Span, uri: &str) -> Option<SourceFile> {
        if let Some(graph) = self.graph() {
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

    pub(super) fn graph(&self) -> Option<&malc::source::SourceGraph> {
        match &self.analysis {
            AnalysisState::Failed { graph } | AnalysisState::Ready { graph, .. } => graph.as_ref(),
            AnalysisState::Stale => None,
        }
    }

    pub(super) fn has_analysis(&self) -> bool {
        matches!(&self.analysis, AnalysisState::Ready { .. })
    }

    #[cfg(test)]
    pub(super) fn has_semantic(&self) -> bool {
        matches!(
            &self.analysis,
            AnalysisState::Ready {
                semantic: Some(_),
                ..
            }
        )
    }

    #[cfg(test)]
    pub(super) fn analysis_is_current(&self) -> bool {
        !matches!(&self.analysis, AnalysisState::Stale)
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
