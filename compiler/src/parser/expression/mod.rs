use crate::ast::{BinaryOperator, Expression, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::Parser;

mod control;
mod forms;
mod lambda;

impl Parser<'_> {
    pub(super) fn parse_expression(&mut self) -> Result<Node<Expression>, Diagnostic> {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, minimum: u8) -> Result<Node<Expression>, Diagnostic> {
        self.within_syntax_nesting(|parser| parser.parse_expression_bp_inner(minimum))
    }

    fn parse_expression_bp_inner(&mut self, minimum: u8) -> Result<Node<Expression>, Diagnostic> {
        let mut left = self.parse_prefix()?;
        let mut non_associative = None;
        loop {
            if self.at(&TokenKind::LeftParen) && 23 >= minimum {
                left = self.parse_call(left)?;
                continue;
            }
            if self.at(&TokenKind::LeftBracket) && 23 >= minimum {
                left = self.parse_continuation_application(left)?;
                continue;
            }
            if self.at(&TokenKind::Dot) && 23 >= minimum {
                left = if self.at_conversion_suffix() {
                    self.parse_conversion_suffix(left)?
                } else {
                    self.parse_receiver_call(left)?
                };
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
        if self.at(&TokenKind::Hash) {
            let checkpoint = (
                self.position,
                self.pending_generic_closers,
                self.generic_close_span,
            );
            let start = self.advance().span.start();
            if self.starts_layout_shape()
                && let Ok(shape) = self.parse_layout_shape()
            {
                let end = shape.span.end();
                return Ok(Node::new(
                    Expression::StrideQuery(shape),
                    self.span(start, end),
                ));
            }
            (
                self.position,
                self.pending_generic_closers,
                self.generic_close_span,
            ) = checkpoint;
        }
        if let Some(operator) = self.unary_operator() {
            let token = self.advance().clone();
            let minimum = if operator == UnaryOperator::SymbolLength {
                22
            } else {
                21
            };
            let operand = self.parse_expression_bp(minimum)?;
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
            if self.at(&TokenKind::Less) {
                let checkpoint = (
                    self.position,
                    self.pending_generic_closers,
                    self.generic_close_span,
                );
                if let Ok(arguments) = self.parse_type_arguments() {
                    let end = self.previous_generic_close_span().end();
                    let start = name.span.start();
                    return Ok(Node::new(
                        Expression::GenericName { name, arguments },
                        self.span(start, end),
                    ));
                }
                (
                    self.position,
                    self.pending_generic_closers,
                    self.generic_close_span,
                ) = checkpoint;
            }
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
        if matches!(self.current().kind, TokenKind::Float(_)) {
            let token = self.advance().clone();
            let TokenKind::Float(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::Float(value), token.span));
        }
        if matches!(self.current().kind, TokenKind::Byte(_)) {
            let token = self.advance().clone();
            let TokenKind::Byte(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::Byte(value), token.span));
        }
        if matches!(self.current().kind, TokenKind::Symbol(_)) {
            let token = self.advance().clone();
            let TokenKind::Symbol(value) = token.kind else {
                unreachable!();
            };
            return Ok(Node::new(Expression::Symbol(value), token.span));
        }
        if self.at_result_block() {
            return self.parse_result_block();
        }
        if self.at(&TokenKind::LeftBracket) {
            return self.parse_unit_continuation_application();
        }
        if self.at_lambda() {
            return self.parse_lambda();
        }
        if self.at(&TokenKind::LeftParen) {
            return self.parse_parenthesized_expression();
        }
        if self.at(&TokenKind::TypeIdentifier) {
            return Err(self.error_here(
                "type names are not expressions",
                "use a value name or a closed postfix numeric conversion such as `.i32`",
            ));
        }
        if self.at(&TokenKind::If) {
            return self.parse_if();
        }
        if self.at(&TokenKind::When) {
            return self.parse_when();
        }
        if self.at(&TokenKind::LeftBrace) {
            return self.parse_block_expression();
        }
        Err(self.expected("an expression"))
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
            TokenKind::Hash => (BinaryOperator::SymbolAt, 21, true),
            _ => return None,
        })
    }

    fn unary_operator(&self) -> Option<UnaryOperator> {
        match self.current().kind {
            TokenKind::Minus => Some(UnaryOperator::Negate),
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Tilde => Some(UnaryOperator::BitwiseNot),
            TokenKind::Hash => Some(UnaryOperator::SymbolLength),
            TokenKind::Star => Some(UnaryOperator::Star),
            _ => None,
        }
    }
}
