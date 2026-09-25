use crate::ast::{Name, Node, TypeExpression};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;
use crate::source::Span;

use super::Parser;

/// Closing-bracket state of the generic lists being parsed. The lexer emits `>>` as one shift token; it closes two
/// lists only when it ends a list nested directly in another, and its second half then closes the enclosing list.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct GenericLists {
    depth: usize,
    /// The second half of a split `>>` and the token position after it, where only the enclosing close may follow.
    pending_close: Option<(Span, usize)>,
    last_close: Option<Span>,
}

impl GenericLists {
    /// Whether the enclosing list is already closed by the second half of a `>>`, so nothing may follow in it.
    pub(super) const fn close_pending(self) -> bool {
        self.pending_close.is_some()
    }
}

impl Parser<'_> {
    pub(super) fn parse_type_parameters(&mut self) -> Result<Vec<Name>, Diagnostic> {
        if !self.at(&TokenKind::Less) {
            return Ok(Vec::new());
        }
        self.parse_required_type_parameters()
    }

    pub(super) fn parse_required_type_parameters(&mut self) -> Result<Vec<Name>, Diagnostic> {
        self.within_generic_list(|parser| {
            let mut parameters =
                vec![parser.parse_name(&TokenKind::TypeIdentifier, "a type parameter")?];
            while parser.take(&TokenKind::Comma).is_some() {
                parameters.push(parser.parse_name(&TokenKind::TypeIdentifier, "a type parameter")?);
            }
            Ok(parameters)
        })
    }

    pub(super) fn parse_type_arguments(&mut self) -> Result<Vec<Node<TypeExpression>>, Diagnostic> {
        self.within_generic_list(|parser| {
            let mut arguments = vec![parser.parse_type()?];
            while !parser.generic.close_pending() && parser.take(&TokenKind::Comma).is_some() {
                arguments.push(parser.parse_type()?);
            }
            Ok(arguments)
        })
    }

    /// The span of the `>` that closed the list parsed last.
    pub(super) fn previous_generic_close_span(&self) -> Span {
        self.generic
            .last_close
            .expect("a generic close was just parsed")
    }

    fn within_generic_list<T>(
        &mut self,
        parse_items: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        self.expect(&TokenKind::Less, "`<`")?;
        self.generic.depth += 1;
        let items = parse_items(self).and_then(|items| {
            self.expect_generic_close()?;
            Ok(items)
        });
        self.generic.depth -= 1;
        items
    }

    fn expect_generic_close(&mut self) -> Result<(), Diagnostic> {
        if let Some((close, position)) = self.generic.pending_close.take() {
            if position != self.position {
                return Err(self.expected("`>` after type arguments"));
            }
            self.generic.last_close = Some(close);
            return Ok(());
        }
        if self.at(&TokenKind::Greater) {
            self.generic.last_close = Some(self.advance().span);
            return Ok(());
        }
        if self.at(&TokenKind::ShiftRight) && self.generic.depth > 1 {
            let span = self.advance().span;
            let middle = span.start() + 1;
            self.generic.last_close = Some(self.span(span.start(), middle));
            self.generic.pending_close = Some((self.span(middle, span.end()), self.position));
            return Ok(());
        }
        Err(self.expected("`>` after type arguments"))
    }
}
