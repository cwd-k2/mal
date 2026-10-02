//! The token cursor: position, expectation, spans, diagnostics, and the bound on syntax nesting.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_name(
        &mut self,
        kind: &TokenKind,
        expected: &str,
    ) -> Result<Name, Diagnostic> {
        let token = self.expect(kind, expected)?;
        Ok(Name {
            text: self.source.text()[token.span.start()..token.span.end()].into(),
            span: token.span,
        })
    }

    pub(super) fn at(&self, kind: &TokenKind) -> bool {
        discriminant(&self.current().kind) == discriminant(kind)
    }

    pub(super) fn take(&mut self, kind: &TokenKind) -> Option<&'a Token> {
        self.at(kind).then(|| self.advance())
    }

    pub(super) fn expect(
        &mut self,
        kind: &TokenKind,
        expected: &str,
    ) -> Result<&'a Token, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(self.expected(expected))
        }
    }

    pub(super) fn advance(&mut self) -> &'a Token {
        let token = self.current();
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    pub(super) fn current(&self) -> &'a Token {
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .expect("parser requires an EOF-terminated token stream")
    }

    pub(super) fn current_span(&self) -> Span {
        self.current().span
    }

    pub(super) fn previous_span(&self) -> Span {
        self.tokens[self.position.saturating_sub(1)].span
    }

    pub(super) fn expected(&self, expected: &str) -> Diagnostic {
        self.error_here(
            format!("expected {expected}"),
            format!("expected {expected} here"),
        )
    }

    pub(super) fn error_here(
        &self,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic::error(message).with_primary(self.current_span(), label)
    }

    pub(super) fn within_syntax_nesting<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        if self.nesting == MAX_SYNTAX_NESTING {
            return Err(self.error_here(
                "syntax nesting limit exceeded",
                format!("malc supports at most {MAX_SYNTAX_NESTING} levels"),
            ));
        }
        self.nesting += 1;
        let result = parse(self);
        self.nesting -= 1;
        result
    }

    pub(super) fn join(&self, start: Span, end: Span) -> Span {
        self.span(start.start(), end.end())
    }

    pub(super) fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.source.id(), start, end)
    }
}
