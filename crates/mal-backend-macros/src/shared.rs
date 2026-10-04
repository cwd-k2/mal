//! Token utilities shared by the C and LLVM syntax parsers.

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub(super) type ParseResult = Result<TokenStream, String>;

#[derive(Clone)]
pub(super) struct Cursor {
    pub(super) tokens: Vec<TokenTree>,
    pub(super) position: usize,
}

impl Cursor {
    pub(super) fn new(tokens: TokenStream) -> Self {
        Self {
            tokens: tokens.into_iter().collect(),
            position: 0,
        }
    }

    pub(super) fn peek(&self) -> Option<&TokenTree> {
        self.tokens.get(self.position)
    }

    pub(super) fn next(&mut self) -> Option<TokenTree> {
        let token = self.tokens.get(self.position).cloned();
        self.position += usize::from(token.is_some());
        token
    }

    pub(super) fn is_empty(&self) -> bool {
        self.position == self.tokens.len()
    }

    pub(super) fn eat_punct(&mut self, character: char) -> bool {
        if matches!(self.peek(), Some(TokenTree::Punct(token)) if token.as_char() == character) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

pub(super) fn parse(input: TokenStream, parser: fn(TokenStream) -> ParseResult) -> TokenStream {
    let span = input
        .clone()
        .into_iter()
        .next()
        .map_or_else(Span::call_site, |token| token.span());
    parser(input).unwrap_or_else(|message| compile_error(&message, span))
}

pub(super) fn compile_error(message: &str, span: Span) -> TokenStream {
    let mut name = Ident::new("compile_error", span);
    name.set_span(span);
    let mut bang = Punct::new('!', Spacing::Alone);
    bang.set_span(span);
    let mut message = Literal::string(message);
    message.set_span(span);
    let mut arguments = Group::new(
        Delimiter::Parenthesis,
        [TokenTree::Literal(message)].into_iter().collect(),
    );
    arguments.set_span(span);
    [
        TokenTree::Ident(name),
        TokenTree::Punct(bang),
        TokenTree::Group(arguments),
    ]
    .into_iter()
    .collect()
}
