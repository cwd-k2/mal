use mal_compiler::editor::{Occurrence, SymbolKind};
use mal_syntax::source::{SourceFile, Span};

pub(super) fn contents(
    source: &SourceFile,
    span: Span,
    ty: &str,
    occurrence: Option<&Occurrence>,
    documentation: Option<&str>,
    location: Option<&str>,
) -> String {
    let (declaration, label) = if let Some(occurrence) = occurrence {
        let declaration = if occurrence.kind == SymbolKind::Type && occurrence.name == ty {
            occurrence.name.clone()
        } else {
            format!("{} :: {ty}", occurrence.name)
        };
        let label = match occurrence.kind {
            SymbolKind::Type => "type",
            SymbolKind::Function => "function",
            SymbolKind::Parameter => "parameter",
            SymbolKind::Value => "value",
        };
        (declaration, Some(label))
    } else {
        let expression = &source.text()[span.start()..span.end()];
        (format!("{expression} :: {ty}"), None)
    };
    let mut contents = format!("```mal\n{declaration}\n```");
    if let Some(label) = label {
        append_section(&mut contents, label);
    }
    if let Some(documentation) = documentation {
        append_section(&mut contents, documentation);
    }
    if let Some(location) = location {
        append_section(
            &mut contents,
            &format!("Defined in {}", inline_code(location)),
        );
    }
    contents
}

fn append_section(contents: &mut String, section: &str) {
    contents.push_str("\n\n");
    contents.push_str(section);
}

fn inline_code(text: &str) -> String {
    if text.contains('`') {
        format!("`` {text} ``")
    } else {
        format!("`{text}`")
    }
}
