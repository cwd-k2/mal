use mal_syntax::lexer::{LexemeKind, lex_lossless};
use mal_syntax::source::{SourceFile, Span};

pub fn declaration_documentation(source: &SourceFile, declaration: Span) -> Option<String> {
    if declaration.file() != source.id() {
        return None;
    }
    let lexed = lex_lossless(source).ok()?;
    let mut cursor = lexed.lexemes.iter().position(|lexeme| {
        lexeme.kind == LexemeKind::Token && lexeme.span.start() == declaration.start()
    })?;
    let mut lines = Vec::new();

    while cursor >= 2 {
        let whitespace = lexed.lexemes[cursor - 1];
        let comment = lexed.lexemes[cursor - 2];
        if whitespace.kind != LexemeKind::Whitespace
            || line_break_count(&source.text()[whitespace.span.start()..whitespace.span.end()]) != 1
            || comment.kind != LexemeKind::LineComment
            || !is_standalone(source.text(), comment.span.start())
        {
            break;
        }
        let text = &source.text()[comment.span.start() + 2..comment.span.end()];
        lines.push(text.strip_prefix(' ').unwrap_or(text).trim_end());
        cursor -= 2;
    }

    lines.reverse();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn is_standalone(text: &str, comment_start: usize) -> bool {
    let line_start = text.as_bytes()[..comment_start]
        .iter()
        .rposition(|byte| matches!(byte, b'\r' | b'\n'))
        .map_or(0, |position| position + 1);
    text.as_bytes()[line_start..comment_start]
        .iter()
        .all(|byte| matches!(byte, b' ' | b'\t'))
}

fn line_break_count(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        count += usize::from(matches!(bytes[index], b'\r' | b'\n'));
        index += usize::from(bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n')) + 1;
    }
    count
}
