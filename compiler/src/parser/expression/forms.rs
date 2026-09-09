use crate::ast::{Expression, Node};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
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

    pub(super) fn parse_external_call(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::Extern, "`extern`")?.span.start();
        let name = self.parse_name(&TokenKind::ValueIdentifier, "an external operation name")?;
        let arguments = self.parse_arguments()?;
        Ok(Node::new(
            Expression::ExternalCall { name, arguments },
            self.span(start, self.previous_span().end()),
        ))
    }

    pub(super) fn parse_type_leading_expression(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let type_name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
        let start = type_name.span.start();
        if self.take(&TokenKind::Dot).is_some() {
            let member = self.parse_name(
                &TokenKind::ValueIdentifier,
                "a predefined primitive name after `.`",
            )?;
            let end = member.span.end();
            return Ok(Node::new(
                Expression::TypeQualifiedPrimitive { type_name, member },
                self.span(start, end),
            ));
        }
        if self.take(&TokenKind::LeftParen).is_some() {
            let value = self.parse_expression()?;
            let right = self.expect(&TokenKind::RightParen, "`)`")?;
            return Ok(Node::new(
                Expression::Conversion {
                    type_name,
                    value: Box::new(value),
                },
                self.span(start, right.span.end()),
            ));
        }
        self.expect(&TokenKind::LeftBracket, "`[` after a sum type name")?;
        let index = self.parse_integer("a sum variant index")?;
        self.expect(&TokenKind::RightBracket, "`]`")?;
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let value = self.parse_expression()?;
        let right = self.expect(&TokenKind::RightParen, "`)`")?;
        Ok(Node::new(
            Expression::SumInjection {
                type_name,
                index,
                value: Box::new(value),
            },
            self.span(start, right.span.end()),
        ))
    }
}
