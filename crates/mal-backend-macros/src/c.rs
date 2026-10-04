//! Parser for the Rust-shaped C construction language.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::{Cursor, parse};

fn tokens(source: &str) -> TokenStream {
    source.parse().expect("generated Rust tokens parse")
}

fn comma_separated(items: impl IntoIterator<Item = TokenStream>) -> TokenStream {
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

fn rust_call(path: &str, arguments: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let mut output = tokens(path);
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        comma_separated(arguments),
    ))]);
    output
}

fn rust_method_call(receiver: TokenStream, method: &str) -> TokenStream {
    let mut output = TokenStream::new();
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        receiver,
    ))]);
    output.extend(tokens(&format!(".{method}()")));
    output
}

fn rust_method_suffix(receiver: TokenStream, suffix: &str) -> TokenStream {
    let mut output = TokenStream::new();
    output.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        receiver,
    ))]);
    output.extend(tokens(suffix));
    output
}

fn c_expression(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_initializer_fields(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_expression_list(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_type_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let ty = c_type_tokens(&mut input)?;
    if input.is_empty() {
        Ok(ty)
    } else {
        Err("unexpected token after C type".into())
    }
}

fn c_type_tokens(input: &mut Cursor) -> Result<TokenStream, String> {
    if input.eat_punct('*') {
        let is_const = match input.next() {
            Some(TokenTree::Ident(qualifier)) if qualifier.to_string() == "const" => true,
            Some(TokenTree::Ident(qualifier)) if qualifier.to_string() == "mut" => false,
            _ => return Err("expected `const` or `mut` after `*` in C type".into()),
        };
        let pointee = c_type_tokens(input)?;
        return Ok(rust_method_call(
            pointee,
            if is_const {
                "const_pointee_pointer"
            } else {
                "pointer"
            },
        ));
    }

    match input.next() {
        Some(TokenTree::Ident(identifier)) if identifier.to_string() == "Struct" => {
            if !input.eat_punct('<') {
                return Err("expected `<` after `Struct`".into());
            }
            let name = match input.next() {
                Some(TokenTree::Ident(name)) => format!("{:?}", name.to_string()).parse().unwrap(),
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected C struct tag".into()),
            };
            if !input.eat_punct('>') {
                return Err("expected `>` after C struct tag".into());
            }
            Ok(rust_call(
                "crate::backend::c::syntax::TypeName::structure",
                [name],
            ))
        }
        Some(TokenTree::Ident(identifier)) => Ok(rust_call(
            "crate::backend::c::syntax::TypeName::named",
            [
                format!("{:?}", identifier.to_string().trim_start_matches("r#"))
                    .parse()
                    .unwrap(),
            ],
        )),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => Ok(rust_call(
            "crate::backend::c::syntax::TypeName::from",
            [group.stream()],
        )),
        _ => Err("expected C type".into()),
    }
}

fn token_slice(tokens: &[TokenTree]) -> TokenStream {
    tokens.iter().cloned().collect()
}

fn take_until_punct(input: &mut Cursor, punctuation: char) -> Result<TokenStream, String> {
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

fn c_statement_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_switch_arms(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_block_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_name(input: &mut Cursor, description: &str) -> Result<TokenStream, String> {
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

fn c_parameters_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_signature_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let signature = c_signature_tokens(&mut input)?;
    if input.is_empty() {
        Ok(signature)
    } else {
        Err("unexpected token after C signature".into())
    }
}

fn c_function_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_items_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let mut operations = TokenStream::new();
    while !input.is_empty() {
        if input.eat_punct('.') {
            if !input.eat_punct('.') {
                return Err("expected `..` for C item splice".into());
            }
            let items = match input.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected Rust block after C item splice".into()),
            };
            input.eat_punct(';');
            operations.extend(tokens("__mal_backend_items.extend"));
            operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, items))]);
            operations.extend(tokens(";"));
            continue;
        }
        if matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace)
        {
            let item = match input.next() {
                Some(TokenTree::Group(group)) => group.stream(),
                _ => unreachable!(),
            };
            input.eat_punct(';');
            push_item(&mut operations, item);
            continue;
        }

        let item = match input.peek() {
            Some(TokenTree::Punct(token)) if token.as_char() == '#' => c_function_item(&mut input)?,
            Some(TokenTree::Ident(name)) => match name.to_string().as_str() {
                "if" => {
                    operations.extend(c_conditional_items(&mut input)?);
                    continue;
                }
                "fn" => c_function_item(&mut input)?,
                "type" => c_type_alias_item(&mut input)?,
                "struct" => c_record_item(&mut input, "struct")?,
                "union" => c_record_item(&mut input, "union")?,
                "include_system" | "include_quoted" | "define" | "assert" | "comment" => {
                    c_item_macro(&mut input)?
                }
                name => return Err(format!("unsupported C item `{name}`")),
            },
            _ => return Err("expected C item".into()),
        };
        push_item(&mut operations, item);
    }
    let mut body = tokens(
        "let mut __mal_backend_items = crate::backend::c::syntax::TranslationUnit::default();",
    );
    body.extend(operations);
    body.extend(tokens("__mal_backend_items"));
    Ok([TokenTree::Group(Group::new(Delimiter::Brace, body))]
        .into_iter()
        .collect())
}

