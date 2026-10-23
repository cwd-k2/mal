//! LLVM function signatures and declarations.

use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::support::*;
use super::ty::*;

struct FunctionHeader {
    name: TokenStream,
    parameters: TokenStream,
    result: TokenStream,
    attributes: TokenStream,
    internal: bool,
}

fn function_header(input: TokenStream, declaration: bool) -> Result<FunctionHeader, String> {
    let mut input = Cursor::new(input);
    let mut internal = false;
    let mut function_attributes = TokenStream::new();
    while matches!(input.peek(), Some(TokenTree::Punct(token)) if token.as_char() == '#') {
        input.next();
        let attribute = group(
            input.next().ok_or("expected function attribute")?,
            Delimiter::Bracket,
            "function attribute",
        )?;
        let mut attribute = Cursor::new(attribute.stream());
        let name = attribute
            .next()
            .ok_or("expected function attribute name")?
            .to_string();
        match name.as_str() {
            "linkage" if !declaration => {
                let value = group(
                    attribute.next().ok_or("expected linkage")?,
                    Delimiter::Parenthesis,
                    "linkage",
                )?;
                if value.stream().to_string() != "internal" || !attribute.is_empty() {
                    return Err("only internal LLVM linkage is supported".into());
                }
                internal = true;
            }
            "attributes" => {
                let value = group(
                    attribute.next().ok_or("expected attribute list")?,
                    Delimiter::Parenthesis,
                    "attribute list",
                )?;
                if !attribute.is_empty() {
                    return Err("unexpected token after LLVM attribute list".into());
                }
                function_attributes = value.stream();
            }
            _ => return Err(format!("unsupported LLVM function annotation `{name}`")),
        }
    }
    if input.next().map(|token| token.to_string()) != Some("fn".into()) {
        return Err("expected `fn`".into());
    }
    let name = scalar(input.next().ok_or("expected function name")?);
    let parameter_group = group(
        input.next().ok_or("expected function parameters")?,
        Delimiter::Parenthesis,
        "function parameters",
    )?;
    if !input.eat_punct('-') || !input.eat_punct('>') {
        return Err("expected `->`".into());
    }
    let mut result = input.tokens[input.position..].to_vec();
    if declaration
        && matches!(result.last(), Some(TokenTree::Punct(token)) if token.as_char() == ';')
    {
        result.pop();
    } else if declaration {
        return Err("expected `;` after LLVM declaration".into());
    }
    let result = llvm_type_tokens(result.into_iter().collect())?;
    Ok(FunctionHeader {
        name,
        parameters: parameters(parameter_group.stream())?,
        result,
        attributes: attributes(function_attributes)?,
        internal,
    })
}

pub(super) fn signature_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let header = function_header(input, false)?;
    let mut signature = format!(
        "{SYNTAX}::FunctionSignature::new({}, {}, {}).with_attributes({})",
        header.result, header.name, header.parameters, header.attributes
    );
    if header.internal {
        signature.push_str(&format!(".with_linkage({SYNTAX}::Linkage::Internal)"));
    }
    Ok(code(signature))
}

pub(super) fn declaration_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let header = function_header(input, true)?;
    Ok(code(format!(
        "{SYNTAX}::FunctionDeclaration::new({}, {}, {}).with_attributes({})",
        header.result, header.name, header.parameters, header.attributes
    )))
}
