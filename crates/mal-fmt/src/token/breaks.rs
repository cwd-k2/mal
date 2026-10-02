//! Line breaks the source wrote inside expressions, kept where an operator may break the line.

use super::*;

impl Formatter<'_> {
    pub(super) fn preserve_source_break(&mut self, token_index: usize, kind: &TokenKind) {
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

        let follows_arrow = self
            .lexed
            .tokens
            .get(token_index.saturating_sub(1))
            .is_some_and(|token| matches!(token.kind, TokenKind::Arrow | TokenKind::FatArrow));
        if follows_arrow {
            self.start_expression_continuation();
            self.newline();
            return;
        }

        if matches!(self.previous, Previous::LeftBracket) {
            if let Some(layout) = self.brackets.last_mut()
                && !layout.multiline
            {
                layout.multiline = true;
                layout.indent_delta = 1;
                self.indent += layout.indent_delta;
            }
            self.newline();
            return;
        }
        if self.brackets.last().is_some_and(|layout| {
            layout.multiline && layout.parenthesis_depth == self.parenthesis_indents.len()
        }) && (matches!(self.previous, Previous::Comma)
            || matches!(kind, TokenKind::RightBracket))
        {
            self.newline();
            return;
        }

        let receiver_call = matches!(kind, TokenKind::Dot)
            && !matches!(
                self.lexed
                    .tokens
                    .get(token_index.saturating_sub(1))
                    .map(|token| &token.kind),
                Some(TokenKind::TypeIdentifier)
            );
        let binary_before = is_breakable_operator(kind) && self.previous.ends_expression();
        let binary_after = matches!(self.previous, Previous::Operator);
        let list_item = matches!(self.previous, Previous::LeftParen | Previous::Comma);
        let list_end = matches!(kind, TokenKind::RightParen | TokenKind::RightBracket);
        if receiver_call || binary_before || binary_after || list_item || list_end {
            self.newline();
            self.source_line_indent = Some(if list_item {
                self.parenthesis_indents
                    .last()
                    .map_or(self.indent + 1, |indent| indent + 1)
            } else if matches!(kind, TokenKind::RightParen) {
                self.parenthesis_indents
                    .last()
                    .copied()
                    .unwrap_or(self.indent)
            } else {
                self.indent + usize::from(!list_end)
            });
        }
    }
}

fn is_breakable_operator(kind: &TokenKind) -> bool {
    matches!(
        kind,
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
            | TokenKind::ShiftRight
    )
}
