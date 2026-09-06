use crate::ast::{Expression, Lambda, Node, Parameter};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    pub(super) fn parse_lambda(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::Backslash, "`\\`")?.span.start();
        let mut captures = Vec::new();
        if self.take(&TokenKind::Less).is_some() {
            captures.push(self.parse_name(&TokenKind::ValueIdentifier, "a captured value name")?);
            while self.take(&TokenKind::Comma).is_some() {
                captures
                    .push(self.parse_name(&TokenKind::ValueIdentifier, "a captured value name")?);
            }
            self.expect(&TokenKind::Greater, "`>`")?;
        }
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let mut parameters = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            parameters.push(self.parse_parameter()?);
            while self.take(&TokenKind::Comma).is_some() {
                parameters.push(self.parse_parameter()?);
            }
        }
        self.expect(&TokenKind::RightParen, "`)`")?;
        let body = self.parse_lambda_body()?;
        let span = self.span(start, body.span.end());
        Ok(Node::new(
            Expression::Lambda(Lambda {
                captures,
                parameters,
                body,
            }),
            span,
        ))
    }

    fn parse_parameter(&mut self) -> Result<Parameter, Diagnostic> {
        let name = self.parse_name(&TokenKind::ValueIdentifier, "a parameter name")?;
        let start = name.span.start();
        self.expect(&TokenKind::DoubleColon, "`::`")?;
        let ty = self.parse_type()?;
        Ok(Parameter {
            name,
            span: self.span(start, ty.span.end()),
            ty,
        })
    }
}
