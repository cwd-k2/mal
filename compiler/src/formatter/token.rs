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
    Bind,
    Operator,
    Unary,
    Backslash,
    CaptureClose,
}

pub(super) enum IfStage {
    Condition(bool),
    ThenKeyword(bool),
    ThenBranch(usize, bool),
    AwaitElse(bool),
    ElseKeyword(bool),
    ElseBranch(usize, bool),
    Finished(bool),
}

pub(super) enum CaseStage {
    Keyword(bool),
    Scrutinee(usize, bool),
    Arms(usize, bool),
    AwaitArmOrEnd(usize, bool),
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
        if let Some(CaseStage::AwaitArmOrEnd(arms_indent, continuation)) = self.cases.last() {
            if matches!(kind, TokenKind::LeftBracket) {
                let arms_indent = *arms_indent;
                let continuation = *continuation;
                *self.cases.last_mut().expect("matched case") =
                    CaseStage::Arms(arms_indent, continuation);
            } else {
                let continuation = *continuation;
                self.cases.pop();
                if continuation {
                    self.indent = self.indent.saturating_sub(1);
                }
            }
        }
        if let Some(IfStage::Finished(continuation)) = self.ifs.last() {
            let continuation = *continuation;
            self.ifs.pop();
            if continuation {
                self.indent = self.indent.saturating_sub(1);
            }
        }
        if self.pending_newline {
            self.newline();
            self.pending_newline = false;
        }
        self.preserve_source_break(kind);
        match kind {
            TokenKind::LeftBrace => {
                self.space();
                self.write(text);
                self.brace_depth += 1;
                if self.blocks.compact[token_index] {
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenKeyword(continuation) => {
                                *stage = IfStage::ThenBranch(self.indent, *continuation);
                            }
                            IfStage::ElseKeyword(continuation) => {
                                *stage = IfStage::ElseBranch(self.indent, *continuation);
                            }
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
                        IfStage::ThenKeyword(continuation) => {
                            *stage = IfStage::ThenBranch(self.indent, *continuation);
                        }
                        IfStage::ElseKeyword(continuation) => {
                            *stage = IfStage::ElseBranch(self.indent, *continuation);
                        }
                        _ => {}
                    }
                }
                self.newline();
                self.previous = Previous::LeftBrace;
            }
            TokenKind::RightBrace => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
                if self.blocks.compact[token_index] {
                    self.space();
                    self.write(text);
                    if let Some(stage) = self.ifs.last_mut() {
                        match stage {
                            IfStage::ThenBranch(branch_indent, continuation)
                                if *branch_indent == self.indent =>
                            {
                                *stage = IfStage::AwaitElse(*continuation);
                            }
                            IfStage::ElseBranch(branch_indent, continuation)
                                if *branch_indent == self.indent =>
                            {
                                *stage = IfStage::Finished(*continuation);
                            }
                            _ => {}
                        }
                    }
                    if let Some(CaseStage::Arms(arms_indent, continuation)) = self.cases.last()
                        && self.indent == *arms_indent
                    {
                        let arms_indent = *arms_indent;
                        let continuation = *continuation;
                        *self.cases.last_mut().expect("matched case arms") =
                            CaseStage::AwaitArmOrEnd(arms_indent, continuation);
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
                        IfStage::ThenBranch(branch_indent, continuation)
                            if *branch_indent == closing_indent =>
                        {
                            *stage = IfStage::AwaitElse(*continuation);
                        }
                        IfStage::ElseBranch(branch_indent, continuation)
                            if *branch_indent == closing_indent =>
                        {
                            *stage = IfStage::Finished(*continuation);
                        }
                        _ => {}
                    }
                }
                if let Some(CaseStage::Arms(arms_indent, continuation)) = self.cases.last()
                    && self.indent == *arms_indent
                {
                    let arms_indent = *arms_indent;
                    let continuation = *continuation;
                    *self.cases.last_mut().expect("matched case arms") =
                        CaseStage::AwaitArmOrEnd(arms_indent, continuation);
                }
                self.previous = Previous::RightBrace;
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
                if matches!(self.previous, Previous::Keyword) {
                    self.space();
                }
                self.write(text);
                self.paren_depth += 1;
                if let Some(CaseStage::Keyword(continuation)) = self.cases.last() {
                    let continuation = *continuation;
                    *self.cases.last_mut().expect("matched case keyword") =
                        CaseStage::Scrutinee(self.paren_depth, continuation);
                }
                self.previous = Previous::LeftParen;
            }
            TokenKind::RightParen => {
                self.trim_space();
                self.write(text);
                if matches!(
                    self.cases.last(),
                    Some(CaseStage::Scrutinee(depth, _)) if *depth == self.paren_depth
                ) {
                    let continuation = match self.cases.last().expect("matched case scrutinee") {
                        CaseStage::Scrutinee(_, continuation) => *continuation,
                        _ => unreachable!("matched case scrutinee"),
                    };
                    if continuation {
                        self.indent += 1;
                    }
                    *self.cases.last_mut().expect("matched case scrutinee") =
                        CaseStage::Arms(self.indent, continuation);
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
                self.newline();
                let continuation = match self.ifs.last().expect("matched condition") {
                    IfStage::Condition(continuation) => *continuation,
                    _ => unreachable!("matched condition"),
                };
                if continuation {
                    self.indent += 1;
                }
                self.write(text);
                *self.ifs.last_mut().expect("matched condition") =
                    IfStage::ThenKeyword(continuation);
                self.previous = Previous::Keyword;
            }
            TokenKind::Else if matches!(self.ifs.last(), Some(IfStage::AwaitElse(_))) => {
                self.newline();
                self.write(text);
                let continuation = match self.ifs.last().expect("matched then branch") {
                    IfStage::AwaitElse(continuation) => *continuation,
                    _ => unreachable!("matched then branch"),
                };
                *self.ifs.last_mut().expect("matched then branch") =
                    IfStage::ElseKeyword(continuation);
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
                self.ifs
                    .push(IfStage::Condition(!self.controls.is_aligned(token_index)));
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
                self.cases
                    .push(CaseStage::Keyword(!self.controls.is_aligned(token_index)));
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
