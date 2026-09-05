use crate::ast::{Program, TopItem};
use crate::lexer::{Lexed, LexemeKind, TokenKind};
use crate::source::SourceFile;

pub(super) struct BlockLayout {
    pub(super) compact: Vec<bool>,
    pub(super) omit: Vec<bool>,
    pub(super) terminate: Vec<bool>,
}

impl BlockLayout {
    pub(super) fn new(lexed: &Lexed) -> Self {
        let mut matching = vec![None; lexed.tokens.len()];
        let mut stack = Vec::new();
        for (index, token) in lexed.tokens.iter().enumerate() {
            match token.kind {
                TokenKind::LeftBrace => stack.push(index),
                TokenKind::RightBrace => {
                    if let Some(left) = stack.pop() {
                        matching[left] = Some(index);
                        matching[index] = Some(left);
                    }
                }
                _ => {}
            }
        }

        let mut compact = vec![false; lexed.tokens.len()];
        let mut omit = vec![false; lexed.tokens.len()];
        let mut terminate = vec![false; lexed.tokens.len()];
        for left in 0..lexed.tokens.len() {
            let Some(right) = matching[left] else {
                continue;
            };
            if left > right {
                continue;
            }
            let has_nested_block = lexed.tokens[left + 1..right]
                .iter()
                .any(|token| matches!(token.kind, TokenKind::LeftBrace));
            let has_comment = lexed.lexemes.iter().any(|lexeme| {
                matches!(lexeme.kind, LexemeKind::LineComment)
                    && lexeme.span.start() > lexed.tokens[left].span.end()
                    && lexeme.span.end() < lexed.tokens[right].span.start()
            });
            let mut depth = 0_usize;
            let semicolons = lexed.tokens[left + 1..right]
                .iter()
                .enumerate()
                .filter_map(|(offset, token)| match token.kind {
                    TokenKind::LeftBrace => {
                        depth += 1;
                        None
                    }
                    TokenKind::RightBrace => {
                        depth = depth.saturating_sub(1);
                        None
                    }
                    TokenKind::Semicolon if depth == 0 => Some(left + 1 + offset),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let last = right.checked_sub(1);
            let is_compact = !has_nested_block
                && !has_comment
                && (semicolons.is_empty()
                    || semicolons.len() == 1 && semicolons.first().copied() == last);
            compact[left] = is_compact;
            compact[right] = is_compact;
            if is_compact {
                if let Some(index) = semicolons.first() {
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

pub(super) fn top_level_breaks(
    source: &SourceFile,
    lexed: &Lexed,
    program: &Program,
) -> Vec<usize> {
    let mut breaks = Vec::new();
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
        if !(is_declaration(&previous.kind) && is_declaration(&next.kind)) {
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

fn is_declaration(item: &TopItem) -> bool {
    !matches!(item, TopItem::Binding(_))
}
