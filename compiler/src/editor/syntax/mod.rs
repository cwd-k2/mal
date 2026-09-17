use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind, lex};
use crate::source::{SourceFile, Span};

use super::SymbolKind;

mod declaration;

use declaration::function_declarations;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxToken {
    pub span: Span,
    pub kind: SymbolKind,
    pub declaration: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxRequirement {
    declaration_span: Span,
    path_span: Span,
    path: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxDocument {
    tokens: Vec<SyntaxToken>,
    functions: Vec<String>,
    requirements: Vec<SyntaxRequirement>,
}

pub(super) fn analyze(source: &SourceFile) -> Result<SyntaxDocument, Diagnostic> {
    let tokens = lex(source)?;
    let function_declarations = function_declarations(&tokens);
    let syntax_tokens = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let kind = match token.kind {
                TokenKind::TypeIdentifier => SymbolKind::Type,
                TokenKind::ValueIdentifier if function_declarations.contains(&index) => {
                    SymbolKind::Function
                }
                TokenKind::ValueIdentifier
                    if matches!(index.checked_sub(1).and_then(|index| tokens.get(index)), Some(previous) if previous.kind == TokenKind::Dot)
                        || matches!(tokens.get(index + 1), Some(next) if next.kind == TokenKind::LeftParen) =>
                {
                    SymbolKind::Function
                }
                TokenKind::ValueIdentifier => SymbolKind::Value,
                _ => return None,
            };
            let declaration = function_declarations.contains(&index)
                || matches!(tokens.get(index + 1), Some(next) if matches!(next.kind, TokenKind::DoubleColon | TokenKind::Bind));
            Some(SyntaxToken {
                span: token.span,
                kind,
                declaration,
            })
        })
        .collect();
    let mut functions = function_declarations
        .into_iter()
        .map(|index| source.text()[tokens[index].span.start()..tokens[index].span.end()].to_owned())
        .collect::<Vec<_>>();
    functions.sort();
    functions.dedup();
    Ok(SyntaxDocument {
        tokens: syntax_tokens,
        functions,
        requirements: requirements(&tokens),
    })
}

impl SyntaxDocument {
    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }

    pub fn functions(&self) -> &[String] {
        &self.functions
    }

    pub fn requirements(&self) -> &[SyntaxRequirement] {
        &self.requirements
    }
}

impl SyntaxRequirement {
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    pub const fn path_span(&self) -> Span {
        self.path_span
    }

    pub fn path(&self) -> &[u8] {
        &self.path
    }
}

fn requirements(tokens: &[Token]) -> Vec<SyntaxRequirement> {
    let mut requirements = Vec::new();
    let mut cursor = 0;
    while let Some(declaration) = tokens.get(cursor..cursor + 3) {
        let (TokenKind::Require, TokenKind::Symbol(path), TokenKind::Semicolon) = (
            &declaration[0].kind,
            &declaration[1].kind,
            &declaration[2].kind,
        ) else {
            break;
        };
        requirements.push(SyntaxRequirement {
            declaration_span: Span::new(
                declaration[0].span.file(),
                declaration[0].span.start(),
                declaration[2].span.end(),
            ),
            path_span: Span::new(
                declaration[1].span.file(),
                declaration[1].span.start() + 1,
                declaration[1].span.end() - 1,
            ),
            path: path.clone(),
        });
        cursor += 3;
    }
    requirements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FileId;

    #[test]
    fn indexes_tokens_and_functions_without_parsing_the_program() {
        let source = SourceFile::new(
            FileId::new(0),
            "syntax.mal",
            "require \"./dependency.mal\";\nextern send :: Int32 -> Unit;\nhelper := (value) -> [done] => done(value);\nmain :: Unit -> Int32 := () -> helper(1).;\n".into(),
        );
        let document = analyze(&source).expect("lexical syntax document");

        assert_eq!(document.functions(), &["helper", "main", "send"]);
        assert_eq!(document.requirements().len(), 1);
        let requirement = &document.requirements()[0];
        assert_eq!(
            &source.text()
                [requirement.declaration_span().start()..requirement.declaration_span().end()],
            "require \"./dependency.mal\";"
        );
        assert_eq!(
            &source.text()[requirement.path_span().start()..requirement.path_span().end()],
            "./dependency.mal"
        );
        assert_eq!(requirement.path(), b"./dependency.mal");
        assert!(document.tokens().iter().any(|token| {
            token.kind == SymbolKind::Function
                && &source.text()[token.span.start()..token.span.end()] == "helper"
        }));
    }

    #[test]
    fn does_not_treat_a_product_containing_a_function_as_a_function() {
        let source = SourceFile::new(
            FileId::new(0),
            "syntax.mal",
            "pair :: (Int32 -> Int32, Int32) := missing;\n\
             callback :: (Int32 -> Int32) := missing;\n\
             main :: Unit -> Int32 := () -> { callback(1). };\n"
                .into(),
        );
        let document = analyze(&source).expect("lexical syntax document");

        assert_eq!(document.functions(), &["callback", "main"]);
    }
}
