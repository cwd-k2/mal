//! Token extraction and collection helpers shared by the LLVM grammar modules.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

pub(super) const SYNTAX: &str = "crate::backend::llvm::syntax";

pub(super) fn code(source: impl AsRef<str>) -> TokenStream {
    source
        .as_ref()
        .parse()
        .expect("generated Rust tokens parse")
}

pub(super) fn rust_expression(token: &TokenTree) -> Option<TokenStream> {
    let TokenTree::Group(group) = token else {
        return None;
    };
    if group.delimiter() != Delimiter::Brace {
        return None;
    }
    Some(group.stream())
}

pub(super) fn scalar(token: TokenTree) -> TokenStream {
    rust_expression(&token).unwrap_or_else(|| [token].into_iter().collect())
}

pub(super) fn one_token(input: TokenStream, description: &str) -> Result<TokenTree, String> {
    let mut tokens = input.into_iter();
    let token = tokens
        .next()
        .ok_or_else(|| format!("expected {description}"))?;
    if tokens.next().is_some() {
        return Err(format!("expected one {description}"));
    }
    Ok(token)
}

pub(super) fn group(
    token: TokenTree,
    delimiter: Delimiter,
    description: &str,
) -> Result<Group, String> {
    match token {
        TokenTree::Group(group) if group.delimiter() == delimiter => Ok(group),
        _ => Err(format!("expected {description}")),
    }
}

pub(super) fn split(input: TokenStream, separator: char) -> Vec<TokenStream> {
    let mut items = Vec::new();
    let mut current = TokenStream::new();
    for token in input {
        if matches!(&token, TokenTree::Punct(punctuation) if punctuation.as_char() == separator) {
            items.push(current);
            current = TokenStream::new();
        } else {
            current.extend([token]);
        }
    }
    items.push(current);
    items
}

pub(super) fn comma_items(input: TokenStream) -> Result<Vec<TokenStream>, String> {
    let mut items = split(input, ',');
    if items.last().is_some_and(TokenStream::is_empty) {
        items.pop();
    }
    if items.iter().any(TokenStream::is_empty) {
        return Err("expected an item between commas".into());
    }
    Ok(items)
}

pub(super) fn splice(item: &TokenStream) -> Option<TokenStream> {
    let tokens = item.clone().into_iter().collect::<Vec<_>>();
    if tokens.len() == 3
        && tokens[..2]
            .iter()
            .all(|token| matches!(token, TokenTree::Punct(token) if token.as_char() == '.'))
    {
        rust_expression(&tokens[2])
    } else {
        None
    }
}

pub(super) fn embedded(item: &TokenStream) -> Option<TokenStream> {
    let token = one_token(item.clone(), "embedded Rust expression").ok()?;
    rust_expression(&token)
}

pub(super) fn vector(
    input: TokenStream,
    mut static_item: impl FnMut(TokenStream) -> Result<TokenStream, String>,
) -> Result<TokenStream, String> {
    let mut statements = String::from("{ #[allow(unused_mut)] let mut __mal_items = Vec::new();");
    for item in comma_items(input)? {
        if let Some(expression) = splice(&item) {
            statements.push_str(&format!("__mal_items.extend({expression});"));
        } else if let Some(expression) = embedded(&item) {
            statements.push_str(&format!("__mal_items.push({expression});"));
        } else {
            statements.push_str(&format!("__mal_items.push({});", static_item(item)?));
        }
    }
    statements.push_str("__mal_items }");
    Ok(code(statements))
}

pub(super) fn fallible_vector(
    input: TokenStream,
    mut static_item: impl FnMut(TokenStream) -> Result<TokenStream, String>,
) -> Result<TokenStream, String> {
    let mut statements =
        String::from("(|| { #[allow(unused_mut)] let mut __mal_items = Vec::new();");
    for item in comma_items(input)? {
        if let Some(expression) = splice(&item) {
            statements.push_str(&format!("__mal_items.extend({expression});"));
        } else if let Some(expression) = embedded(&item) {
            statements.push_str(&format!("__mal_items.push({expression});"));
        } else {
            statements.push_str(&format!("__mal_items.push(({})?);", static_item(item)?));
        }
    }
    statements.push_str("Some(__mal_items) })()");
    Ok(code(statements))
}

pub(super) fn named_fields(input: TokenStream) -> Result<Vec<(String, TokenStream)>, String> {
    comma_items(input)?
        .into_iter()
        .map(|item| {
            let tokens = item.into_iter().collect::<Vec<_>>();
            let colon = tokens
                .iter()
                .position(
                    |token| matches!(token, TokenTree::Punct(token) if token.as_char() == ':'),
                )
                .ok_or("expected `:` after LLVM field name")?;
            if colon != 1 {
                return Err("expected one LLVM field name".into());
            }
            Ok((
                tokens[0].to_string(),
                tokens[colon + 1..].iter().cloned().collect(),
            ))
        })
        .collect()
}

pub(super) fn fields(input: TokenStream, expected: &[&str]) -> Result<Vec<TokenStream>, String> {
    let fields = named_fields(input)?;
    if fields.len() != expected.len()
        || fields
            .iter()
            .zip(expected)
            .any(|((actual, _), expected)| actual != expected)
    {
        return Err(format!(
            "expected LLVM fields `{}` in this order",
            expected.join(", ")
        ));
    }
    Ok(fields.into_iter().map(|(_, value)| value).collect())
}
