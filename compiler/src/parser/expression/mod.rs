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
                left = self.parse_receiver_call(left)?;
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
            return self.parse_type_leading_expression();
        }
        if self.at(&TokenKind::If) {
            return self.parse_if();
        }
        if self.at(&TokenKind::When) {
            return self.parse_when();
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
            _ => None,
        }
    }
}
