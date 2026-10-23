//! Expressions, initializer lists, and expression sequences.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::support::*;
use super::ty::c_type_tokens;

pub(super) fn c_expression(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let expression = c_binary_expression(&mut input, 0)?;
    if input.is_empty() {
        Ok(expression)
    } else {
        Err("unexpected token after C expression".into())
    }
}

fn binary_operator(input: &Cursor) -> Option<(&'static str, u8, usize)> {
    let remaining = &input.tokens[input.position..];
    let punctuation = |index: usize, expected: char| matches!(remaining.get(index), Some(TokenTree::Punct(token)) if token.as_char() == expected);
    if punctuation(0, '&') && punctuation(1, '&') {
        Some(("logical_and", 2, 2))
    } else if punctuation(0, '=') && punctuation(1, '=') {
        Some(("equal", 3, 2))
    } else if punctuation(0, '!') && punctuation(1, '=') {
        Some(("not_equal", 3, 2))
    } else if punctuation(0, '>') {
        Some(("greater", 4, 1))
    } else if punctuation(0, '+') {
        Some(("add", 5, 1))
    } else if punctuation(0, '-') {
        Some(("subtract", 5, 1))
    } else if punctuation(0, '*') {
        Some(("multiply", 6, 1))
    } else if punctuation(0, '=') {
        Some(("assign", 1, 1))
    } else {
        None
    }
}

fn c_binary_expression(input: &mut Cursor, minimum_precedence: u8) -> Result<TokenStream, String> {
    let mut left = c_prefix_expression(input)?;
    loop {
        if matches!(input.peek(), Some(TokenTree::Ident(keyword)) if keyword.to_string() == "as") {
            if 7 < minimum_precedence {
                break;
            }
            input.next();
            let ty = c_type_tokens(input)?;
            left = rust_call("crate::backend::c::syntax::Expr::cast", [ty, left]);
            continue;
        }
        let Some((constructor, precedence, width)) = binary_operator(input) else {
            break;
        };
        if precedence < minimum_precedence {
            break;
        }
        input.position += width;
        let next_precedence = if constructor == "assign" {
            precedence
        } else {
            precedence + 1
        };
        let right = c_binary_expression(input, next_precedence)?;
        left = rust_call(
            &format!("crate::backend::c::syntax::Expr::{constructor}"),
            [left, right],
        );
    }
    Ok(left)
}

