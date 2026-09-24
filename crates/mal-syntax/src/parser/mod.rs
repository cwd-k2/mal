use std::mem::discriminant;

use crate::ast::{
    Binding, LambdaBody, Name, Node, Pattern, Program, Requirement, TopItem, TypeExpression,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind, lex};
use crate::source::{SourceFile, Span};

mod expression;

const MAX_SYNTAX_NESTING: usize = 64;

/// Lexes and parses one source file into a surface AST.
pub fn parse(source: &SourceFile) -> Result<Program, Diagnostic> {
    let tokens = lex(source)?;
    parse_tokens(source, &tokens)
}

/// Parses tokens the caller already lexed from `source`, so the formatter can share one lexing pass.
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
    nesting: usize,
    pending_generic_closers: usize,
    generic_close_span: Option<Span>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a SourceFile, tokens: &'a [Token]) -> Self {
        Self {
            source,
            tokens,
            position: 0,
            nesting: 0,
            pending_generic_closers: 0,
            generic_close_span: None,
        }
    }

    fn parse_program(mut self) -> Result<Program, Diagnostic> {
        let start = self.current_span().start();
        let mut requirements = Vec::new();
        while self.at(&TokenKind::Require) {
            requirements.push(self.parse_requirement()?);
        }
        let mut items = Vec::new();
        while !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Require) {
                return Err(
                    Diagnostic::error("require declaration after a top-level item")
                        .with_primary(self.current_span(), "requirements must appear first"),
                );
            }
            items.push(self.parse_top_item()?);
        }
        let end = self.current_span().end();
        Ok(Program {
            requirements,
            items,
            span: self.span(start, end),
        })
    }

    fn parse_requirement(&mut self) -> Result<Node<Requirement>, Diagnostic> {
        let start = self.expect(&TokenKind::Require, "`require`")?.span.start();
        let path = self.current().clone();
        let TokenKind::Symbol(value) = &path.kind else {
            return Err(self.expected("a quoted requirement path"));
        };
        self.advance();
        let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(Node::new(
            Requirement {
                path: value.clone(),
                path_span: path.span,
            },
            self.span(start, semicolon.span.end()),
        ))
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
            let parameters = self.parse_type_parameters()?;
            self.expect(&TokenKind::DoubleColon, "`::`")?;
            let value = self.parse_type()?;
            if parameters.is_empty() {
                TopItem::TypeAlias { name, value }
            } else {
                TopItem::GenericTypeAlias {
                    name,
                    parameters,
                    value,
                }
            }
        } else if self.at(&TokenKind::ValueIdentifier)
            && self
                .tokens
                .get(self.position + 1)
                .is_some_and(|token| token.kind == TokenKind::Less)
        {
            let name = self.parse_name(&TokenKind::ValueIdentifier, "a value name")?;
            let parameters = self.parse_required_type_parameters()?;
            self.expect(&TokenKind::DoubleColon, "`::`")?;
            let annotation = self.parse_type()?;
            self.expect(&TokenKind::Bind, "`:=`")?;
            let value = self.parse_expression()?;
            TopItem::GenericBinding {
                name,
                parameters,
                annotation,
                value,
            }
        } else {
            TopItem::Binding(self.parse_binding()?)
        };
        let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(Node::new(kind, self.span(start, semicolon.span.end())))
    }

    fn parse_type(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        self.within_syntax_nesting(Self::parse_type_inner)
    }

    fn parse_type_inner(&mut self) -> Result<Node<TypeExpression>, Diagnostic> {
        let parameter = self.parse_atomic_type()?;
        if self.pending_generic_closers == 0 && self.take(&TokenKind::Arrow).is_some() {
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
            let start = name.span.start();
            if self.at(&TokenKind::Less) {
                let arguments = self.parse_type_arguments()?;
                let end = self.previous_generic_close_span().end();
                return Ok(Node::new(
                    TypeExpression::Application {
                        constructor: name,
                        arguments,
                    },
                    self.span(start, end),
                ));
            }
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
            if let Some(right) = self.take(&TokenKind::RightBracket) {
                return Ok(Node::new(
                    TypeExpression::Sum(Vec::new()),
                    self.join(left.span, right.span),
                ));
            }
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

    fn parse_type_parameters(&mut self) -> Result<Vec<Name>, Diagnostic> {
        if !self.at(&TokenKind::Less) {
            return Ok(Vec::new());
        }
        self.parse_required_type_parameters()
    }

    fn parse_required_type_parameters(&mut self) -> Result<Vec<Name>, Diagnostic> {
        self.expect(&TokenKind::Less, "`<`")?;
        let mut parameters = vec![self.parse_name(&TokenKind::TypeIdentifier, "a type parameter")?];
        while self.take(&TokenKind::Comma).is_some() {
            parameters.push(self.parse_name(&TokenKind::TypeIdentifier, "a type parameter")?);
        }
        self.expect_generic_close()?;
        Ok(parameters)
    }

    fn parse_type_arguments(&mut self) -> Result<Vec<Node<TypeExpression>>, Diagnostic> {
        self.expect(&TokenKind::Less, "`<`")?;
        let mut arguments = vec![self.parse_type()?];
        while self.pending_generic_closers == 0 && self.take(&TokenKind::Comma).is_some() {
            arguments.push(self.parse_type()?);
        }
        self.expect_generic_close()?;
        Ok(arguments)
    }

    fn expect_generic_close(&mut self) -> Result<(), Diagnostic> {
        if self.pending_generic_closers > 0 {
            self.pending_generic_closers -= 1;
            return Ok(());
        }
        if self.at(&TokenKind::Greater) {
            self.generic_close_span = Some(self.advance().span);
            return Ok(());
        }
        if self.at(&TokenKind::ShiftRight) {
            self.generic_close_span = Some(self.advance().span);
            self.pending_generic_closers = 1;
            return Ok(());
        }
        Err(self.expected("`>` after type arguments"))
    }

    fn previous_generic_close_span(&self) -> Span {
        self.generic_close_span
            .expect("a generic close was just parsed")
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
        self.within_syntax_nesting(Self::parse_pattern_inner)
    }

    fn parse_pattern_inner(&mut self) -> Result<Node<Pattern>, Diagnostic> {
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
        self.expect(&TokenKind::Arrow, "`->`")?;
        self.parse_expression_body()
    }

    fn parse_expression_body(&mut self) -> Result<LambdaBody, Diagnostic> {
        if self.at(&TokenKind::LeftBrace) {
            return self.parse_expression_block();
        }
        let result = self.parse_expression()?;
        let span = result.span;
        Ok(LambdaBody {
            items: Vec::new(),
            result: Box::new(result),
            span,
        })
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

    fn within_syntax_nesting<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        if self.nesting == MAX_SYNTAX_NESTING {
            return Err(self.error_here(
                "syntax nesting limit exceeded",
                format!("malc supports at most {MAX_SYNTAX_NESTING} levels"),
            ));
        }
        self.nesting += 1;
        let result = parse(self);
        self.nesting -= 1;
        result
    }

    fn join(&self, start: Span, end: Span) -> Span {
        self.span(start.start(), end.end())
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.source.id(), start, end)
    }
}