fn c_conditional_items(input: &mut Cursor) -> Result<TokenStream, String> {
    input.next();
    let negated = input.eat_punct('!');
    match input.next() {
        Some(TokenTree::Ident(name)) if name.to_string() == "defined" => {}
        _ => return Err("expected `defined(name)` after C item `if`".into()),
    }
    let name = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            let mut name = Cursor::new(group.stream());
            let value = c_name(&mut name, "preprocessor name")?;
            if !name.is_empty() {
                return Err("unexpected token after preprocessor name".into());
            }
            value
        }
        _ => return Err("expected preprocessor name in parentheses".into()),
    };
    let then_items = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            c_items_syntax(group.stream())?
        }
        _ => return Err("expected item block after preprocessor condition".into()),
    };
    let otherwise = if matches!(input.peek(), Some(TokenTree::Ident(name)) if name.to_string() == "else")
    {
        input.next();
        match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                Some(c_items_syntax(group.stream())?)
            }
            _ => return Err("expected item block after `else`".into()),
        }
    } else {
        None
    };

    let mut operations = TokenStream::new();
    let directive = if negated {
        rust_call("crate::backend::c::syntax::Directive::ifndef", [name])
    } else {
        rust_call("crate::backend::c::syntax::Directive::if_defined", [name])
    };
    push_item(&mut operations, directive);
    operations.extend(tokens("__mal_backend_items.extend"));
    operations.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        then_items,
    ))]);
    operations.extend(tokens(";"));
    if let Some(otherwise) = otherwise {
        push_item(
            &mut operations,
            tokens("crate::backend::c::syntax::Directive::Else"),
        );
        operations.extend(tokens("__mal_backend_items.extend"));
        operations.extend([TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            otherwise,
        ))]);
        operations.extend(tokens(";"));
    }
    push_item(
        &mut operations,
        tokens("crate::backend::c::syntax::Directive::Endif"),
    );
    Ok(operations)
}

fn c_invocation_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let name = c_name(&mut input, "macro name")?;
    let arguments = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            c_expression_list(group.stream())?
        }
        _ => return Err("expected macro invocation arguments".into()),
    };
    input.eat_punct(';');
    if !input.is_empty() {
        return Err("unexpected token after macro invocation".into());
    }
    Ok(rust_call(
        "crate::backend::c::syntax::MacroInvocation::new",
        [name, arguments],
    ))
}

fn push_item(operations: &mut TokenStream, item: TokenStream) {
    operations.extend(tokens("__mal_backend_items.push"));
    operations.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, item))]);
    operations.extend(tokens(";"));
}

