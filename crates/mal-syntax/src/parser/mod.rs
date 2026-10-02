//! Recursive-descent syntax admission with Pratt parsing for expressions.

use std::mem::discriminant;

use crate::ast::{
    Binding, Expression, LambdaBody, Name, Node, Pattern, Program, Requirement, TopItem,
    TypeExpression,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind, lex};
use crate::source::{SourceFile, Span};

mod binding;
mod cursor;
mod expression;
mod generic;
mod types;

use generic::GenericLists;

const MAX_SYNTAX_NESTING: usize = 64;

/// Lexes and parses one source file into a surface AST.
pub fn parse(source: &SourceFile) -> Result<Program, Diagnostic> {
    let tokens = lex(source)?;
    parse_tokens(source, &tokens)
}

/// Lexes `source` and parses only its leading `require` declarations, which is all source-graph loading needs. The rest
/// of the file is parsed once, by the analysis that consumes it.
pub fn parse_requirements(source: &SourceFile) -> Result<Vec<Node<Requirement>>, Diagnostic> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(source, &tokens);
    let mut requirements = Vec::new();
    while parser.at(&TokenKind::Require) {
        requirements.push(parser.parse_requirement()?);
    }
    Ok(requirements)
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
    generic: GenericLists,
}

impl<'a> Parser<'a> {
    fn new(source: &'a SourceFile, tokens: &'a [Token]) -> Self {
        Self {
            source,
            tokens,
            position: 0,
            nesting: 0,
            generic: GenericLists::default(),
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
        } else if self.take(&TokenKind::Opaque).is_some() {
            let name = self.parse_name(&TokenKind::TypeIdentifier, "a type name")?;
            let parameters = self.parse_type_parameters()?;
            self.expect(&TokenKind::DoubleColon, "`::`")?;
            let representation = self.parse_type()?;
            TopItem::OpaqueType {
                name,
                parameters,
                representation,
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
            let arguments = self.parse_type_arguments()?;
            self.expect(&TokenKind::DoubleColon, "`::`")?;
            let annotation = self.parse_type()?;
            let value = if self.take(&TokenKind::Bind).is_some() {
                Some(self.parse_expression()?)
            } else {
                None
            };
            TopItem::GenericBinding {
                name,
                arguments,
                annotation,
                value,
            }
        } else {
            TopItem::Binding(self.parse_binding()?)
        };
        let semicolon = self.expect(&TokenKind::Semicolon, "`;`")?;
        Ok(Node::new(kind, self.span(start, semicolon.span.end())))
    }
}
