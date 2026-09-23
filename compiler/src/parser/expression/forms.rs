use crate::ast::{Expression, Node};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    pub(super) fn at_conversion_suffix(&self) -> bool {
        if !self.at(&TokenKind::Dot) {
            return false;
        }
        let Some(token) = self.tokens.get(self.position + 1) else {
            return false;
        };
        if token.kind != TokenKind::ValueIdentifier {
            return false;
        }
        matches!(
            &self.source.text()[token.span.start()..token.span.end()],
            "i8" | "i16"
                | "i32"
                | "i64"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "f32"
                | "f64"
                | "bytes"
                | "usize"
        ) && !self
            .tokens
            .get(self.position + 2)
            .is_some_and(|next| next.kind == TokenKind::LeftParen)
    }

    pub(super) fn parse_conversion_suffix(
        &mut self,
        value: Node<Expression>,
    ) -> Result<Node<Expression>, Diagnostic> {
        let start = value.span.start();
        self.expect(&TokenKind::Dot, "`.`")?;
        let type_name = self.parse_name(&TokenKind::ValueIdentifier, "a numeric conversion")?;
        let end = type_name.span.end();
        Ok(Node::new(
            Expression::Conversion {
                type_name,
                value: Box::new(value),
            },
            self.span(start, end),
        ))
    }

    pub(super) fn parse_parenthesized_expression(
        &mut self,
    ) -> Result<Node<Expression>, Diagnostic> {
        let left = self.expect(&TokenKind::LeftParen, "`(`")?;
        if let Some(right) = self.take(&TokenKind::RightParen) {
            return Ok(Node::new(
                Expression::Unit,
                self.join(left.span, right.span),
            ));
        }
        let first = self.parse_expression()?;
        if self.take(&TokenKind::Comma).is_none() {
            let right = self.expect(&TokenKind::RightParen, "`)`")?;
            return Ok(Node::new(
                Expression::Parenthesized(Box::new(first)),
                self.join(left.span, right.span),
            ));
        }
        let mut elements = vec![first, self.parse_expression()?];
        while self.take(&TokenKind::Comma).is_some() {
            elements.push(self.parse_expression()?);
        }
        let right = self.expect(&TokenKind::RightParen, "`)`")?;
        Ok(Node::new(
            Expression::Product(elements),
            self.join(left.span, right.span),
        ))
    }

    pub(super) fn parse_call(
        &mut self,
        callee: Node<Expression>,
    ) -> Result<Node<Expression>, Diagnostic> {
        let arguments = self.parse_arguments()?;
        let end = self.previous_span().end();
        let span = self.span(callee.span.start(), end);
        Ok(Node::new(
            Expression::Call {
                callee: Box::new(callee),
                arguments,
            },
            span,
        ))
    }

    pub(super) fn parse_receiver_call(
        &mut self,
        receiver: Node<Expression>,
    ) -> Result<Node<Expression>, Diagnostic> {
        let start = receiver.span.start();
        self.expect(&TokenKind::Dot, "`.`")?;
        let name = self.parse_name(&TokenKind::ValueIdentifier, "a function name after `.`")?;
        let callee_start = name.span.start();
        let callee = if self.at(&TokenKind::Less) {
            let arguments = self.parse_type_arguments()?;
            let end = self.previous_generic_close_span().end();
            Node::new(
                Expression::GenericName { name, arguments },
                self.span(callee_start, end),
            )
        } else {
            let callee_span = name.span;
            Node::new(Expression::Name(name), callee_span)
        };
        let mut arguments = self.parse_arguments()?;
        arguments.insert(0, receiver);
        let end = self.previous_span().end();
        Ok(Node::new(
            Expression::Call {
                callee: Box::new(callee),
                arguments,
            },
            self.span(start, end),
        ))
    }

    pub(super) fn parse_continuation_application(
        &mut self,
        value: Node<Expression>,
    ) -> Result<Node<Expression>, Diagnostic> {
        let start = value.span.start();
        self.expect(&TokenKind::LeftBracket, "`[`")?;
        let continuations = self.parse_continuations()?;
        let right = self.expect(&TokenKind::RightBracket, "`]`")?;
        Ok(Node::new(
            Expression::ContinuationApplication {
                value: Box::new(value),
                continuations,
            },
            self.span(start, right.span.end()),
        ))
    }

    pub(super) fn parse_unit_continuation_application(
        &mut self,
    ) -> Result<Node<Expression>, Diagnostic> {
        let left = self.expect(&TokenKind::LeftBracket, "`[`")?;
        let continuations = self.parse_continuations()?;
        let right = self.expect(&TokenKind::RightBracket, "`]`")?;
        Ok(Node::new(
            Expression::ContinuationApplication {
                value: Box::new(Node::new(Expression::Unit, left.span)),
                continuations,
            },
            self.join(left.span, right.span),
        ))
    }

    fn parse_continuations(&mut self) -> Result<Vec<Node<Expression>>, Diagnostic> {
        if self.at(&TokenKind::RightBracket) {
            return Ok(Vec::new());
        }
        let mut continuations = vec![self.parse_expression()?];
        while self.take(&TokenKind::Comma).is_some() {
            continuations.push(self.parse_expression()?);
        }
        Ok(continuations)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Node<Expression>>, Diagnostic> {
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let mut arguments = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            arguments.push(self.parse_expression()?);
            while self.take(&TokenKind::Comma).is_some() {
                arguments.push(self.parse_expression()?);
            }
        }
        self.expect(&TokenKind::RightParen, "`)`")?;
        Ok(arguments)
    }
}