fn c_item_macro(input: &mut Cursor) -> Result<TokenStream, String> {
    let name = match input.next() {
        Some(TokenTree::Ident(name)) => name.to_string(),
        _ => unreachable!(),
    };
    if !input.eat_punct('!') {
        return Err(format!("expected `!` after `{name}`"));
    }
    let arguments = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            group.stream()
        }
        _ => return Err(format!("expected arguments after `{name}!`")),
    };
    if !input.eat_punct(';') {
        return Err(format!("expected `;` after `{name}!`"));
    }
    match name.as_str() {
        "include_system" | "include_quoted" => {
            let path = c_include_path(arguments)?;
            Ok(rust_call(
                if name == "include_system" {
                    "crate::backend::c::syntax::Directive::include_system"
                } else {
                    "crate::backend::c::syntax::Directive::include_quoted"
                },
                [path],
            ))
        }
        "define" => c_define_item(arguments),
        "assert" => {
            let mut arguments = split_c(arguments, ',').into_iter();
            let condition = c_expression(arguments.next().ok_or("expected assert condition")?)?;
            let message = scalar_value(one_c_token(
                arguments.next().ok_or("expected assert message")?,
                "assert message",
            )?);
            if arguments.any(|argument| !argument.is_empty()) {
                return Err("unexpected C assert argument".into());
            }
            Ok(rust_call(
                "crate::backend::c::syntax::Declaration::static_assert",
                [condition, message],
            ))
        }
        "comment" => Ok(rust_call(
            "crate::backend::c::syntax::Comment::new",
            [scalar_value(one_c_token(arguments, "comment")?)],
        )),
        _ => unreachable!(),
    }
}

fn c_include_path(input: TokenStream) -> Result<TokenStream, String> {
    let input = input.into_iter().collect::<Vec<_>>();
    if let [TokenTree::Group(group)] = input.as_slice()
        && group.delimiter() == Delimiter::Brace
    {
        return Ok(group.stream());
    }
    if let [TokenTree::Literal(literal)] = input.as_slice() {
        let literal = literal.to_string();
        if literal.starts_with('"') || literal.starts_with("r\"") || literal.starts_with("r#") {
            return literal
                .parse()
                .map_err(|_| "invalid include path literal".into());
        }
    }
    let mut path = String::new();
    for token in input {
        match token {
            TokenTree::Ident(identifier) => path.push_str(&identifier.to_string()),
            TokenTree::Punct(punctuation)
                if matches!(punctuation.as_char(), '.' | '/' | '-' | '+') =>
            {
                path.push(punctuation.as_char());
            }
            _ => return Err("expected a header path or `{ rust_expression }`".into()),
        }
    }
    if path.is_empty() {
        return Err("expected include path".into());
    }
    Ok(format!("{path:?}").parse().unwrap())
}

fn scalar_value(token: TokenTree) -> TokenStream {
    match token {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => group.stream(),
        token => [token].into_iter().collect(),
    }
}

fn one_c_token(input: TokenStream, description: &str) -> Result<TokenTree, String> {
    let mut input = input.into_iter();
    let token = input
        .next()
        .ok_or_else(|| format!("expected {description}"))?;
    if input.next().is_some() {
        return Err(format!("expected one {description}"));
    }
    Ok(token)
}

fn split_c(input: TokenStream, separator: char) -> Vec<TokenStream> {
    let mut result = Vec::new();
    let mut current = TokenStream::new();
    for token in input {
        if matches!(&token, TokenTree::Punct(punctuation) if punctuation.as_char() == separator) {
            result.push(current);
            current = TokenStream::new();
        } else {
            current.extend([token]);
        }
    }
    result.push(current);
    result
}

