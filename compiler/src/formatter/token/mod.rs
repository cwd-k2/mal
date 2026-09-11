use crate::lexer::TokenKind;

use super::Formatter;

mod control;

pub(super) use self::control::IfStage;

pub(super) struct BracketLayout {
    multiline: bool,
    indent_delta: usize,
    align_sum_continuations: Option<bool>,
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
        if self.blocks.omit[token_index] {
            return;
        }
        self.finish_completed_control();
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
        }
        if self.should_preserve_blank_line()
            && !matches!(kind, TokenKind::RightBrace | TokenKind::Else)
        {
            self.blank_line();
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
        self.preserve_source_break(kind);
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
                self.pending_newline = true;
                self.previous = Previous::Semicolon;
            }
            TokenKind::Comma => {
                self.trim_space();
                self.write(text);
                self.space();
                self.previous = Previous::Comma;
            }
            TokenKind::LeftParen => {
                self.write_left_paren(text);
            }
            TokenKind::RightParen => {
                self.write_right_paren(text);
            }
            TokenKind::LeftBracket => {
                self.write(text);
                let alignment = self.controls.sum_continuation_alignment(token_index);
                let indent_delta = alignment.map_or(0, |aligned| usize::from(!aligned));
                self.indent += indent_delta;
                self.brackets.push(BracketLayout {
                    multiline: alignment.is_some(),
                    indent_delta,
                    align_sum_continuations: alignment,
                });
                self.previous = Previous::LeftBracket;
            }
            TokenKind::RightBracket => {
                self.trim_space();
                self.write(text);
                self.previous = Previous::RightBracket;
            }
            TokenKind::Minus if !self.previous.ends_expression() => {
                if matches!(self.previous, Previous::Keyword) {
                    self.space();
                }
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
            TokenKind::Bind => {
                self.space();
                self.write(text);
                self.space();
                self.previous = Previous::Bind;
            }
            TokenKind::DoubleColon
            | TokenKind::Arrow
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
            TokenKind::Else if matches!(self.ifs.last(), Some(IfStage::AwaitElse(_))) => {
                self.write_else(text);
            }
            TokenKind::If => {
                self.write_if(token_index, text);
            }
            TokenKind::Require
            | TokenKind::Extern
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

    fn preserve_source_break(&mut self, kind: &TokenKind) {
        if !self.source_break {
            return;
        }
        if matches!(kind, TokenKind::Bind) || matches!(self.previous, Previous::Bind) {
            if self.binding_continuations.last() != Some(&self.brace_depth) {
                self.indent += 1;
                self.binding_continuations.push(self.brace_depth);
            }
            self.newline();
            return;
        }
        if self.line_start {
            return;
        }

        if matches!(self.previous, Previous::LeftBracket) {
            if let Some(layout) = self.brackets.last_mut()
                && !layout.multiline
            {
                layout.multiline = true;
                layout.indent_delta = usize::from(layout.align_sum_continuations != Some(true));
                self.indent += layout.indent_delta;
            }
            self.newline();
            return;
        }
        if self.brackets.last().is_some_and(|layout| layout.multiline)
            && (matches!(self.previous, Previous::Comma) || matches!(kind, TokenKind::RightBracket))
        {
            self.newline();
            return;
        }

        let binary_before = is_breakable_operator(kind) && self.previous.ends_expression();
        let binary_after = matches!(self.previous, Previous::Operator);
        let list_item = matches!(self.previous, Previous::LeftParen | Previous::Comma);
        let list_end = matches!(kind, TokenKind::RightParen | TokenKind::RightBracket);
        if binary_before || binary_after || list_item || list_end {
            self.newline();
            self.source_line_indent = Some(self.indent + usize::from(!list_end));
        }
    }
}

fn is_breakable_operator(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::DoubleColon
            | TokenKind::Arrow
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
            | TokenKind::ShiftRight
    )
}
