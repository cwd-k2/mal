use super::Previous;
use crate::Formatter;

#[derive(Clone, Copy)]
pub(crate) struct IfLayout {
    continuation: bool,
    inline: bool,
}

pub(crate) enum IfStage {
    Condition(IfLayout),
    ThenKeyword(IfLayout),
    ThenBranch(usize, IfLayout),
    AwaitElse(IfLayout),
    ElseKeyword(IfLayout),
    ElseBranch(usize, IfLayout),
    Finished(IfLayout),
}

impl Formatter<'_> {
    pub(super) fn finish_completed_controls(&mut self, count: usize) {
        for _ in 0..count {
            let Some(stage) = self.ifs.pop() else {
                break;
            };
            let layout = match stage {
                IfStage::Condition(layout)
                | IfStage::ThenKeyword(layout)
                | IfStage::ThenBranch(_, layout)
                | IfStage::AwaitElse(layout)
                | IfStage::ElseKeyword(layout)
                | IfStage::ElseBranch(_, layout)
                | IfStage::Finished(layout) => layout,
            };
            if layout.continuation && !layout.inline {
                self.indent = self.indent.saturating_sub(1);
            }
        }
    }

    pub(super) fn write_left_brace(&mut self, token_index: usize, text: &str) {
        self.space();
        self.write(text);
        self.brace_depth += 1;
        if self.blocks.compact[token_index] {
            self.block_indent_deltas.push(0);
            self.enter_if_branch(self.brace_depth);
            self.space();
            self.previous = Previous::LeftBrace;
            return;
        }
        let indent_delta = self.line_indent.saturating_sub(self.indent) + 1;
        self.indent += indent_delta;
        self.block_indent_deltas.push(indent_delta);
        self.enter_if_branch(self.brace_depth);
        self.newline();
        self.previous = Previous::LeftBrace;
    }

    pub(super) fn write_right_brace(&mut self, token_index: usize, text: &str) {
        self.brace_depth = self.brace_depth.saturating_sub(1);
        let closing_depth = self.brace_depth + 1;
        let indent_delta = self
            .block_indent_deltas
            .pop()
            .expect("right brace has a matching left brace");
        if self.blocks.compact[token_index] {
            self.space();
            self.write(text);
            self.finish_if_branch(closing_depth);
            self.previous = Previous::RightBrace;
            return;
        }
        if self.blocks.terminate[token_index] {
            self.trim_space();
            self.write(";");
        }
        let closing_indent = self.indent.saturating_sub(1);
        self.indent = self.indent.saturating_sub(indent_delta);
        self.newline();
        self.source_line_indent = Some(closing_indent);
        self.write(text);
        self.finish_if_branch(closing_depth);
        self.previous = Previous::RightBrace;
    }

    pub(super) fn write_left_paren(&mut self, text: &str) {
        if matches!(self.previous, Previous::Keyword) {
            self.space();
        }
        self.write(text);
        self.parenthesis_indents.push(self.line_indent);
        self.previous = Previous::LeftParen;
    }

    pub(super) fn write_right_paren(&mut self, text: &str) {
        self.trim_space();
        self.write(text);
        self.parenthesis_indents.pop();
        self.previous = Previous::RightParen;
    }

    pub(super) fn write_then(&mut self, text: &str) {
        let layout = match self.ifs.last().expect("matched condition") {
            IfStage::Condition(layout) => *layout,
            _ => unreachable!("matched condition"),
        };
        if layout.inline {
            self.space();
        } else {
            self.newline();
        }
        if layout.continuation && !layout.inline {
            self.indent += 1;
        }
        self.write(text);
        *self.ifs.last_mut().expect("matched condition") = IfStage::ThenKeyword(layout);
        self.previous = Previous::Keyword;
    }

    pub(super) fn write_else(&mut self, text: &str) {
        let layout = match self.ifs.last().expect("matched then branch") {
            IfStage::AwaitElse(layout) | IfStage::ThenKeyword(layout) => *layout,
            _ => unreachable!("matched then branch"),
        };
        if layout.inline {
            self.space();
        } else {
            self.newline();
        }
        self.write(text);
        *self.ifs.last_mut().expect("matched then branch") = IfStage::ElseKeyword(layout);
        self.previous = Previous::Keyword;
    }

    pub(super) fn write_if(&mut self, token_index: usize, text: &str) {
        self.space_before_control_keyword();
        self.write(text);
        self.ifs.push(IfStage::Condition(IfLayout {
            continuation: !self.controls.is_aligned(token_index),
            inline: self.controls.is_inline(token_index),
        }));
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

    fn enter_if_branch(&mut self, depth: usize) {
        if let Some(stage) = self.ifs.last_mut() {
            match stage {
                IfStage::ThenKeyword(layout) => {
                    *stage = IfStage::ThenBranch(depth, *layout);
                }
                IfStage::ElseKeyword(layout) => {
                    *stage = IfStage::ElseBranch(depth, *layout);
                }
                _ => {}
            }
        }
    }

    fn finish_if_branch(&mut self, closing_depth: usize) {
        if let Some(stage) = self.ifs.last_mut() {
            match stage {
                IfStage::ThenBranch(branch_depth, layout) if *branch_depth == closing_depth => {
                    *stage = IfStage::AwaitElse(*layout);
                }
                IfStage::ElseBranch(branch_depth, layout) if *branch_depth == closing_depth => {
                    *stage = IfStage::Finished(*layout);
                }
                _ => {}
            }
        }
    }
}
