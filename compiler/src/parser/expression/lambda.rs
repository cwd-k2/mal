use crate::ast::{Expression, Lambda, Node, Pattern};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    pub(super) fn parse_lambda(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.current().span.start();
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let parameter = self.parse_lambda_parameter()?.map(Box::new);
        self.expect(&TokenKind::RightParen, "`)`")?;
        let body = self.parse_lambda_body()?;
        let span = self.span(start, body.span.end());
        Ok(Node::new(
            Expression::Lambda(Lambda { parameter, body }),
            span,
        ))
    }

    fn parse_lambda_parameter(&mut self) -> Result<Option<Node<Pattern>>, Diagnostic> {
        if self.at(&TokenKind::RightParen) {
            return Ok(None);
        }
        let first = self.parse_pattern()?;
        if self.take(&TokenKind::Comma).is_none() {
            return Ok(Some(first));
        }
        let start = first.span.start();
        let mut elements = vec![first, self.parse_pattern()?];
        while self.take(&TokenKind::Comma).is_some() {
            elements.push(self.parse_pattern()?);
        }
        let end = elements
            .last()
            .expect("lambda product has elements")
            .span
            .end();
        Ok(Some(Node::new(
            Pattern::Product(elements),
            self.span(start, end),
        )))
    }

    pub(super) fn at_lambda(&self) -> bool {
        if !self.at(&TokenKind::LeftParen) {
            return false;
        }
        let mut depth = 0usize;
        for (offset, token) in self.tokens[self.position..].iter().enumerate() {
            match token.kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self
                            .tokens
                            .get(self.position + offset + 1)
                            .is_some_and(|next| next.kind == TokenKind::LeftBrace);
                    }
                }
                _ => {}
            }
        }
        false
    }
}
