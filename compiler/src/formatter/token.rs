use crate::lexer::TokenKind;

use super::Formatter;

mod control;

pub(super) use self::control::{CaseStage, IfStage};

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
    Backslash,
    CaptureClose,
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
        self.finish_completed_control(kind);
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
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
                if matches!(self.previous, Previous::RightParen | Previous::RightBrace) {
                    self.newline();
                }
                self.write(text);
                self.previous = Previous::LeftBracket;
            }
            TokenKind::RightBracket => {
                self.trim_space();
                self.write(text);
                self.previous = Previous::RightBracket;
            }
            TokenKind::Backslash => {
                if self.previous.ends_expression() {
                    self.space();
                }
                self.write(text);
                self.previous = Previous::Backslash;
            }
            TokenKind::Less if matches!(self.previous, Previous::Backslash) => {
                self.write(text);
                self.in_capture = true;
                self.previous = Previous::LeftBracket;
            }
            TokenKind::Greater if self.in_capture => {
                self.trim_space();
                self.write(text);
                self.in_capture = false;
                self.previous = Previous::CaptureClose;
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
            TokenKind::At => {
                self.write(text);
                self.previous = Previous::Unary;
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
            TokenKind::Case => {
                self.write_case(token_index, text);
            }
            TokenKind::Require | TokenKind::Extern | TokenKind::Then | TokenKind::Else => {
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
