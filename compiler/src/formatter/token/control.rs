use super::Previous;
use crate::formatter::Formatter;

pub(in crate::formatter) enum IfStage {
    Condition(bool),
    ThenKeyword(bool),
    ThenBranch(usize, bool),
    AwaitElse(bool),
    ElseKeyword(bool),
    ElseBranch(usize, bool),
    Finished(bool),
}

impl Formatter<'_> {
    pub(super) fn finish_completed_control(&mut self) {
        if let Some(IfStage::Finished(continuation)) = self.ifs.last() {
            let continuation = *continuation;
            self.ifs.pop();
            if continuation {
                self.indent = self.indent.saturating_sub(1);
            }
        }
    }

    pub(super) fn write_left_brace(&mut self, token_index: usize, text: &str) {
        self.space();
        self.write(text);
        self.brace_depth += 1;
        if self.blocks.compact[token_index] {
            self.enter_if_branch(self.indent);
            self.space();
            self.previous = Previous::LeftBrace;
            return;
        }
        self.indent += 1;
        self.enter_if_branch(self.indent);
        self.newline();
        self.previous = Previous::LeftBrace;
    }

    pub(super) fn write_right_brace(&mut self, token_index: usize, text: &str) {
        self.brace_depth = self.brace_depth.saturating_sub(1);
        if self.blocks.compact[token_index] {
            self.space();
            self.write(text);
            self.finish_if_branch(self.indent);
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
        self.finish_if_branch(closing_indent);
        self.previous = Previous::RightBrace;
    }

    pub(super) fn write_left_paren(&mut self, text: &str) {
        if matches!(self.previous, Previous::Keyword) {
            self.space();
        }
        self.write(text);
        self.paren_depth += 1;
        self.previous = Previous::LeftParen;
    }

    pub(super) fn write_right_paren(&mut self, text: &str) {
        self.trim_space();
        self.write(text);
        self.paren_depth = self.paren_depth.saturating_sub(1);
        self.previous = Previous::RightParen;
    }

    pub(super) fn write_then(&mut self, text: &str) {
        self.newline();
        let continuation = match self.ifs.last().expect("matched condition") {
            IfStage::Condition(continuation) => *continuation,
            _ => unreachable!("matched condition"),
        };
        if continuation {
            self.indent += 1;
        }
        self.write(text);
        *self.ifs.last_mut().expect("matched condition") = IfStage::ThenKeyword(continuation);
        self.previous = Previous::Keyword;
    }

    pub(super) fn write_else(&mut self, text: &str) {
        self.newline();
        self.write(text);
        let continuation = match self.ifs.last().expect("matched then branch") {
            IfStage::AwaitElse(continuation) => *continuation,
            _ => unreachable!("matched then branch"),
        };
        *self.ifs.last_mut().expect("matched then branch") = IfStage::ElseKeyword(continuation);
        self.previous = Previous::Keyword;
    }

    pub(super) fn write_if(&mut self, token_index: usize, text: &str) {
        self.space_before_control_keyword();
        self.write(text);
        self.ifs
            .push(IfStage::Condition(!self.controls.is_aligned(token_index)));
        self.previous = Previous::Keyword;
    }

    fn space_before_control_keyword(&mut self) {
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
    }

    fn enter_if_branch(&mut self, indent: usize) {
        if let Some(stage) = self.ifs.last_mut() {
            match stage {
                IfStage::ThenKeyword(continuation) => {
                    *stage = IfStage::ThenBranch(indent, *continuation);
                }
                IfStage::ElseKeyword(continuation) => {
                    *stage = IfStage::ElseBranch(indent, *continuation);
                }
                _ => {}
            }
        }
    }

    fn finish_if_branch(&mut self, closing_indent: usize) {
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
    }
}
