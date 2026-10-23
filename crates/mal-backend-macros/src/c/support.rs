//! Token construction and cursor helpers shared by the C grammar modules.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

pub(super) fn tokens(source: &str) -> TokenStream {
    source.parse().expect("generated Rust tokens parse")
}

pub(super) fn comma_separated(items: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let mut output = TokenStream::new();
    let mut first = true;
    for item in items {
        if !first {
            output.extend(tokens(","));
        }
        first = false;
        output.extend(item);
    }
    output
}

pub(super) fn rust_call(
    path: &str,
    arguments: impl IntoIterator<Item = TokenStream>,
) -> TokenStream {
    let mut output = tokens(path);
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        comma_separated(arguments),
    ))]);
    output
}

pub(super) fn rust_method_call(receiver: TokenStream, method: &str) -> TokenStream {
    let mut output = TokenStream::new();
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        receiver,
    ))]);
    output.extend(tokens(&format!(".{method}()")));
    output
}

pub(super) fn rust_method_suffix(receiver: TokenStream, suffix: &str) -> TokenStream {
    let mut output = TokenStream::new();
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        receiver,
    ))]);
    output.extend(tokens(suffix));
    output
}

pub(super) fn token_slice(tokens: &[TokenTree]) -> TokenStream {
    tokens.iter().cloned().collect()
}

pub(super) fn take_until_punct(
    input: &mut Cursor,
    punctuation: char,
) -> Result<TokenStream, String> {
    let start = input.position;
    while !input.is_empty()
        && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == punctuation)
    {
        input.position += 1;
    }
    if input.is_empty() {
        return Err(format!("expected `{punctuation}`"));
    }
    let result = token_slice(&input.tokens[start..input.position]);
    input.position += 1;
    Ok(result)
}

pub(super) fn c_name(input: &mut Cursor, description: &str) -> Result<TokenStream, String> {
    match input.next() {
        Some(TokenTree::Ident(name)) => {
            Ok(format!("{:?}", name.to_string().trim_start_matches("r#"))
                .parse()
                .unwrap())
        }
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            Ok(group.stream())
        }
        _ => Err(format!("expected {description}")),
    }
}

pub(super) fn rust_array(items: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let mut output = TokenStream::new();
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Bracket,
        comma_separated(items),
    ))]);
    output
}
