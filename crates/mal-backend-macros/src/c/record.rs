//! Type aliases, records, fields, and function translation-unit items.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::function::{c_function_syntax, c_parameters_syntax, c_signature_syntax};
use super::support::*;
use super::ty::c_type_syntax;

pub(super) fn c_type_alias_item(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    let alias = c_name(input, "type alias")?;
    if !input.eat_punct('=') {
        return Err("expected `=` in C type alias".into());
    }
    if matches!(input.peek(), Some(TokenTree::Ident(kind)) if matches!(kind.to_string().as_str(), "struct" | "union"))
    {
        let kind = match input.next() {
            Some(TokenTree::Ident(kind)) => kind.to_string(),
            _ => unreachable!(),
        };
        let tag = if matches!(input.peek(), Some(TokenTree::Ident(_))) {
            match input.next() {
                Some(TokenTree::Ident(tag)) => Some(tag.to_string()),
                _ => unreachable!(),
            }
        } else {
            None
        };
        let fields = c_record_body(input)?;
        if !input.eat_punct(';') {
            return Err("expected `;` after C record alias".into());
        }
        return Ok(rust_call(
            if kind == "struct" {
                "crate::backend::c::syntax::RecordDefinition::typedef_structure"
            } else {
                "crate::backend::c::syntax::RecordDefinition::typedef_union"
            },
            [option_string(tag), fields, alias],
        ));
    }
    let ty_start = input.position;
    while !input.eat_punct(';') {
        if input.next().is_none() {
            return Err("expected `;` after C type alias".into());
        }
    }
    let ty = c_type_syntax(token_slice(&input.tokens[ty_start..input.position - 1]))?;
    Ok(rust_call(
        "crate::backend::c::syntax::Declaration::type_alias",
        [ty, alias],
    ))
}

fn option_string(value: Option<String>) -> TokenStream {
    match value {
        Some(value) => rust_call(
            "Some",
            [rust_call(
                "String::from",
                [format!("{value:?}").parse().unwrap()],
            )],
        ),
        None => tokens("None"),
    }
}

pub(super) fn c_record_item(input: &mut Cursor, expected: &str) -> Result<TokenStream, String> {
    input.next();
    let tag = c_name(input, "record tag")?;
    let fields = c_record_body(input)?;
    input.eat_punct(';');
    Ok(rust_call(
        if expected == "struct" {
            "crate::backend::c::syntax::RecordDefinition::structure"
        } else {
            "crate::backend::c::syntax::RecordDefinition::union"
        },
        [tag, fields],
    ))
}

pub(super) fn c_record_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let kind = match input.peek() {
        Some(TokenTree::Ident(kind)) if matches!(kind.to_string().as_str(), "struct" | "union") => {
            kind.to_string()
        }
        _ => return Err("expected `struct` or `union` C record".into()),
    };
    let record = c_record_item(&mut input, &kind)?;
    if input.is_empty() {
        Ok(record)
    } else {
        Err("unexpected token after C record".into())
    }
}

fn c_record_body(input: &mut Cursor) -> Result<TokenStream, String> {
    match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            c_record_fields_syntax(group.stream())
        }
        _ => Err("expected C record fields".into()),
    }
}

