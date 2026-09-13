use std::collections::HashSet;

use crate::lexer::{Token, TokenKind};

pub(super) fn function_declarations(tokens: &[Token]) -> HashSet<usize> {
    let mut declarations = HashSet::new();
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LeftBrace => brace_depth += 1,
            TokenKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::ValueIdentifier
                if brace_depth == 0 && top_level_value_is_function(tokens, index) =>
            {
                declarations.insert(index);
            }
            _ => {}
        }
    }
    declarations
}

fn top_level_value_is_function(tokens: &[Token], name: usize) -> bool {
    let Some(next) = tokens.get(name + 1) else {
        return false;
    };
    if next.kind == TokenKind::Bind {
        return initializer_is_lambda(tokens, name + 2);
    }
    if next.kind != TokenKind::DoubleColon {
        return false;
    }

    let type_start = name + 2;
    let mut cursor = type_start;
    while let Some(token) = tokens.get(cursor) {
        match token.kind {
            TokenKind::Bind => {
                return type_is_function(&tokens[type_start..cursor])
                    || initializer_is_lambda(tokens, cursor + 1);
            }
            TokenKind::Semicolon | TokenKind::Eof => {
                return type_is_function(&tokens[type_start..cursor]);
            }
            _ => cursor += 1,
        }
    }
    false
}

fn type_is_function(mut tokens: &[Token]) -> bool {
    while enclosing_parentheses(tokens) {
        tokens = &tokens[1..tokens.len() - 1];
    }

    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut arrow = false;
    for token in tokens {
        match token.kind {
            TokenKind::LeftParen => parentheses += 1,
            TokenKind::RightParen => parentheses = parentheses.saturating_sub(1),
            TokenKind::LeftBracket => brackets += 1,
            TokenKind::RightBracket => brackets = brackets.saturating_sub(1),
            TokenKind::Comma if parentheses == 0 && brackets == 0 => return false,
            TokenKind::Arrow if parentheses == 0 && brackets == 0 => arrow = true,
            _ => {}
        }
    }
    arrow
}

fn enclosing_parentheses(tokens: &[Token]) -> bool {
    if !matches!(tokens.first(), Some(token) if token.kind == TokenKind::LeftParen)
        || !matches!(tokens.last(), Some(token) if token.kind == TokenKind::RightParen)
    {
        return false;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LeftParen => depth += 1,
            TokenKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index + 1 == tokens.len();
                }
            }
            _ => {}
        }
    }
    false
}

fn initializer_is_lambda(tokens: &[Token], start: usize) -> bool {
    if !matches!(tokens.get(start), Some(token) if token.kind == TokenKind::LeftParen) {
        return false;
    }
    let mut parentheses = 0usize;
    let mut cursor = start;
    while let Some(token) = tokens.get(cursor) {
        match token.kind {
            TokenKind::LeftParen => parentheses += 1,
            TokenKind::RightParen => {
                parentheses = parentheses.saturating_sub(1);
                if parentheses == 0 {
                    cursor += 1;
                    break;
                }
            }
            TokenKind::Semicolon | TokenKind::Eof => return false,
            _ => {}
        }
        cursor += 1;
    }
    if matches!(tokens.get(cursor), Some(token) if token.kind == TokenKind::LeftBracket) {
        while !matches!(
            tokens.get(cursor),
            None | Some(Token {
                kind: TokenKind::RightBracket,
                ..
            })
        ) {
            cursor += 1;
        }
        cursor += 1;
    }
    matches!(tokens.get(cursor), Some(token) if token.kind == TokenKind::LeftBrace)
}
