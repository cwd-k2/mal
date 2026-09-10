use crate::ast::{BodyItem, Expression, ExpressionBlock, Node};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    pub(super) fn parse_if(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::If, "`if`")?.span.start();
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "`)`")?;
        self.expect(&TokenKind::Then, "`then`")?;
        let then_branch = self.parse_expression_block()?;
        self.expect(&TokenKind::Else, "`else`")?;
        let else_branch = self.parse_expression_block()?;
        let span = self.span(start, else_branch.span.end());
        Ok(Node::new(
            Expression::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            span,
        ))
    }

    pub(in crate::parser) fn parse_expression_block(
        &mut self,
    ) -> Result<ExpressionBlock, Diagnostic> {
        let left = self.expect(&TokenKind::LeftBrace, "`{`")?;
        let mut items = Vec::new();
        loop {
            if self.at(&TokenKind::RightBrace) || self.at(&TokenKind::Eof) {
                return Err(self.expected("a block result expression"));
            }
            if self.at_binding() {
                let binding = self.parse_binding()?;
                let start = binding.pattern.span.start();
                let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
                items.push(BodyItem::Binding(Node::new(
                    binding,
                    self.span(start, semicolon.span.end()),
                )));
                continue;
            }
            let expression = self.parse_expression()?;
            if self.take(&TokenKind::Semicolon).is_some() {
                if self.at(&TokenKind::RightBrace) {
                    let right = self.advance();
                    return Ok(ExpressionBlock {
                        items,
                        result: Box::new(expression),
                        span: self.join(left.span, right.span),
                    });
                }
                items.push(BodyItem::Expression(expression));
                continue;
            }
            let right = self.expect(&TokenKind::RightBrace, "`}`")?;
            return Ok(ExpressionBlock {
                items,
                result: Box::new(expression),
                span: self.join(left.span, right.span),
            });
        }
    }
}
