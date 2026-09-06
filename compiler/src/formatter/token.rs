use crate::lexer::TokenKind;

use super::Formatter;

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
    Operator,
    Unary,
    Backslash,
    CaptureClose,
}

pub(super) enum IfStage {
    Condition,
    ThenKeyword,
    ThenBranch(usize),
    AwaitElse,
    ElseKeyword,
    ElseBranch(usize),
    Finished,
}

pub(super) enum CaseStage {
    Keyword,
    Scrutinee(usize),
    Arms(usize),
    AwaitArmOrEnd(usize),
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
        if let Some(CaseStage::AwaitArmOrEnd(arms_indent)) = self.cases.last() {
            if matches!(kind, TokenKind::LeftBracket) {
                let arms_indent = *arms_indent;
                *self.cases.last_mut().expect("matched case") = CaseStage::Arms(arms_indent);
            } else {
                self.cases.pop();
                self.indent = self.indent.saturating_sub(1);
            }
        }
        if matches!(self.ifs.last(), Some(IfStage::Finished)) {
            self.ifs.pop();
            self.indent = self.indent.saturating_sub(1);
        }
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
        }
        match kind {
            TokenKind::LeftBrace => {
                self.space();
                self.write(text);
                if self.blocks.compact[token_index] {
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenKeyword => *stage = IfStage::ThenBranch(self.indent),
                            IfStage::ElseKeyword => *stage = IfStage::ElseBranch(self.indent),
                            _ => {}
                        }
                    }
                    self.space();
                    self.previous = Previous::LeftBrace;
                    return;
                }
                self.indent += 1;
                if let Some(stage) = self.ifs.last_mut() {
                    match stage {
                        IfStage::ThenKeyword => *stage = IfStage::ThenBranch(self.indent),
                        IfStage::ElseKeyword => *stage = IfStage::ElseBranch(self.indent),
                        _ => {}
                    }
                }
                self.newline();
                self.previous = Previous::LeftBrace;
            }
            TokenKind::RightBrace => {
                if self.blocks.compact[token_index] {
                    self.space();
                    self.write(text);
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenBranch(branch_indent) if *branch_indent == self.indent => {
                                *stage = IfStage::AwaitElse;
                            }
                            IfStage::ElseBranch(branch_indent) if *branch_indent == self.indent => {
                                *stage = IfStage::Finished;
                            }
                            _ => {}
                        }
                    }
                    if let Some(CaseStage::Arms(arms_indent)) = self.cases.last()
                        && self.indent == *arms_indent
                    {
                        let arms_indent = *arms_indent;
                        *self.cases.last_mut().expect("matched case arms") =
                            CaseStage::AwaitArmOrEnd(arms_indent);
                    }
                    self.previous = Previous::RightBrace;
                    return;
                }
                if self.blocks.terminate[token_index] {
                    self.trim_space();
                    self.write(";");
                }
                let closing_indent = self.indent;
                self.indent = self.indent.saturating_sub(1);
                self.newline();
                self.write(text);
                if let Some(stage) = self.ifs.last_mut() {
                    match stage {
                        IfStage::ThenBranch(branch_indent) if *branch_indent == closing_indent => {
                            *stage = IfStage::AwaitElse;
                        }
                        IfStage::ElseBranch(branch_indent) if *branch_indent == closing_indent => {
                            *stage = IfStage::Finished;
                        }
                        _ => {}
                    }
                }
                if let Some(CaseStage::Arms(arms_indent)) = self.cases.last()
                    && self.indent == *arms_indent
                {
                    let arms_indent = *arms_indent;
                    *self.cases.last_mut().expect("matched case arms") =
                        CaseStage::AwaitArmOrEnd(arms_indent);
                }
                self.previous = Previous::RightBrace;
            }
            TokenKind::Semicolon => {
                self.trim_space();
                self.write(text);
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
                if matches!(self.previous, Previous::Keyword) {
                    self.space();
                }
                self.write(text);
                self.paren_depth += 1;
                if matches!(self.cases.last(), Some(CaseStage::Keyword)) {
                    *self.cases.last_mut().expect("matched case keyword") =
                        CaseStage::Scrutinee(self.paren_depth);
                }
                self.previous = Previous::LeftParen;
            }
            TokenKind::RightParen => {
                self.trim_space();
                self.write(text);
                if matches!(
                    self.cases.last(),
                    Some(CaseStage::Scrutinee(depth)) if *depth == self.paren_depth
                ) {
                    self.indent += 1;
                    *self.cases.last_mut().expect("matched case scrutinee") =
                        CaseStage::Arms(self.indent);
                }
                self.paren_depth = self.paren_depth.saturating_sub(1);
                self.previous = Previous::RightParen;
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
            TokenKind::DoubleColon
            | TokenKind::Bind
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
            TokenKind::Then if matches!(self.ifs.last(), Some(IfStage::Condition)) => {
                self.newline();
                self.indent += 1;
                self.write(text);
                *self.ifs.last_mut().expect("matched condition") = IfStage::ThenKeyword;
                self.previous = Previous::Keyword;
            }
            TokenKind::Else if matches!(self.ifs.last(), Some(IfStage::AwaitElse)) => {
                self.newline();
                self.write(text);
                *self.ifs.last_mut().expect("matched then branch") = IfStage::ElseKeyword;
                self.previous = Previous::Keyword;
            }
            TokenKind::If => {
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
                self.ifs.push(IfStage::Condition);
                self.previous = Previous::Keyword;
            }
            TokenKind::Case => {
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
                self.cases.push(CaseStage::Keyword);
                self.previous = Previous::Keyword;
            }
            TokenKind::Extern | TokenKind::Then | TokenKind::Else => {
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
