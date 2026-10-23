//! Translation-unit and preprocessor items.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::expression::{c_expression, c_expression_list};
use super::record::{c_function_item, c_record_item, c_type_alias_item};
use super::support::*;

pub(super) fn c_items_syntax(input: TokenStream) -> Result<TokenStream, String> {
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

pub(super) fn c_invocation_syntax(input: TokenStream) -> Result<TokenStream, String> {
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
