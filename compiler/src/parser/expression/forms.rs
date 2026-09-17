use crate::ast::{Expression, LayoutShape, Node, PlacementOperand};
use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;

use super::super::Parser;

impl Parser<'_> {
    pub(super) fn parse_placement(
        &mut self,
        value: Node<Expression>,
    ) -> Result<Node<Expression>, Diagnostic> {
        let start = value.span.start();
        self.expect(&TokenKind::At, "`@`")?;
        let checkpoint = (
            self.position,
            self.pending_generic_closers,
            self.generic_close_span,
        );
        let shape = self
            .starts_layout_shape()
            .then(|| self.parse_layout_shape())
            .transpose();
        let operand = if let Ok(Some(shape)) = shape {
            PlacementOperand::Shape(shape)
        } else {
            (
                self.position,
                self.pending_generic_closers,
                self.generic_close_span,
            ) = checkpoint;
            if self.at(&TokenKind::LeftParen) {
                PlacementOperand::Value(Box::new(self.parse_parenthesized_expression()?))
            } else if self.at(&TokenKind::ValueIdentifier)
                || matches!(self.current().kind, TokenKind::Integer(_))
            {
                PlacementOperand::Value(Box::new(self.parse_prefix()?))
            } else {
                return Err(self.expected("a layout shape or USize placement operand"));
            }
        };
        let end = match &operand {
            PlacementOperand::Shape(shape) => shape.span.end(),
            PlacementOperand::Value(value) => value.span.end(),
        };
        Ok(Node::new(
            Expression::Placement {
                value: Box::new(value),
                operand,
            },
            self.span(start, end),
        ))
    }

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

    pub(super) fn starts_layout_shape(&self) -> bool {
        match self.current().kind {
            TokenKind::ValueIdentifier => matches!(
                &self.source.text()[self.current_span().start()..self.current_span().end()],
                "unit"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "f32"
                    | "f64"
                    | "address"
                    | "bytesize"
                    | "usize"
                    | "bool"
            ),
            TokenKind::LeftParen | TokenKind::LeftBracket => true,
            _ => false,
        }
    }

    pub(super) fn parse_layout_shape(&mut self) -> Result<Node<LayoutShape>, Diagnostic> {
        let start = self.current_span().start();
        if self.at(&TokenKind::ValueIdentifier) {
            let token = self.advance();
            let text = &self.source.text()[token.span.start()..token.span.end()];
            let kind = match text {
                "unit" => LayoutShape::Unit,
                "i8" => LayoutShape::Int8,
                "i16" => LayoutShape::Int16,
                "i32" => LayoutShape::Int32,
                "i64" => LayoutShape::Int64,
                "u8" => LayoutShape::UInt8,
                "u16" => LayoutShape::UInt16,
                "u32" => LayoutShape::UInt32,
                "u64" => LayoutShape::UInt64,
                "f32" => LayoutShape::Float32,
                "f64" => LayoutShape::Float64,
                "address" => LayoutShape::Address,
                "bytesize" => LayoutShape::ByteSize,
                "usize" => LayoutShape::USize,
                "bool" => LayoutShape::Bool,
                _ => return Err(self.expected("a closed layout shape")),
            };
            return Ok(Node::new(kind, token.span));
        }
        let (close, product) = if self.take(&TokenKind::LeftParen).is_some() {
            (TokenKind::RightParen, true)
        } else if self.take(&TokenKind::LeftBracket).is_some() {
            (TokenKind::RightBracket, false)
        } else {
            return Err(self.expected("a closed layout shape"));
        };
        let first = self.parse_layout_shape()?;
        self.expect(&TokenKind::Comma, "`,` after the first shape member")?;
        let mut members = vec![first, self.parse_layout_shape()?];
        while self.take(&TokenKind::Comma).is_some() {
            members.push(self.parse_layout_shape()?);
        }
        let right = self.expect(&close, if product { "`)`" } else { "`]`" })?;
        Ok(Node::new(
            if product {
                LayoutShape::Product(members)
            } else {
                LayoutShape::Sum(members)
            },
            self.span(start, right.span.end()),
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
        let callee_span = name.span;
        let mut arguments = self.parse_arguments()?;
        arguments.insert(0, receiver);
        let end = self.previous_span().end();
        Ok(Node::new(
            Expression::Call {
                callee: Box::new(Node::new(Expression::Name(name), callee_span)),
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
        if self.at(&TokenKind::TypeIdentifier) {
            let type_name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
            let right = self.expect(&TokenKind::RightBracket, "`]` after a construction type")?;
            return Ok(Node::new(
                Expression::Conversion {
                    type_name,
                    value: Box::new(value),
                },
                self.span(start, right.span.end()),
            ));
        }
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
        Err(self.error_here(
            "expected a type application or qualified primitive",
            "use `T(value)`, `value[T]`, or `T.member`",
        ))
    }
}
