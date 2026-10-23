//! Parameters, signatures, and function definitions.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::statement::c_block_syntax;
use super::support::*;
use super::ty::c_type_syntax;

fn c_attributes(input: &mut Cursor) -> Result<Vec<String>, String> {
    let mut attributes = Vec::new();
    while input.eat_punct('#') {
        let attribute = match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                group.stream().to_string()
            }
            _ => return Err("expected attribute after `#`".into()),
        };
        attributes.push(attribute);
    }
    Ok(attributes)
}

pub(super) fn c_parameters_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C parameter splice".into());
            }
            let values = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after parameter splice".into()),
            };
            operations.extend(tokens("__mal_backend_parameters.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, values))]);
            operations.extend(tokens(";"));
        } else if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
            && !matches!(input.tokens.get(input.position + 1), Some(TokenTree::Punct(token)) if token.as_char() == ':')
        {
            let parameter = match input.next() {
                Some(TokenTree::Group(group)) => group.stream(),
                _ => unreachable!(),
            };
            operations.extend(tokens("__mal_backend_parameters.push"));
            operations.extend([TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                parameter,
            ))]);
            operations.extend(tokens(";"));
        } else {
            let attributes = c_attributes(&mut input)?;
            let unnamed =
                matches!(input.peek(), Some(TokenTree::Ident(name)) if name.to_string() == "_");
            let name = c_name(&mut input, "C parameter name")?;
            if !input.eat_punct(':') {
                return Err("expected `:` after C parameter name".into());
            }
            let start = input.position;
            while !input.is_empty()
                && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == ',')
            {
                input.position += 1;
            }
            let ty = c_type_syntax(token_slice(&input.tokens[start..input.position]))?;
            let mut parameter = if unnamed {
                rust_call("crate::backend::c::syntax::Parameter::unnamed", [ty])
            } else {
                rust_call("crate::backend::c::syntax::Parameter::named", [ty, name])
            };
            for attribute in attributes {
                if attribute == "maybe_unused" {
                    parameter = rust_method_call(parameter, "maybe_unused");
                } else {
                    return Err(format!(
                        "unsupported C parameter attribute `#[{attribute}]`"
                    ));
                }
            }
            operations.extend(tokens("__mal_backend_parameters.push"));
            operations.extend([TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                parameter,
            ))]);
            operations.extend(tokens(";"));
        }
        if !input.eat_punct(',') && !input.is_empty() {
            return Err("expected `,` between C parameters".into());
        }
    }
    let mut body = tokens("let mut __mal_backend_parameters = Vec::new();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_parameters"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}

pub(super) fn c_parameter_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let tokens = input.clone().into_iter().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Err("expected one C parameter".into());
    }
    if matches!(tokens.first(), Some(TokenTree::Punct(token)) if token.as_char() == '.') {
        return Err("a C parameter splice is not one parameter".into());
    }
    let commas = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            matches!(token, TokenTree::Punct(token) if token.as_char() == ',').then_some(index)
        })
        .collect::<Vec<_>>();
    if commas.len() > 1
        || commas
            .first()
            .is_some_and(|index| *index + 1 != tokens.len())
    {
        return Err("expected one C parameter".into());
    }
    c_parameters_syntax(input).map(|parameters| {
        rust_method_suffix(
            parameters,
            ".into_iter().next().expect(\"one parsed C parameter\")",
        )
    })
}

fn c_signature_tokens(input: &mut Cursor) -> Result<TokenStream, String> {
    let attributes = c_attributes(input)?;
    match input.next() {
        Some(TokenTree::Ident(keyword)) if keyword.to_string() == "fn" => {}
        _ => return Err("expected `fn` in C signature".into()),
    }
    let name = c_name(input, "C function name")?;
    let parameters = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            c_parameters_syntax(group.stream())?
        }
        _ => return Err("expected C function parameters".into()),
    };
    let result = if input.eat_punct('-') {
        if !input.eat_punct('>') {
            return Err("expected `->` before C result type".into());
        }
        c_type_syntax(token_slice(&input.tokens[input.position..]))?
    } else {
        rust_call(
            "crate::backend::c::syntax::TypeName::named",
            [tokens("\"void\"")],
        )
    };
    input.position = input.tokens.len();
    let signature = rust_call(
        "crate::backend::c::syntax::FunctionSignature::new",
        [result, name, parameters],
    );
    if attributes.is_empty() {
        return Ok(signature);
    }
    let mut specifiers = Vec::new();
    for attribute in attributes {
        let variant = match attribute.as_str() {
            "static" => "crate::backend::c::syntax::FunctionSpecifier::Static",
            "inline" => "crate::backend::c::syntax::FunctionSpecifier::Inline",
            "noreturn" => "crate::backend::c::syntax::FunctionSpecifier::NoReturn",
            "overloadable" => "crate::backend::c::syntax::FunctionSpecifier::Overloadable",
            _ => return Err(format!("unsupported C function attribute `#[{attribute}]`")),
        };
        specifiers.push(tokens(variant));
    }
    let mut array = TokenStream::new();
    array.extend([TokenTree::Group(Group::new(
        Delimiter::Bracket,
        comma_separated(specifiers),
    ))]);
    Ok(rust_call(
        "crate::backend::c::syntax::FunctionSignature::with_specifiers",
        [signature, array],
    ))
}

pub(super) fn c_signature_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let signature = c_signature_tokens(&mut input)?;
    if input.is_empty() {
        Ok(signature)
    } else {
        Err("unexpected token after C signature".into())
    }
}

pub(super) fn c_function_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let body_index = input
        .tokens
        .iter()
        .rposition(|token| matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace))
        .ok_or_else(|| "expected C function body".to_string())?;
    let signature = c_signature_syntax(token_slice(&input.tokens[..body_index]))?;
    let body = match input.tokens.get(body_index).cloned() {
        Some(TokenTree::Group(group)) => c_block_syntax(group.stream())?,
        _ => unreachable!(),
    };
    input.position = input.tokens.len();
    Ok(rust_call(
        "crate::backend::c::syntax::FunctionDefinition::from_signature",
        [signature, body],
    ))
}
