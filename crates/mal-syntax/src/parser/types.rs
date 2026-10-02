//! Type syntax.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        self.within_syntax_nesting(Self::parse_type_inner)
    }

    pub(super) fn parse_type_inner(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        let parameter = self.parse_atomic_type()?;
        if !self.generic.close_pending() && self.take(&TokenKind::Arrow).is_some() {
            let start = parameter.span.start();
            let result = self.parse_type()?;
            let span = self.span(start, result.span.end());
            Ok(Node::new(
                TypeExpression::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                },
                span,
            ))
        } else {
            Ok(parameter)
        }
    }

    pub(super) fn parse_atomic_type(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        if self.at(&TokenKind::TypeIdentifier) {
            let name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
            let start = name.span.start();
            if self.at(&TokenKind::Less) {
                let arguments = self.parse_type_arguments()?;
                let end = self.previous_generic_close_span().end();
                return Ok(Node::new(
                    TypeExpression::Application {
                        constructor: name,
                        arguments,
                    },
                    self.span(start, end),
                ));
            }
            let span = name.span;
            return Ok(Node::new(TypeExpression::Named(name), span));
        }
        if let Some(left) = self.take(&TokenKind::LeftParen) {
            if let Some(right) = self.take(&TokenKind::RightParen) {
                return Ok(Node::new(
                    TypeExpression::Unit,
                    self.join(left.span, right.span),
                ));
            }
            let first = self.parse_type()?;
            if self.take(&TokenKind::Comma).is_none() {
                let right = self.expect(&TokenKind::RightParen, "`)`")?;
                return Ok(Node::new(
                    TypeExpression::Parenthesized(Box::new(first)),
                    self.join(left.span, right.span),
                ));
            }
            let mut elements = vec![first, self.parse_type()?];
            while self.take(&TokenKind::Comma).is_some() {
                elements.push(self.parse_type()?);
            }
            let right = self.expect(&TokenKind::RightParen, "`)`")?;
            return Ok(Node::new(
                TypeExpression::Product(elements),
                self.join(left.span, right.span),
            ));
        }
        if let Some(left) = self.take(&TokenKind::LeftBracket) {
            if let Some(right) = self.take(&TokenKind::RightBracket) {
                return Ok(Node::new(
                    TypeExpression::Sum(Vec::new()),
                    self.join(left.span, right.span),
                ));
            }
            let first = self.parse_type()?;
            self.expect(&TokenKind::Comma, "`,` after the first sum member")?;
            let mut members = vec![first, self.parse_type()?];
            while self.take(&TokenKind::Comma).is_some() {
                members.push(self.parse_type()?);
            }
            let right = self.expect(&TokenKind::RightBracket, "`]`")?;
            return Ok(Node::new(
                TypeExpression::Sum(members),
                self.join(left.span, right.span),
            ));
        }
        Err(self.expected("a type"))
    }
}
