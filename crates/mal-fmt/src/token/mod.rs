//! Stateful token spacing and source-break preservation for the canonical formatter.

use mal_syntax::lexer::TokenKind;

use super::Formatter;

mod breaks;
mod control;

pub(super) use self::control::IfStage;

pub(super) struct BracketLayout {
    multiline: bool,
    indent_delta: usize,
    parenthesis_depth: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Previous {
    None,
    Word,
    Keyword,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Bind,
    Operator,
    Unary,
    Dot,
}

impl Previous {
    fn ends_expression(self) -> bool {
        matches!(
            self,
            Self::Word | Self::RightParen | Self::RightBracket | Self::RightBrace
        )
    }
}

impl Formatter<'_> {
    pub(super) fn write_token(&mut self, token_index: usize, kind: &TokenKind, text: &str) {
        self.finish_completed_controls(self.controls.completed_before(token_index));
        if self.blocks.omit[token_index] {
            return;
        }
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
        }
        if self.should_preserve_blank_line()
            && !matches!(kind, TokenKind::RightBrace | TokenKind::Else)
        {
            self.blank_line();
        }
        if matches!(
            kind,
            TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace
        ) {
            // A continuation started inside the closing delimiter cannot outlive it.
            self.end_expression_continuations();
        }
        if matches!(kind, TokenKind::RightBracket)
            && let Some(layout) = self.brackets.pop()
            && layout.multiline
        {
            self.indent = self.indent.saturating_sub(layout.indent_delta);
        }
        if self.controls.should_break_before_sum_token(token_index) {
            self.newline();
        }
        self.preserve_source_break(token_index, kind);
        if matches!(self.previous, Previous::Keyword) {
            self.space();
        }
        if self.generic_delimiters[token_index] {
            match kind {
                TokenKind::Less => {
                    self.trim_space();
                    self.write(text);
                    self.previous = Previous::Operator;
                    return;
                }
                TokenKind::Greater | TokenKind::ShiftRight => {
                    self.trim_space();
                    self.write(text);
                    self.previous = Previous::Word;
                    return;
                }
                _ => unreachable!("only generic angle delimiters are marked"),
            }
        }
        match kind {
            TokenKind::LeftBrace => {
                self.write_left_brace(token_index, text);
            }
            TokenKind::RightBrace => {
                self.write_right_brace(token_index, text);
            }
            TokenKind::Semicolon => {
                self.trim_space();
                self.write(text);
                if self.binding_continuations.last() == Some(&self.brace_depth) {
                    self.binding_continuations.pop();
                    self.indent = self.indent.saturating_sub(1);
                }
                self.end_expression_continuations();
                self.pending_newline = true;
                self.previous = Previous::Semicolon;
            }
            TokenKind::Comma => {
                self.end_expression_continuations();
                self.trim_space();
                self.write(text);
                self.space();
                self.previous = Previous::Comma;
            }
            TokenKind::LeftParen => {
                self.write_left_paren(text);
            }
            TokenKind::RightParen => {
                self.write_right_paren(token_index, text);
            }
            TokenKind::LeftBracket => {
                self.write(text);
                let sum_continuations = self.controls.is_sum_continuation(token_index);
                let indent_delta = usize::from(sum_continuations);
                self.indent += indent_delta;
                self.brackets.push(BracketLayout {
                    multiline: sum_continuations,
                    indent_delta,
                    parenthesis_depth: self.parenthesis_indents.len(),
                });
                self.previous = Previous::LeftBracket;
            }
            TokenKind::RightBracket => {
                self.trim_space();
                self.write(text);
                self.previous = Previous::RightBracket;
            }
            TokenKind::Minus if !self.previous.ends_expression() => {
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::Bang | TokenKind::Tilde => {
                if self.previous.ends_expression() {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::Dot => {
                self.trim_space();
                self.write(text);
                self.previous = Previous::Dot;
            }
            TokenKind::Hash if !self.previous.ends_expression() => {
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::Star if !self.previous.ends_expression() => {
                self.write(text);
                self.previous = Previous::Unary;
            }
            TokenKind::Bind => {
                self.space();
                self.write(text);
                self.space();
                self.previous = Previous::Bind;
            }
            TokenKind::DoubleColon
            | TokenKind::Arrow
            | TokenKind::FatArrow
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Ampersand
            | TokenKind::AmpersandAmpersand
            | TokenKind::Pipe
            | TokenKind::PipePipe
            | TokenKind::Caret
            | TokenKind::Hash
            | TokenKind::ShiftLeft
            | TokenKind::ShiftRight => {
                self.space();
                self.write(text);
                self.space();
                self.previous = Previous::Operator;
            }
            TokenKind::Then if matches!(self.ifs.last(), Some(IfStage::Condition(_))) => {
                self.write_then(text);
            }
            TokenKind::Else
                if matches!(
                    self.ifs.last(),
                    Some(IfStage::AwaitElse(_) | IfStage::ThenKeyword(_))
                ) =>
            {
                self.write_else(text);
            }
            TokenKind::If => {
                self.write_if(token_index, text);
            }
            TokenKind::Require
            | TokenKind::Extern
            | TokenKind::Opaque
            | TokenKind::When
            | TokenKind::Then
            | TokenKind::Else => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Keyword;
            }
            TokenKind::TypeIdentifier
            | TokenKind::ValueIdentifier
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::Byte(_)
            | TokenKind::Symbol(_)
            | TokenKind::Underscore => {
                if matches!(
                    self.previous,
                    Previous::Word
                        | Previous::Keyword
                        | Previous::RightParen
                        | Previous::RightBracket
                        | Previous::RightBrace
                ) {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Word;
            }
            TokenKind::Eof => unreachable!("EOF has no lossless lexeme"),
        }
    }
}