fn c_prefix_expression(input: &mut Cursor) -> Result<TokenStream, String> {
    if input.eat_punct('&') {
        return Ok(rust_call(
            "crate::backend::c::syntax::Expr::address_of",
            [c_prefix_expression(input)?],
        ));
    }
    if input.eat_punct('*') {
        return Ok(rust_call(
            "crate::backend::c::syntax::Expr::dereference",
            [c_prefix_expression(input)?],
        ));
    }

    if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
        && matches!(input.tokens.get(input.position + 1), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
    {
        let ty = match input.next() {
            Some(TokenTree::Group(group)) => rust_call(
                "crate::backend::c::syntax::TypeName::from",
                [group.stream()],
            ),
            _ => unreachable!(),
        };
        let fields = match input.next() {
            Some(TokenTree::Group(group)) => c_initializer_fields(group.stream())?,
            _ => unreachable!(),
        };
        return Ok(rust_call(
            "crate::backend::c::syntax::Expr::compound_literal",
            [ty, fields],
        ));
    }

    let mut expression = match input.next() {
        Some(TokenTree::Ident(keyword)) if keyword.to_string() == "if" => {
            let condition_start = input.position;
            while !matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
            {
                if input.next().is_none() {
                    return Err("expected then expression after C conditional".into());
                }
            }
            let condition =
                c_expression(token_slice(&input.tokens[condition_start..input.position]))?;
            let then_value = match input.next() {
                Some(TokenTree::Group(group)) => c_expression(group.stream())?,
                _ => unreachable!(),
            };
            match input.next() {
                Some(TokenTree::Ident(keyword)) if keyword.to_string() == "else" => {}
                _ => return Err("expected `else` in C conditional expression".into()),
            }
            let otherwise = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    c_expression(group.stream())?
                }
                _ => return Err("expected else expression in C conditional".into()),
            };
            rust_call(
                "crate::backend::c::syntax::Expr::conditional",
                [condition, then_value, otherwise],
            )
        }
        Some(TokenTree::Ident(identifier)) => {
            let name = identifier.to_string();
            if name == "sizeof"
                && matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis)
            {
                let value = match input.next() {
                    Some(TokenTree::Group(group)) => c_expression(group.stream())?,
                    _ => unreachable!(),
                };
                rust_call("crate::backend::c::syntax::Expr::sizeof_value", [value])
            } else if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
            {
                let fields = match input.next() {
                    Some(TokenTree::Group(group)) => c_initializer_fields(group.stream())?,
                    _ => unreachable!(),
                };
                rust_call(
                    "crate::backend::c::syntax::Expr::compound_literal",
                    [
                        rust_call(
                            "crate::backend::c::syntax::TypeName::named",
                            [format!("{name:?}").parse().unwrap()],
                        ),
                        fields,
                    ],
                )
            } else {
                rust_call(
                    "crate::backend::c::syntax::Expr::identifier",
                    [format!("{:?}", name.trim_start_matches("r#"))
                        .parse()
                        .unwrap()],
                )
            }
        }
        Some(TokenTree::Literal(literal)) => {
            let spelling = literal.to_string();
            if spelling.starts_with('"')
                || spelling.starts_with("r\"")
                || spelling.starts_with("r#")
            {
                rust_call(
                    "crate::backend::c::syntax::Expr::string",
                    [[TokenTree::Literal(literal)].into_iter().collect()],
                )
            } else {
                rust_call(
                    "crate::backend::c::syntax::Expr::number",
                    [format!("{spelling:?}").parse().unwrap()],
                )
            }
        }
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            c_expression(group.stream())?
        }
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => rust_call(
            "crate::backend::c::syntax::IntoExpr::into_expr",
            [group.stream()],
        ),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => rust_call(
            "crate::backend::c::syntax::Expr::initializer_list",
            [c_expression_list(group.stream())?],
        ),
        Some(token) => return Err(format!("unsupported token in C expression: {token}")),
        None => return Err("expected C expression".into()),
    };

    loop {
        match input.peek() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                let group = match input.next() {
                    Some(TokenTree::Group(group)) => group,
                    _ => unreachable!(),
                };
                expression = rust_call(
                    "crate::backend::c::syntax::Expr::call",
                    [expression, c_expression_list(group.stream())?],
                );
            }
            Some(TokenTree::Punct(token)) if token.as_char() == '.' => {
                input.next();
                let name = match input.next() {
                    Some(TokenTree::Ident(name)) => {
                        format!("{:?}", name.to_string()).parse().unwrap()
                    }
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                        group.stream()
                    }
                    _ => return Err("expected field name after `.`".into()),
                };
                expression =
                    rust_call("crate::backend::c::syntax::Expr::field", [expression, name]);
            }
            _ => break,
        }
    }
    Ok(expression)
}

