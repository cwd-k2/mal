use mal_compiler::lexer::{
    DecimalFloatLiteral, FloatSuffix, IntegerLiteral, IntegerSuffix, Radix, TokenKind, lex,
};
use mal_compiler::source::{FileId, SourceFile, Span};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(7), "test.mal", text.into())
}

fn kinds(text: &str) -> Vec<TokenKind> {
    lex(&source(text))
        .expect("source should lex")
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[path = "lexer/byte_strings.rs"]
mod byte_strings;
#[path = "lexer/numbers.rs"]
mod numbers;
#[path = "lexer/tokens.rs"]
mod tokens;
