use crate::ast::{Program, TopItem};
use crate::lexer::{Lexed, LexemeKind, TokenKind};
use crate::source::SourceFile;

pub(super) struct BlockLayout {
    pub(super) compact: Vec<bool>,
    pub(super) omit: Vec<bool>,
    pub(super) terminate: Vec<bool>,
}

impl BlockLayout {
    pub(super) fn new(source: &SourceFile, lexed: &Lexed) -> Self {
        let mut matching_parentheses = vec![None; lexed.tokens.len()];
        let mut parentheses = Vec::new();
        for (index, token) in lexed.tokens.iter().enumerate() {
            match token.kind {
                TokenKind::LeftParen => parentheses.push(index),
                TokenKind::RightParen => {
                    if let Some(left) = parentheses.pop() {
                        matching_parentheses[left] = Some(index);
                        matching_parentheses[index] = Some(left);
                    }
                }
                _ => {}
            }
        }

        let mut compact = vec![false; lexed.tokens.len()];
        let mut omit = vec![false; lexed.tokens.len()];
        let mut terminate = vec![false; lexed.tokens.len()];
        let comments = lexed
            .lexemes
            .iter()
            .filter(|lexeme| matches!(lexeme.kind, LexemeKind::LineComment))
            .map(|lexeme| lexeme.span)
            .collect::<Vec<_>>();
        let mut blocks = Vec::<OpenBlock>::new();
        for (index, token) in lexed.tokens.iter().enumerate() {
            if matches!(token.kind, TokenKind::LeftBrace) {
                if let Some(parent) = blocks.last_mut() {
                    parent.has_nested = true;
                }
                blocks.push(OpenBlock {
                    left: index,
                    has_nested: false,
                    semicolons: Vec::new(),
                });
                continue;
            }
            if matches!(token.kind, TokenKind::Semicolon) {
                if let Some(block) = blocks.last_mut() {
                    block.semicolons.push(index);
                }
                continue;
            }
            if !matches!(token.kind, TokenKind::RightBrace) {
                continue;
            }
            let Some(block) = blocks.pop() else {
                continue;
            };
            let left = block.left;
            let right = index;
            let first_comment = comments
                .partition_point(|comment| comment.start() <= lexed.tokens[left].span.end());
            let has_comment = comments
                .get(first_comment)
                .is_some_and(|comment| comment.end() < lexed.tokens[right].span.start());
            let last = right.checked_sub(1);
            let is_single_line = source
                .location(lexed.tokens[left].span.start())
                .zip(source.location(lexed.tokens[right].span.end()))
                .is_some_and(|(left, right)| left.line == right.line);
            let is_compact = !starts_when_block(&lexed.tokens, &matching_parentheses, left)
                && is_single_line
                && !block.has_nested
                && !has_comment
                && (block.semicolons.is_empty()
                    || block.semicolons.len() == 1 && block.semicolons.first().copied() == last);
            compact[left] = is_compact;
            compact[right] = is_compact;
            if is_compact {
                if let Some(index) = block.semicolons.first() {
                    omit[*index] = true;
                }
            } else if last
                .is_some_and(|index| !matches!(lexed.tokens[index].kind, TokenKind::Semicolon))
            {
                terminate[right] = true;
            }
        }

        Self {
            compact,
            omit,
            terminate,
        }
    }
}

struct OpenBlock {
    left: usize,
    has_nested: bool,
    semicolons: Vec<usize>,
}

fn starts_when_block(
    tokens: &[crate::lexer::Token],
    matching_parentheses: &[Option<usize>],
    left_brace: usize,
) -> bool {
    let Some(right_parenthesis) = left_brace.checked_sub(1) else {
        return false;
    };
    if !matches!(tokens[right_parenthesis].kind, TokenKind::RightParen) {
        return false;
    }
    matching_parentheses[right_parenthesis]
        .and_then(|left| left.checked_sub(1))
        .is_some_and(|before| matches!(tokens[before].kind, TokenKind::When))
}

pub(super) fn top_level_breaks(
    source: &SourceFile,
    lexed: &Lexed,
    program: &Program,
) -> Vec<usize> {
    let mut breaks = Vec::new();
    if let (Some(requirement), Some(item)) = (program.requirements.last(), program.items.first()) {
        breaks.push(item.span.start());
        if source.text()[requirement.span.end()..item.span.start()]
            .lines()
            .any(|line| line.trim_start().starts_with("//"))
        {
            breaks[0] = lexed
                .lexemes
                .iter()
                .find(|lexeme| {
                    matches!(lexeme.kind, LexemeKind::LineComment)
                        && lexeme.span.start() >= requirement.span.end()
                        && lexeme.span.end() <= item.span.start()
                })
                .map_or(item.span.start(), |comment| comment.span.start());
        }
    }
    let mut lexeme_index = 0;
    for items in program.items.windows(2) {
        let previous = &items[0];
        let next = &items[1];
        while lexed
            .lexemes
            .get(lexeme_index)
            .is_some_and(|lexeme| lexeme.span.end() <= previous.span.end())
        {
            lexeme_index += 1;
        }
        let between = &source.text()[previous.span.end()..next.span.start()];
        if has_blank_line(between)
            || is_function_binding(&previous.kind)
            || is_function_binding(&next.kind)
        {
            let previous_line = source
                .location(previous.span.end())
                .expect("parsed item span belongs to the source")
                .line;
            let anchor = lexed
                .lexemes
                .iter()
                .skip(lexeme_index)
                .take_while(|lexeme| lexeme.span.start() < next.span.start())
                .find(|lexeme| {
                    matches!(lexeme.kind, LexemeKind::LineComment)
                        && source
                            .location(lexeme.span.start())
                            .expect("lexeme span belongs to the source")
                            .line
                            > previous_line
                })
                .map_or(next.span.start(), |comment| comment.span.start());
            breaks.push(anchor);
        }
    }
    breaks
}

fn is_function_binding(item: &TopItem) -> bool {
    let expression = match item {
        TopItem::Binding(binding) => &binding.value.kind,
        TopItem::GenericBinding { value, .. } => &value.kind,
        _ => return false,
    };
    let mut expression = expression;
    while let crate::ast::Expression::Parenthesized(inner) = expression {
        expression = &inner.kind;
    }
    matches!(expression, crate::ast::Expression::Lambda(_))
}

fn has_blank_line(text: &str) -> bool {
    let mut saw_newline = false;
    let mut line_is_empty = true;
    for byte in text.bytes() {
        match byte {
            b'\n' if saw_newline && line_is_empty => return true,
            b'\n' => {
                saw_newline = true;
                line_is_empty = true;
            }
            b'\r' | b' ' | b'\t' if saw_newline => {}
            _ if saw_newline => line_is_empty = false,
            _ => {}
        }
    }
    false
}
