//! Statements, blocks, and switch arms.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::expression::c_expression;
use super::support::*;
use super::ty::c_type_syntax;

pub(super) fn c_statement_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let statement = c_statement_tokens(&mut input)?;
    if input.is_empty() {
        Ok(statement)
    } else {
        Err("unexpected token after C statement".into())
    }
}

fn c_statement_tokens(input: &mut Cursor) -> Result<TokenStream, String> {
    if let Some(TokenTree::Ident(keyword)) = input.peek() {
        match keyword.to_string().as_str() {
            "let" => return c_let_statement(input),
            "return" => return c_return_statement(input),
            "if" => return c_if_statement(input),
            "match" => return c_match_statement(input),
            _ => {}
        }
    }

    if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
        && (input.position + 1 == input.tokens.len()
            || matches!(input.tokens.get(input.position + 1), Some(TokenTree::Punct(token)) if token.as_char() == ';'))
    {
        let statement = match input.next() {
            Some(TokenTree::Group(group)) => group.stream(),
            _ => unreachable!(),
        };
        input.eat_punct(';');
        return Ok(statement);
    }

    let expression = c_expression(take_until_punct(input, ';')?)?;
    Ok(rust_call(
        "crate::backend::c::syntax::Statement::expression",
        [expression],
    ))
}

fn c_let_statement(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    let name = match input.next() {
        Some(TokenTree::Ident(name)) => format!("{:?}", name.to_string()).parse().unwrap(),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group.stream(),
        _ => return Err("expected identifier after `let`".into()),
    };
    if !input.eat_punct(':') {
        return Err("expected `:` after C variable name".into());
    }
    let type_start = input.position;
    while !input.is_empty()
        && !matches!(input.peek(), Some(TokenTree::Punct(token)) if matches!(token.as_char(), '=' | ';'))
    {
        input.position += 1;
    }
    let ty = c_type_syntax(token_slice(&input.tokens[type_start..input.position]))?;
    let declaration = rust_call(
        "crate::backend::c::syntax::VariableDeclaration::new",
        [ty, name],
    );
    let initializer = if input.eat_punct('=') {
        let expression = c_expression(take_until_punct(input, ';')?)?;
        rust_call("Some", [expression])
    } else {
        if !input.eat_punct(';') {
            return Err("expected `;` after C variable declaration".into());
        }
        tokens("None")
    };
    Ok(rust_call(
        "crate::backend::c::syntax::Statement::variable_declaration",
        [declaration, initializer],
    ))
}

fn c_return_statement(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    if input.eat_punct(';') {
        Ok(rust_call(
            "crate::backend::c::syntax::Statement::return_void",
            std::iter::empty::<TokenStream>(),
        ))
    } else {
        let expression = c_expression(take_until_punct(input, ';')?)?;
        Ok(rust_call(
            "crate::backend::c::syntax::Statement::return_value",
            [expression],
        ))
    }
}

fn c_if_statement(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    let condition_start = input.position;
    while !matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
    {
        if input.next().is_none() {
            return Err("expected block after C `if` condition".into());
        }
    }
    let condition = c_expression(token_slice(&input.tokens[condition_start..input.position]))?;
    let body = match input.next() {
        Some(TokenTree::Group(group)) => c_block_syntax(group.stream())?,
        _ => unreachable!(),
    };
    Ok(rust_call(
        "crate::backend::c::syntax::Statement::if_then",
        [condition, body],
    ))
}

fn c_match_statement(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    let value_start = input.position;
    while !matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
    {
        if input.next().is_none() {
            return Err("expected arms after C `match` value".into());
        }
    }
    let value = c_expression(token_slice(&input.tokens[value_start..input.position]))?;
    let arms = match input.next() {
        Some(TokenTree::Group(group)) => c_switch_arms(group.stream())?,
        _ => unreachable!(),
    };
    Ok(rust_call(
        "crate::backend::c::syntax::Statement::switch",
        [value, arms],
    ))
}

pub(super) fn c_switch_arms(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C match-arm splice".into());
            }
            let values = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after match-arm splice".into()),
            };
            operations.extend(tokens("__mal_backend_cases.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, values))]);
            operations.extend(tokens(";"));
        } else {
            let label_start = input.position;
            while !(matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == '=')
                && matches!(input.tokens.get(input.position + 1), Some(TokenTree::Punct(token)) if token.as_char() == '>'))
            {
                if input.next().is_none() {
                    return Err("expected `=>` in C match arm".into());
                }
            }
            let label = token_slice(&input.tokens[label_start..input.position]);
            input.position += 2;
            let body = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    c_block_syntax(group.stream())?
                }
                _ => return Err("expected block after C match arm".into()),
            };
            let case = if label.to_string() == "_" {
                rust_call("crate::backend::c::syntax::SwitchCase::default", [body])
            } else {
                rust_call(
                    "crate::backend::c::syntax::SwitchCase::case",
                    [c_expression(label)?, body],
                )
            };
            operations.extend(tokens("__mal_backend_cases.push"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, case))]);
            operations.extend(tokens(";"));
        }
        if !input.eat_punct(',') && !input.is_empty() {
            return Err("expected `,` between C match arms".into());
        }
    }
    let mut body = tokens("let mut __mal_backend_cases = Vec::new();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_cases"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}

pub(super) fn c_block_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C statement splice".into());
            }
            let values = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after statement splice".into()),
            };
            input.eat_punct(';');
            operations.extend(tokens("__mal_backend_block.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, values))]);
            operations.extend(tokens(";"));
        } else {
            let statement = c_statement_tokens(&mut input)?;
            operations.extend(tokens("__mal_backend_block.push"));
            operations.extend([TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                statement,
            ))]);
            operations.extend(tokens(";"));
        }
    }
    let mut body =
        tokens("let mut __mal_backend_block = crate::backend::c::syntax::Block::default();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_block"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}
