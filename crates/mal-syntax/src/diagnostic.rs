//! Structured diagnostics and source-aware rendering shared by compiler stages.

use std::fmt::Write;

use crate::source::{SourceProvider, Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The diagnostic class presented to users and tools.
pub enum Severity {
    /// A condition that prevents admission or artifact generation.
    Error,
}

impl Severity {
    const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// The source range that owns a diagnostic and its local explanation.
pub struct Label {
    /// The labeled source range.
    pub span: Span,
    /// Why this range is relevant to the diagnostic.
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A structured compiler diagnostic with at most one primary source label.
pub struct Diagnostic {
    /// The diagnostic class.
    pub severity: Severity,
    /// The source-independent summary.
    pub message: String,
    /// The source range that owns the error, when one is available.
    pub primary: Option<Label>,
    /// Additional source-independent context in presentation order.
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Starts an error without attaching a source location.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            primary: None,
            notes: Vec::new(),
        }
    }

    /// Attaches the source range that owns the error and its local explanation.
    pub fn with_primary(mut self, span: Span, message: impl Into<String>) -> Self {
        self.primary = Some(Label {
            span,
            message: message.into(),
        });
        self
    }

    /// Appends context that does not own another source range.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Renders the message, the source line of the primary label, and notes. A label whose file or span is not in `sources`
    /// is reported as an invalid span instead of panicking.
    pub fn render(&self, sources: &impl SourceProvider) -> String {
        let mut rendered = format!("{}: {}\n", self.severity.name(), self.message);
        if let Some(label) = &self.primary {
            render_label(&mut rendered, sources, label);
        }
        for note in &self.notes {
            let _ = writeln!(rendered, "note: {note}");
        }
        rendered
    }
}

fn render_label(rendered: &mut String, sources: &impl SourceProvider, label: &Label) {
    let Some(source) = sources.source(label.span.file()) else {
        let _ = writeln!(rendered, " --> <unknown source>:<invalid span>");
        return;
    };
    if !source.contains(label.span) {
        let _ = writeln!(rendered, " --> {}:<invalid span>", source.path().display());
        return;
    }

    let start = source
        .location(label.span.start())
        .expect("validated span start must have a location");
    let line = source
        .line(start.line)
        .expect("validated span must refer to an existing line");
    let end_on_line = label.span.end().min(line.start + line.text.len());
    let underline_width = source.text()[label.span.start()..end_on_line]
        .chars()
        .count()
        .max(1);
    // Tabs before the label are repeated so the caret lines up with the line above wherever the terminal puts tab stops.
    let padding = source.text()[line.start..label.span.start()]
        .chars()
        .map(|character| if character == '\t' { '\t' } else { ' ' })
        .collect::<String>();
    let gutter_width = start.line.to_string().len();
    let _ = writeln!(
        rendered,
        " --> {}:{}:{}",
        source.path().display(),
        start.line,
        start.column
    );
    let _ = writeln!(rendered, "{:gutter_width$} |", "");
    let _ = writeln!(rendered, "{} | {}", start.line, line.text);
    let _ = writeln!(
        rendered,
        "{:gutter_width$} | {}{} {}",
        "",
        padding,
        "^".repeat(underline_width),
        label.message
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};

    #[test]
    fn renders_a_primary_source_label() {
        let file = FileId::new(0);
        let source = SourceFile::new(file, "sample.mal", "first\nvalue := 12;\n".into());
        let diagnostic = Diagnostic::error("unexpected integer")
            .with_primary(Span::new(file, 15, 17), "expected Unit")
            .with_note("a lambda body has one result type");

        assert_eq!(
            diagnostic.render(&source),
            concat!(
                "error: unexpected integer\n",
                " --> sample.mal:2:10\n",
                "  |\n",
                "2 | value := 12;\n",
                "  |          ^^ expected Unit\n",
                "note: a lambda body has one result type\n",
            )
        );
    }

    #[test]
    fn repeats_tabs_before_the_caret() {
        let file = FileId::new(0);
        let source = SourceFile::new(file, "sample.mal", "\tvalue := 12;\n".into());
        let diagnostic =
            Diagnostic::error("unexpected integer").with_primary(Span::new(file, 10, 12), "here");

        assert!(
            diagnostic
                .render(&source)
                .contains("  | \t         ^^ here\n")
        );
    }

    #[test]
    fn empty_spans_have_a_single_caret() {
        let file = FileId::new(0);
        let source = SourceFile::new(file, "sample.mal", "abc".into());
        let diagnostic =
            Diagnostic::error("missing token").with_primary(Span::new(file, 3, 3), "expected `;`");

        assert!(diagnostic.render(&source).contains("|    ^ expected `;`"));
    }
}
