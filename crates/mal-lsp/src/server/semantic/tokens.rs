use mal_compiler::editor::SymbolKind;
use mal_compiler::source::{SourceFile, Span, Utf16Position};

pub(super) fn lexical(source: &SourceFile) -> Vec<usize> {
    let Ok(syntax) = mal_compiler::editor::analyze_syntax(source) else {
        return Vec::new();
    };
    encode(
        source,
        syntax
            .tokens()
            .iter()
            .map(|token| (token.span, token.kind, token.declaration)),
    )
}

pub(super) fn encode(
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
