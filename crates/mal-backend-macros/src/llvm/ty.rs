//! LLVM types, parameters, and function attributes.

use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::support::*;

pub(super) fn llvm_type_tokens(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let token = input.next().ok_or("expected LLVM type")?;
    if let Some(expression) = rust_expression(&token) {
        if !input.is_empty() {
            return Err("unexpected token after embedded LLVM type".into());
        }
        return Ok(expression);
    }
    let TokenTree::Ident(kind) = token else {
        return Err("expected LLVM type".into());
    };
    let kind = kind.to_string();
    let result = match kind.as_str() {
        "void" => code(format!("{SYNTAX}::Type::Void")),
        "ptr" => code(format!("{SYNTAX}::Type::Pointer")),
        "float" => code(format!("{SYNTAX}::Type::Float")),
        "double" => code(format!("{SYNTAX}::Type::Double")),
        "int" => {
            let arguments = group(
                input.next().ok_or("expected int width")?,
                Delimiter::Parenthesis,
                "int width",
            )?;
            let bits = scalar(one_token(arguments.stream(), "int width")?);
            code(format!("{SYNTAX}::Type::integer({bits})"))
        }
        "array" => {
            let arguments = group(
                input.next().ok_or("expected array arguments")?,
                Delimiter::Parenthesis,
                "array arguments",
            )?;
            let mut arguments = split(arguments.stream(), ',').into_iter();
            let length = scalar(one_token(
                arguments.next().ok_or("expected array length")?,
                "array length",
            )?);
            let element = llvm_type_tokens(arguments.next().ok_or("expected array element type")?)?;
            if arguments.any(|argument| !argument.is_empty()) {
                return Err("unexpected array argument".into());
            }
            code(format!("{SYNTAX}::Type::array({length}, {element})"))
        }
        "structure" => {
            let arguments = group(
                input.next().ok_or("expected structure fields")?,
                Delimiter::Parenthesis,
                "structure fields",
            )?;
            let fields = group(
                one_token(arguments.stream(), "structure field list")?,
                Delimiter::Bracket,
                "structure field list",
            )?;
            let fields = vector(fields.stream(), llvm_type_tokens)?;
            code(format!("{SYNTAX}::Type::structure({fields})"))
        }
        _ => return Err(format!("unsupported LLVM type `{kind}`")),
    };
    if !input.is_empty() {
        return Err("unexpected token after LLVM type".into());
    }
    Ok(result)
}

pub(super) fn wrapped_type(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    llvm_type_tokens(input)
}

pub(super) fn parameter_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let mut tokens = input.into_iter().collect::<Vec<_>>();
    let immarg = if matches!(tokens.first(), Some(TokenTree::Punct(token)) if token.as_char() == '#')
    {
        if tokens.len() < 2 {
            return Err("expected parameter attribute".into());
        }
        let attribute = group(tokens.remove(1), Delimiter::Bracket, "parameter attribute")?;
        tokens.remove(0);
        if attribute.stream().to_string() != "immarg" {
            return Err("unsupported LLVM parameter attribute".into());
        }
        true
    } else {
        false
    };
    let colon = tokens
        .iter()
        .position(|token| matches!(token, TokenTree::Punct(token) if token.as_char() == ':'))
        .ok_or("expected `:` in LLVM parameter")?;
    let name = tokens[..colon].iter().cloned().collect::<TokenStream>();
    let ty = tokens[colon + 1..].iter().cloned().collect::<TokenStream>();
    let ty = llvm_type_tokens(ty)?;
    let parameter = if name.to_string() == "_" {
        code(format!("{SYNTAX}::Parameter::unnamed({ty})"))
    } else {
        let name = scalar(one_token(name, "parameter name")?);
        code(format!("{SYNTAX}::Parameter::named({ty}, {name})"))
    };
    if immarg {
        Ok(code(format!(
            "({parameter}).with_attribute({SYNTAX}::ParameterAttribute::ImmArg)"
        )))
    } else {
        Ok(parameter)
    }
}

pub(super) fn parameters(input: TokenStream) -> Result<TokenStream, String> {
    vector(input, parameter_tokens)
}

fn function_attribute(item: TokenStream) -> Result<TokenStream, String> {
    let name = one_token(item, "LLVM function attribute")?.to_string();
    let variant = match name.as_str() {
        "nofree" => "NoFree",
        "noinline" => "NoInline",
        "nounwind" => "NoUnwind",
        "willreturn" => "WillReturn",
        "memory_none" => "MemoryNone",
        "memory_argmem_read" => "MemoryArgMemRead",
        _ => return Err(format!("unsupported LLVM function attribute `{name}`")),
    };
    Ok(code(format!("{SYNTAX}::FunctionAttribute::{variant}")))
}

pub(super) fn attributes(input: TokenStream) -> Result<TokenStream, String> {
    vector(input, function_attribute)
}
