//! Bindings, their patterns, and the bodies they initialize with.

use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_binding(&mut self) -> Result<Binding, Diagnostic> {
        let pattern = self.parse_pattern()?;
        let annotation = if self.take(&TokenKind::DoubleColon).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&TokenKind::Bind, "`:=`")?;
        let value = self.parse_expression()?;
        Ok(Binding {
            pattern,
            annotation,
            value,
        })
    }

    pub(super) fn at_binding(&mut self) -> bool {
        let checkpoint = self.position;
        let is_binding = self.parse_pattern().is_ok()
            && (self.at(&TokenKind::DoubleColon) || self.at(&TokenKind::Bind));
        self.position = checkpoint;
        is_binding
    }

    pub(super) fn parse_pattern(&mut self) -> Result<Node<Pattern>, Diagnostic> {
        self.within_syntax_nesting(Self::parse_pattern_inner)
    }

    pub(super) fn parse_pattern_inner(&mut self) -> Result<Node<Pattern>, Diagnostic> {
        if self.at(&TokenKind::ValueIdentifier) {
            let name = self.parse_name(&TokenKind::ValueIdentifier, "a value name")?;
            let span = name.span;
            return Ok(Node::new(Pattern::Name(name), span));
        }
        if let Some(token) = self.take(&TokenKind::Underscore) {
            return Ok(Node::new(Pattern::Wildcard, token.span));
        }
        if let Some(left) = self.take(&TokenKind::LeftParen) {
            let first = self.parse_pattern()?;
            self.expect(&TokenKind::Comma, "`,` in a product pattern")?;
            let mut elements = vec![first, self.parse_pattern()?];
            while self.take(&TokenKind::Comma).is_some() {
                elements.push(self.parse_pattern()?);
            }
            let right = self.expect(&TokenKind::RightParen, "`)`")?;
            return Ok(Node::new(
                Pattern::Product(elements),
                self.join(left.span, right.span),
            ));
        }
        Err(self.expected("a binding pattern"))
    }

    pub(super) fn parse_lambda_body(&mut self) -> Result<LambdaBody, Diagnostic> {
        self.expect(&TokenKind::Arrow, "`->`")?;
        self.parse_expression_body()
    }

    /// A body is one expression. A body that is exactly a block keeps that block's items; a block followed by a suffix or
    /// operator is the leftmost operand of a larger expression, as anywhere else.
    pub(super) fn parse_expression_body(&mut self) -> Result<LambdaBody, Diagnostic> {
        let result = if self.at(&TokenKind::LeftBrace) {
            let block = self.parse_expression_block()?;
            let resume = self.position;
            let span = block.span;
            let continued =
                self.parse_suffixes_and_operators(Node::new(Expression::Block(block), span), 0)?;
            if self.position == resume {
                let Expression::Block(block) = continued.kind else {
                    unreachable!("an operand without suffixes or operators is returned unchanged");
                };
                return Ok(block);
            }
            continued
        } else {
            self.parse_expression()?
        };
        let span = result.span;
        Ok(LambdaBody {
            items: Vec::new(),
            result: Box::new(result),
            span,
        })
    }
}