fn c_define_item(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let name = c_name(&mut input, "macro name")?;
    let function_like = matches!(input.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis);
    let parameters = if function_like {
        let group = match input.next() {
            Some(TokenTree::Group(group)) => group,
            _ => unreachable!(),
        };
        let parameters = split_c(group.stream(), ',')
            .into_iter()
            .filter(|parameter| !parameter.is_empty())
            .map(|parameter| {
                let token = one_c_token(parameter, "macro parameter")?;
                match token {
                    TokenTree::Ident(name) => Ok(format!("{:?}", name.to_string())
                        .parse::<TokenStream>()
                        .unwrap()),
                    _ => Err("expected macro parameter name".into()),
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        if parameters.is_empty() {
            tokens("Vec::<&str>::new()")
        } else {
            rust_array(parameters)
        }
    } else {
        tokens("Vec::<&str>::new()")
    };
    if !input.eat_punct('=') {
        if !input.is_empty() {
            return Err("unexpected token after empty C define".into());
        }
        if function_like {
            return Ok(rust_call(
                "crate::backend::c::syntax::Directive::invocations_define",
                [
                    name,
                    parameters,
                    tokens("Vec::<crate::backend::c::syntax::MacroInvocation>::new()"),
                ],
            ));
        }
        return Ok(rust_call(
            "crate::backend::c::syntax::Directive::define_empty",
            [name],
        ));
    }
    let expression = c_expression(token_slice(&input.tokens[input.position..]))?;
    if !function_like {
        Ok(rust_call(
            "crate::backend::c::syntax::Directive::define_expr",
            [name, expression],
        ))
    } else {
        Ok(rust_call(
            "crate::backend::c::syntax::Directive::expression_define",
            [name, parameters, expression],
        ))
    }
}

fn rust_array(items: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let mut output = TokenStream::new();
    let mut first = true;
    for item in items {
        if !first {
            output.extend(tokens(","));
        }
        first = false;
        output.extend(item);
    }
    [TokenTree::Group(Group::new(Delimiter::Bracket, output))]
        .into_iter()
        .collect()
}

fn c_type_alias_item(input: &mut Cursor) -> Result<TokenStream, String> {
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

fn c_record_item(input: &mut Cursor, expected: &str) -> Result<TokenStream, String> {
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

fn c_record_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_record_fields_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

fn c_function_item(input: &mut Cursor) -> Result<TokenStream, String> {
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

pub(super) fn c_expr(input: TokenStream) -> TokenStream {
    parse(input, c_expression)
}

pub(super) fn c_type(input: TokenStream) -> TokenStream {
    parse(input, c_type_syntax)
}

pub(super) fn c_statement(input: TokenStream) -> TokenStream {
    parse(input, c_statement_syntax)
}

pub(super) fn c_block(input: TokenStream) -> TokenStream {
    parse(input, c_block_syntax)
}

pub(super) fn c_items(input: TokenStream) -> TokenStream {
    parse(input, c_items_syntax)
}

pub(super) fn c_record_fields(input: TokenStream) -> TokenStream {
    parse(input, c_record_fields_syntax)
}

pub(super) fn c_record(input: TokenStream) -> TokenStream {
    parse(input, c_record_syntax)
}

pub(super) fn c_initializers(input: TokenStream) -> TokenStream {
    parse(input, c_initializer_fields)
}

pub(super) fn c_invocation(input: TokenStream) -> TokenStream {
    parse(input, c_invocation_syntax)
}

pub(super) fn c_switch_cases(input: TokenStream) -> TokenStream {
    parse(input, c_switch_arms)
}

pub(super) fn c_signature(input: TokenStream) -> TokenStream {
    parse(input, c_signature_syntax)
}

pub(super) fn c_parameter(input: TokenStream) -> TokenStream {
    parse(input, |input| {
        c_parameters_syntax(input).map(|parameters| {
            rust_method_suffix(
                parameters,
                ".into_iter().next().expect(\"one parsed C parameter\")",
            )
        })
    })
}

pub(super) fn c_parameters(input: TokenStream) -> TokenStream {
    parse(input, c_parameters_syntax)
}

pub(super) fn c_function(input: TokenStream) -> TokenStream {
    parse(input, c_function_syntax)
}