pub(super) fn c_initializer_fields(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C initializer splice".into());
            }
            let values = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after initializer splice".into()),
            };
            operations.extend(tokens("__mal_backend_initializers.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, values))]);
            operations.extend(tokens(";"));
            if !input.eat_punct(',') && !input.is_empty() {
                return Err("expected `,` after C initializer splice".into());
            }
            continue;
        }
        if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
        {
            let value = match input.next() {
                Some(TokenTree::Group(group)) => group.stream(),
                _ => unreachable!(),
            };
            let initializer = if input.eat_punct(':') {
                let start = input.position;
                while !input.is_empty()
                    && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == ',')
                {
                    input.position += 1;
                }
                rust_call(
                    "crate::backend::c::syntax::Initializer::designated",
                    [
                        value,
                        c_expression(token_slice(&input.tokens[start..input.position]))?,
                    ],
                )
            } else {
                rust_call("crate::backend::c::syntax::Initializer::from", [value])
            };
            operations.extend(tokens("__mal_backend_initializers.push"));
            operations.extend([TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                initializer,
            ))]);
            operations.extend(tokens(";"));
            if !input.eat_punct(',') && !input.is_empty() {
                return Err("expected `,` after embedded C initializer".into());
            }
            continue;
        }
        let name = match input.next() {
            Some(TokenTree::Ident(name)) => name.to_string(),
            _ => return Err("expected compound-literal field name".into()),
        };
        let mut path = vec![name];
        while input.eat_punct('.') {
            match input.next() {
                Some(TokenTree::Ident(name)) => path.push(name.to_string()),
                _ => return Err("expected field name after initializer `.`".into()),
            }
        }
        if !input.eat_punct(':') {
            return Err("expected `:` after compound-literal field name".into());
        }
        let start = input.position;
        while !input.is_empty()
            && !matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == ',')
        {
            input.position += 1;
        }
        let value = c_expression(token_slice(&input.tokens[start..input.position]))?;
        let initializer = if path.len() == 1
            && (path[0] == "_"
                || path[0].starts_with('_')
                    && path[0][1..].bytes().all(|byte| byte.is_ascii_digit()))
        {
            rust_call(
                "crate::backend::c::syntax::Initializer::positional",
                [value],
            )
        } else if path.len() == 1 {
            rust_call(
                "crate::backend::c::syntax::Initializer::designated",
                [format!("{:?}", path[0]).parse().unwrap(), value],
            )
        } else {
            rust_call(
                "crate::backend::c::syntax::Initializer::designated_path",
                [
                    rust_array(
                        path.into_iter()
                            .map(|name| format!("{name:?}").parse().unwrap()),
                    ),
                    value,
                ],
            )
        };
        operations.extend(tokens("__mal_backend_initializers.push"));
        operations.extend([TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            initializer,
        ))]);
        operations.extend(tokens(";"));
        if !input.eat_punct(',') && !input.is_empty() {
            return Err("expected `,` between compound-literal fields".into());
        }
    }
    let mut body = tokens("let mut __mal_backend_initializers = Vec::new();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_initializers"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}

pub(super) fn c_expression_list(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C expression splice".into());
            }
            let values = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after expression splice".into()),
            };
            operations.extend(tokens("__mal_backend_expressions.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, values))]);
            operations.extend(tokens(";"));
            if !input.eat_punct(',') && !input.is_empty() {
                return Err("expected `,` after C expression splice".into());
            }
            continue;
        }
        let start = input.position;
        let mut depth = 0_usize;
        while input.position < input.tokens.len() {
            match &input.tokens[input.position] {
                TokenTree::Punct(token) if token.as_char() == ',' && depth == 0 => break,
                TokenTree::Punct(token) if matches!(token.as_char(), '<' | '[' | '(' | '{') => {
                    depth += 1
                }
                TokenTree::Punct(token) if matches!(token.as_char(), '>' | ']' | ')' | '}') => {
                    depth = depth.saturating_sub(1)
                }
                _ => {}
            }
            input.position += 1;
        }
        let item = input.tokens[start..input.position]
            .iter()
            .cloned()
            .collect();
        let expression = c_expression(item)?;
        operations.extend(tokens("__mal_backend_expressions.push"));
        operations.extend([TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            expression,
        ))]);
        operations.extend(tokens(";"));
        if !input.eat_punct(',') && !input.is_empty() {
            return Err("expected `,` between C call arguments".into());
        }
    }
    let mut body = tokens("let mut __mal_backend_expressions = Vec::new();");
    body.extend(operations);
    body.extend(tokens("__mal_backend_expressions"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}