pub(super) fn c_record_fields_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C record-field splice".into());
            }
            let fields = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after record-field splice".into()),
            };
            operations.extend(tokens("__mal_backend_fields.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, fields))]);
            operations.extend(tokens(";"));
        } else if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
        {
            let field = match input.next() {
                Some(TokenTree::Group(group)) => rust_call(
                    "crate::backend::c::syntax::RecordField::from",
                    [group.stream()],
                ),
                _ => unreachable!(),
            };
            push_record_field(&mut operations, field);
        } else {
            let name = c_name(&mut input, "record field name")?;
            if !input.eat_punct(':') {
                return Err("expected `:` after record field name".into());
            }
            let field = if matches!(input.peek(), Some(TokenTree::Ident(kind)) if matches!(kind.to_string().as_str(), "struct" | "union"))
            {
                let kind = match input.next() {
                    Some(TokenTree::Ident(kind)) => kind.to_string(),
                    _ => unreachable!(),
                };
                let fields = c_record_body(&mut input)?;
                rust_call(
                    "crate::backend::c::syntax::RecordField::record",
                    [
                        tokens(if kind == "struct" {
                            "crate::backend::c::syntax::RecordKind::Struct"
                        } else {
                            "crate::backend::c::syntax::RecordKind::Union"
                        }),
                        fields,
                        name,
                    ],
                )
            } else if matches!(input.peek(), Some(TokenTree::Ident(kind)) if kind.to_string() == "fn")
            {
                input.next();
                let parameters = match input.next() {
                    Some(TokenTree::Group(group))
                        if group.delimiter() == Delimiter::Parenthesis =>
                    {
                        c_parameters_syntax(group.stream())?
                    }
                    _ => return Err("expected function-pointer parameters".into()),
                };
                if !input.eat_punct('-') || !input.eat_punct('>') {
                    return Err("expected function-pointer result".into());
                }
                let result_start = input.position;
                while !input.is_empty()
                    && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == ',')
                {
                    input.position += 1;
                }
                let result =
                    c_type_syntax(token_slice(&input.tokens[result_start..input.position]))?;
                rust_call(
                    "crate::backend::c::syntax::RecordField::function_pointer",
                    [result, name, parameters],
                )
            } else {
                let type_start = input.position;
                while !input.is_empty()
                    && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == ',')
                {
                    input.position += 1;
                }
                let ty = c_type_syntax(token_slice(&input.tokens[type_start..input.position]))?;
                rust_call(
                    "crate::backend::c::syntax::RecordField::variable",
                    [ty, name],
                )
            };
            push_record_field(&mut operations, field);
        }
        if !input.eat_punct(',') && !input.is_empty() {
            return Err("expected `,` between C record fields".into());
        }
    }
    let mut body = tokens("let mut __mal_backend_fields = Vec::new();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_fields"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}

fn push_record_field(operations: &mut TokenStream, field: TokenStream) {
    operations.extend(tokens("__mal_backend_fields.push"));
    operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, field))]);
    operations.extend(tokens(";"));
}

pub(super) fn c_function_item(input: &mut Cursor) -> Result<TokenStream, String> {
    let start = input.position;
    while !matches!(input.peek(), Some(TokenTree::Ident(name)) if name.to_string() == "fn") {
        if input.next().is_none() {
            return Err("expected C function item".into());
        }
    }
    input.next();
    input.next().ok_or("expected C function name")?;
    match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {}
        _ => return Err("expected C function parameters".into()),
    }
    let result_start = input.position;
    let mut declaration = false;
    loop {
        match input.peek() {
            Some(TokenTree::Punct(token)) if token.as_char() == ';' => {
                input.position += 1;
                declaration = true;
                break;
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let prefix = token_slice(&input.tokens[result_start..input.position]);
                if prefix.is_empty() || c_function_result_prefix(prefix).is_ok() {
                    input.position += 1;
                    break;
                }
                input.position += 1;
            }
            Some(_) => input.position += 1,
            None => return Err("expected C function body or `;`".into()),
        }
    }
    let mut item = token_slice(&input.tokens[start..input.position]);
    if declaration {
        let mut tokens = item.into_iter().collect::<Vec<_>>();
        tokens.pop();
        item = tokens.into_iter().collect();
        Ok(rust_call(
            "crate::backend::c::syntax::Declaration::function",
            [c_signature_syntax(item)?],
        ))
    } else {
        c_function_syntax(item)
    }
}

fn c_function_result_prefix(input: TokenStream) -> Result<(), String> {
    let mut input = Cursor::new(input);
    if input.eat_punct('-') && input.eat_punct('>') {
        c_type_syntax(token_slice(&input.tokens[input.position..])).map(|_| ())
    } else if input.is_empty() {
        Ok(())
    } else {
        Err("expected C function result".into())
    }
}
