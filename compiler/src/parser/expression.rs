use crate::ast::{
    BinaryOperator, BodyItem, CaseArm, Expression, ExpressionBlock, Lambda, Node, Parameter,
    UnaryOperator,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{IntegerLiteral, TokenKind};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_expression(&mut self) -> Result<Node<Expression>, Diagnostic> {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, minimum: u8) -> Result<Node<Expression>, Diagnostic> {
        let mut left = self.parse_prefix()?;
        let mut non_associative = None;
        loop {
            if self.at(&TokenKind::LeftParen) && 23 >= minimum {
                left = self.parse_call(left)?;
                continue;
            }
            let Some((operator, precedence, is_non_associative)) = self.binary_operator() else {
                break;
            };
            if precedence < minimum {
                break;
            }
            if is_non_associative && non_associative == Some(precedence) {
                return Err(self.error_here(
                    "non-associative operator chain",
                    "parenthesize the intended comparison",
                ));
            }
            let operator_token = self.advance().clone();
            let right = self.parse_expression_bp(precedence + 1)?;
            let span = self.span(left.span.start(), right.span.end());
            left = Node::new(
                Expression::Binary {
                    operator: Node::new(operator, operator_token.span),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            );
            non_associative = is_non_associative.then_some(precedence);
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Node<Expression>, Diagnostic> {
        if let Some(operator) = self.unary_operator() {
            let token = self.advance().clone();
            let operand = self.parse_expression_bp(21)?;
            let span = self.span(token.span.start(), operand.span.end());
            return Ok(Node::new(
                Expression::Unary {
                    operator: Node::new(operator, token.span),
                    operand: Box::new(operand),
                },
                span,
            ));
        }
        if self.at(&TokenKind::ValueIdentifier) {
            let name = self.parse_name(&TokenKind::ValueIdentifier, "a value name")?;
            let span = name.span;
            return Ok(Node::new(Expression::Name(name), span));
        }
        if matches!(self.current().kind, TokenKind::Integer(_)) {
            let token = self.advance().clone();
            let TokenKind::Integer(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::Integer(value), token.span));
        }
        if matches!(self.current().kind, TokenKind::Byte(_)) {
            let token = self.advance().clone();
            let TokenKind::Byte(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::Byte(value), token.span));
        }
        if matches!(self.current().kind, TokenKind::String(_)) {
            let token = self.advance().clone();
            let TokenKind::String(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::String(value), token.span));
        }
        if self.at(&TokenKind::LeftParen) {
            return self.parse_parenthesized_expression();
        }
        if self.at(&TokenKind::Backslash) {
            return self.parse_lambda();
        }
        if self.at(&TokenKind::Extern) {
            return self.parse_external_call();
        }
        if self.at(&TokenKind::TypeIdentifier) {
            return self.parse_type_constructor();
        }
        if self.at(&TokenKind::If) {
            return self.parse_if();
        }
        if self.at(&TokenKind::Case) {
            return self.parse_case();
        }
        Err(self.expected("an expression"))
    }

    fn parse_parenthesized_expression(&mut self) -> Result<Node<Expression>, Diagnostic> {
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

    fn parse_call(&mut self, callee: Node<Expression>) -> Result<Node<Expression>, Diagnostic> {
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

    fn parse_external_call(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::Extern, "`extern`")?.span.start();
        let name = self.parse_name(&TokenKind::ValueIdentifier, "an external operation name")?;
        let arguments = self.parse_arguments()?;
        Ok(Node::new(
            Expression::ExternalCall { name, arguments },
            self.span(start, self.previous_span().end()),
        ))
    }

    fn parse_type_constructor(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let type_name = self.parse_name(&TokenKind::TypeIdentifier, "a sum type name")?;
        let start = type_name.span.start();
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

    fn parse_lambda(&mut self) -> Result<Node<Expression>, Diagnostic> {
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

    fn parse_if(&mut self) -> Result<Node<Expression>, Diagnostic> {
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

    fn parse_expression_block(&mut self) -> Result<ExpressionBlock, Diagnostic> {
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

    fn parse_case(&mut self) -> Result<Node<Expression>, Diagnostic> {
        let start = self.expect(&TokenKind::Case, "`case`")?.span.start();
        let scrutinee = self.parse_expression()?;
        self.expect(&TokenKind::LeftBrace, "`{`")?;
        let mut arms = Vec::new();
        while self.at(&TokenKind::LeftBracket) {
            arms.push(self.parse_case_arm()?);
        }
        if arms.is_empty() {
            return Err(self.expected("at least one case arm"));
        }
        let right = self.expect(&TokenKind::RightBrace, "`}`")?;
        Ok(Node::new(
            Expression::Case {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            self.span(start, right.span.end()),
        ))
    }

    fn parse_case_arm(&mut self) -> Result<CaseArm, Diagnostic> {
        let start = self.expect(&TokenKind::LeftBracket, "`[`")?.span.start();
        let index = self.parse_integer("a case variant index")?;
        self.expect(&TokenKind::RightBracket, "`]`")?;
        self.expect(&TokenKind::LeftParen, "`(`")?;
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::RightParen, "`)`")?;
        self.expect(&TokenKind::FatArrow, "`=>`")?;
        let value = self.parse_expression()?;
        let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(CaseArm {
            index,
            pattern,
            value,
            span: self.span(start, semicolon.span.end()),
        })
    }

    fn parse_integer(&mut self, expected: &str) -> Result<Node<IntegerLiteral>, Diagnostic> {
        let token = self
            .advance_if_integer()
            .ok_or_else(|| self.expected(expected))?;
        let TokenKind::Integer(value) = token.kind else {
            unreachable!();
        };
        Ok(Node::new(value, token.span))
    }

    fn binary_operator(&self) -> Option<(BinaryOperator, u8, bool)> {
        Some(match self.current().kind {
            TokenKind::PipePipe => (BinaryOperator::LogicalOr, 1, false),
            TokenKind::AmpersandAmpersand => (BinaryOperator::LogicalAnd, 3, false),
            TokenKind::Pipe => (BinaryOperator::BitwiseOr, 5, false),
            TokenKind::Caret => (BinaryOperator::BitwiseXor, 7, false),
            TokenKind::Ampersand => (BinaryOperator::BitwiseAnd, 9, false),
            TokenKind::EqualEqual => (BinaryOperator::Equal, 11, true),
            TokenKind::BangEqual => (BinaryOperator::NotEqual, 11, true),
            TokenKind::Less => (BinaryOperator::Less, 13, true),
            TokenKind::LessEqual => (BinaryOperator::LessEqual, 13, true),
            TokenKind::Greater => (BinaryOperator::Greater, 13, true),
            TokenKind::GreaterEqual => (BinaryOperator::GreaterEqual, 13, true),
            TokenKind::ShiftLeft => (BinaryOperator::ShiftLeft, 15, false),
            TokenKind::ShiftRight => (BinaryOperator::ShiftRight, 15, false),
            TokenKind::Plus => (BinaryOperator::Add, 17, false),
            TokenKind::Minus => (BinaryOperator::Subtract, 17, false),
            TokenKind::Star => (BinaryOperator::Multiply, 19, false),
            TokenKind::Slash => (BinaryOperator::Divide, 19, false),
            TokenKind::Percent => (BinaryOperator::Remainder, 19, false),
            _ => return None,
        })
    }

    fn unary_operator(&self) -> Option<UnaryOperator> {
        match self.current().kind {
            TokenKind::Minus => Some(UnaryOperator::Negate),
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Tilde => Some(UnaryOperator::BitwiseNot),
            _ => None,
        }
    }
}
