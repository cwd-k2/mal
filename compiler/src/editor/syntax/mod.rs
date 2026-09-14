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
pub struct SyntaxDocument {
    tokens: Vec<SyntaxToken>,
    functions: Vec<String>,
    requirements: Vec<Span>,
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
        requirements: requirement_spans(&tokens),
    })
}

impl SyntaxDocument {
    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }

    pub fn functions(&self) -> &[String] {
        &self.functions
    }

    pub fn requirements(&self) -> &[Span] {
        &self.requirements
    }
}

fn requirement_spans(tokens: &[Token]) -> Vec<Span> {
    tokens
        .windows(3)
        .filter(|tokens| {
            matches!(
                (&tokens[0].kind, &tokens[1].kind, &tokens[2].kind),
                (
                    TokenKind::Require,
                    TokenKind::Symbol(_),
                    TokenKind::Semicolon
                )
            )
        })
        .map(|tokens| {
            Span::new(
                tokens[0].span.file(),
                tokens[0].span.start(),
                tokens[2].span.end(),
            )
        })
        .collect()
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
        let requirement = document.requirements()[0];
        assert_eq!(
            &source.text()[requirement.start()..requirement.end()],
            "require \"./dependency.mal\";"
        );
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
