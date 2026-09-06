use crate::ast::{BodyItem, CaseArm, Expression, ExpressionBlock, Node};
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

    pub(super) fn parse_case(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::Case, "`case`")?.span.start();
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let scrutinee = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "`)`")?;
        let mut arms = Vec::new();
        while self.at(&TokenKind::LeftBracket) {
            arms.push(self.parse_case_arm()?);
        }
        if arms.is_empty() {
            return Err(self.expected("at least one case arm"));
        }
        let end = arms.last().expect("case has at least one arm").span.end();
        Ok(Node::new(
            Expression::Case {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            self.span(start, end),
        ))
    }

    fn parse_case_arm(&mut self) -> Result<CaseArm, Diagnostic> {
        let start = self.expect(&TokenKind::LeftBracket, "`[`")?.span.start();
        let index = self.parse_integer("a case variant index")?;
        self.expect(&TokenKind::RightBracket, "`]`")?;
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::RightParen, "`)`")?;
        let body = self.parse_expression_block()?;
        Ok(CaseArm {
            index,
            pattern,
            span: self.span(start, body.span.end()),
            body,
        })
    }
}
