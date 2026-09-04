use std::mem::discriminant;

use crate::ast::{
    Binding, BodyItem, LambdaBody, Name, Node, Pattern, Program, TopItem, TypeExpression,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind, lex};
use crate::source::{SourceFile, Span};

mod expression;

pub fn parse(source: &SourceFile) -> Result<Program, Diagnostic> {
    let tokens = lex(source)?;
    parse_tokens(source, &tokens)
}

pub fn parse_tokens(source: &SourceFile, tokens: &[Token]) -> Result<Program, Diagnostic> {
    let source_end = Span::new(source.id(), source.text().len(), source.text().len());
    if tokens.iter().any(|token| !source.contains(token.span)) {
        return Err(Diagnostic::error("invalid token stream")
            .with_primary(source_end, "a token span does not belong to this source"));
    }
    if !tokens
        .last()
        .is_some_and(|token| matches!(token.kind, TokenKind::Eof))
    {
        return Err(Diagnostic::error("invalid token stream")
            .with_primary(source_end, "the parser requires a terminal EOF token"));
    }
    Parser::new(source, tokens).parse_program()
}

struct Parser<'a> {
    source: &'a SourceFile,
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a SourceFile, tokens: &'a [Token]) -> Self {
        Self {
            source,
            tokens,
            position: 0,
        }
    }

    fn parse_program(mut self) -> Result<Program, Diagnostic> {
        let start = self.current_span().start();
        let mut items = Vec::new();
        while !self.at(&TokenKind::Eof) {
            items.push(self.parse_top_item()?);
        }
        let end = self.current_span().end();
        Ok(Program {
            items,
            span: self.span(start, end),
        })
    }

    fn parse_top_item(&mut self) -> Result<Node<TopItem>, Diagnostic> {
        let start = self.current_span().start();
        let kind = if self.take(&TokenKind::Extern).is_some() {
            if self.at(&TokenKind::TypeIdentifier) {
                TopItem::ExternalType {
                    name: self.parse_name(&TokenKind::TypeIdentifier, "a type name")?,
                }
            } else {
                let name = self.parse_name(&TokenKind::ValueIdentifier, "an operation name")?;
                self.expect(&TokenKind::DoubleColon, "`::`")?;
                let ty = self.parse_type()?;
                TopItem::ExternalOperation { name, ty }
            }
        } else if self.at(&TokenKind::TypeIdentifier) {
            let name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
            self.expect(&TokenKind::DoubleColon, "`::`")?;
            let value = self.parse_type()?;
            TopItem::TypeAlias { name, value }
        } else {
            TopItem::Binding(self.parse_binding()?)
        };
        let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(Node::new(kind, self.span(start, semicolon.span.end())))
    }

    fn parse_type(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        let parameter = self.parse_atomic_type()?;
        if self.take(&TokenKind::Arrow).is_some() {
            let start = parameter.span.start();
            let result = self.parse_type()?;
            let span = self.span(start, result.span.end());
            Ok(Node::new(
                TypeExpression::Function {
                    parameter: Box::new(parameter),
                    result: Box::new(result),
                },
                span,
            ))
        } else {
            Ok(parameter)
        }
    }

    fn parse_atomic_type(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        if self.at(&TokenKind::TypeIdentifier) {
            let name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
            let span = name.span;
            return Ok(Node::new(TypeExpression::Named(name), span));
        }
        if let Some(left) = self.take(&TokenKind::LeftParen) {
            if let Some(right) = self.take(&TokenKind::RightParen) {
                return Ok(Node::new(
                    TypeExpression::Unit,
                    self.join(left.span, right.span),
                ));
            }
            let first = self.parse_type()?;
            if self.take(&TokenKind::Comma).is_none() {
                let right = self.expect(&TokenKind::RightParen, "`)`")?;
                return Ok(Node::new(
                    TypeExpression::Parenthesized(Box::new(first)),
                    self.join(left.span, right.span),
                ));
            }
            let mut elements = vec![first, self.parse_type()?];
            while self.take(&TokenKind::Comma).is_some() {
                elements.push(self.parse_type()?);
            }
            let right = self.expect(&TokenKind::RightParen, "`)`")?;
            return Ok(Node::new(
                TypeExpression::Product(elements),
                self.join(left.span, right.span),
            ));
        }
        if let Some(left) = self.take(&TokenKind::LeftBracket) {
            let first = self.parse_type()?;
            self.expect(&TokenKind::Comma, "`,` after the first sum member")?;
            let mut members = vec![first, self.parse_type()?];
            while self.take(&TokenKind::Comma).is_some() {
                members.push(self.parse_type()?);
            }
            let right = self.expect(&TokenKind::RightBracket, "`]`")?;
            return Ok(Node::new(
                TypeExpression::Sum(members),
                self.join(left.span, right.span),
            ));
        }
        Err(self.expected("a type"))
    }

    fn parse_binding(&mut self) -> Result<Binding, Diagnostic> {
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

    fn at_binding(&mut self) -> bool {
        let checkpoint = self.position;
        let is_binding = self.parse_pattern().is_ok()
            && (self.at(&TokenKind::DoubleColon) || self.at(&TokenKind::Bind));
        self.position = checkpoint;
        is_binding
    }

    fn parse_pattern(&mut self) -> Result<Node<Pattern>, Diagnostic> {
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

    fn parse_lambda_body(&mut self) -> Result<LambdaBody, Diagnostic> {
        let left = self.expect(&TokenKind::LeftBrace, "`{`")?;
        let mut items = Vec::new();
        while !self.at(&TokenKind::Return) {
            if self.at(&TokenKind::RightBrace) || self.at(&TokenKind::Eof) {
                return Err(self.expected("a terminal `return`"));
            }
            items.push(self.parse_body_item()?);
        }
        self.advance();
        let result = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon, "`;` after the return expression")?;
        let right = self.expect(&TokenKind::RightBrace, "`}`")?;
        Ok(LambdaBody {
            items,
            result: Box::new(result),
            span: self.join(left.span, right.span),
        })
    }

    fn parse_body_item(&mut self) -> Result<BodyItem, Diagnostic> {
        if self.at_binding() {
            let binding = self.parse_binding()?;
            let start = binding.pattern.span.start();
            let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
            return Ok(BodyItem::Binding(Node::new(
                binding,
                self.span(start, semicolon.span.end()),
            )));
        }
        let expression = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(BodyItem::Expression(expression))
    }

    fn parse_name(&mut self, kind: &TokenKind, expected: &str) -> Result<Name, Diagnostic> {
        let token = self.expect(kind, expected)?;
        Ok(Name {
            text: self.source.text()[token.span.start()..token.span.end()].into(),
            span: token.span,
        })
    }

    fn at(&self, kind: &TokenKind) -> bool {
        discriminant(&self.current().kind) == discriminant(kind)
    }

    fn take(&mut self, kind: &TokenKind) -> Option<&'a Token> {
        self.at(kind).then(|| self.advance())
    }

    fn expect(&mut self, kind: &TokenKind, expected: &str) -> Result<&'a Token, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(self.expected(expected))
        }
    }

    fn advance_if_integer(&mut self) -> Option<Token> {
        matches!(self.current().kind, TokenKind::Integer(_)).then(|| self.advance().clone())
    }

    fn advance(&mut self) -> &'a Token {
        let token = self.current();
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    fn current(&self) -> &'a Token {
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .expect("parser requires an EOF-terminated token stream")
    }

    fn current_span(&self) -> Span {
        self.current().span
    }

    fn previous_span(&self) -> Span {
        self.tokens[self.position.saturating_sub(1)].span
    }

    fn expected(&self, expected: &str) -> Diagnostic {
        self.error_here(
            format!("expected {expected}"),
            format!("expected {expected} here"),
        )
    }

    fn error_here(&self, message: impl Into<String>, label: impl Into<String>) -> Diagnostic {
        Diagnostic::error(message).with_primary(self.current_span(), label)
    }

    fn join(&self, start: Span, end: Span) -> Span {
        self.span(start.start(), end.end())
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.source.id(), start, end)
    }
}
