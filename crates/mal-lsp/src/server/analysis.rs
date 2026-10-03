use std::collections::HashMap;

use mal_syntax::source::SourceFile;
use serde_json::{Value, json};

use super::diagnostic::{graph_diagnostic, lsp_diagnostic};
use super::{
    AnalysisState, Document, PublishedDiagnostics, Server, publish_diagnostics, uri_to_path,
    zero_range,
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

    /// Marks stale the document at `changed` and every open document whose last analysis read that file. A document
    /// whose graph could not be loaded may come to depend on it, so it is marked stale too.
    pub(super) fn invalidate_analyses_reading(&mut self, changed: &str) {
        let path = uri_to_path(changed).map(|path| std::fs::canonicalize(&path).unwrap_or(path));
        for (uri, document) in &mut self.documents {
            let reads = uri == changed
                || match &document.analysis {
                    AnalysisState::Stale | AnalysisState::Failed { graph: None } => true,
                    AnalysisState::Ready { graph: None, .. } => false,
                    AnalysisState::Failed { graph: Some(graph) }
                    | AnalysisState::Ready {
                        graph: Some(graph), ..
                    } => path
                        .as_ref()
                        .is_none_or(|path| graph.files().iter().any(|file| file.path() == path)),
                };
            if reads {
                document.analysis = AnalysisState::Stale;
            }
        }
    }

    /// Publishes changed diagnostics for `primary`, when it is open, and then for every other stale document.
    pub(super) fn publish_workspace_diagnostics(
        &mut self,
        primary: Option<&str>,
        messages: &mut Vec<Value>,
    ) {
        if let Some(primary) = primary
            && self.documents.contains_key(primary)
            && let Some(message) = self.diagnostics(primary)
        {
            messages.push(message);
        }
        let mut stale = self
            .documents
            .iter()
            .filter(|(uri, document)| {
                Some(uri.as_str()) != primary && matches!(document.analysis, AnalysisState::Stale)
            })
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        stale.sort();
        for uri in stale {
            if let Some(message) = self.diagnostics(&uri) {
                messages.push(message);
            }
        }
    }

    /// Analyzes the source graph while preferring diagnostics in the open root document.
    ///
    /// If graph loading fails, single-file analysis gets the first chance to report a root syntax
    /// or type error. Only a valid root reports the load error. Diagnostics originating in a loaded
    /// dependency are anchored at the root's zero range because LSP publishes this result for the
    /// root URI.
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
        match mal_syntax::graph::load_with_overlays(&path, &document.text, &overlays) {
            Ok(graph) => match mal_frontend::analysis::analyze_graph_for_editor(&graph) {
                Ok(analysis) => {
                    let diagnostics = analysis
                        .check_diagnostic()
                        .cloned()
                        .or_else(|| analysis.specialization_error())
                        .map(|diagnostic| graph_diagnostic(&graph, diagnostic))
                        .into_iter()
                        .collect();
                    document.analysis = AnalysisState::Ready {
                        graph: Some(graph),
                        analysis: Box::new(analysis),
                        semantic: None,
                    };
                    diagnostics
                }
                Err(diagnostic) => {
                    let diagnostic = graph_diagnostic(&graph, diagnostic);
                    document.analysis = AnalysisState::Failed { graph: Some(graph) };
                    vec![diagnostic]
                }
            },
            Err(load_error) => {
                let source = document.source(uri);
                let diagnostics = match mal_frontend::analysis::analyze(&source) {
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

    pub(super) fn semantic(&mut self) -> Option<&mal_frontend::editor::SemanticDocument> {
        let AnalysisState::Ready {
            graph,
            analysis,
            semantic,
        } = &mut self.analysis
        else {
            return None;
        };
        if semantic.is_none() {
            *semantic = Some(Box::new(graph.as_ref().map_or_else(
                || mal_frontend::editor::from_editor_analysis_for_file(analysis, self.id),
                |graph| {
                    mal_frontend::editor::from_graph_editor_analysis(graph, analysis, graph.root())
                },
            )));
        }
        semantic.as_deref()
    }

    fn analyze_single(&mut self, uri: &str) -> Vec<Value> {
        let source = self.source(uri);
        match mal_frontend::analysis::analyze_for_editor(&source) {
            Ok(analysis) => {
                let diagnostics = analysis
                    .check_diagnostic()
                    .cloned()
                    .or_else(|| analysis.specialization_error())
                    .map(|diagnostic| lsp_diagnostic(&source, diagnostic))
                    .into_iter()
                    .collect();
                self.analysis = AnalysisState::Ready {
                    graph: None,
                    analysis: Box::new(analysis),
                    semantic: None,
                };
                diagnostics
            }
            Err(diagnostic) => {
                self.analysis = AnalysisState::Failed { graph: None };
                vec![lsp_diagnostic(&source, diagnostic)]
            }
        }
    }

    pub(super) fn source_for(
        &self,
        span: mal_syntax::source::Span,
        uri: &str,
    ) -> Option<SourceFile> {
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

    pub(super) fn graph(&self) -> Option<&mal_syntax::source::SourceGraph> {
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
