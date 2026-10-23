//! LLVM globals and metadata.

use proc_macro::{Delimiter, TokenStream};

use super::support::*;
use super::ty::*;
use super::value::*;

pub(super) fn global_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let (name, body) = named_form(input)?;
    if name != "byte_owner" {
        return Err(format!("unsupported LLVM global `{name}`"));
    }
    let values = fields(body.stream(), &["name", "bytes", "alignment"])?;
    let name = scalar(one_token(values[0].clone(), "global name")?);
    let bytes = embedded(&values[1]).ok_or("global bytes must be embedded")?;
    let alignment = scalar(one_token(values[2].clone(), "global alignment")?);
    Ok(code(format!(
        "{SYNTAX}::GlobalDefinition::byte_owner({name}, {bytes}, {alignment})"
    )))
}

pub(super) fn metadata_operand_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let tokens = input.into_iter().collect::<Vec<_>>();
    if tokens.len() != 2 {
        return Err("expected LLVM metadata operand".into());
    }
    let name = tokens[0].to_string();
    let arguments = group(
        tokens[1].clone(),
        Delimiter::Parenthesis,
        "metadata operand",
    )?;
    match name.as_str() {
        "node" => {
            let id = scalar(one_token(arguments.stream(), "metadata node id")?);
            Ok(code(format!("{SYNTAX}::MetadataOperand::Node({id})")))
        }
        "text" => {
            let text = scalar(one_token(arguments.stream(), "metadata text")?);
            Ok(code(format!(
                "{SYNTAX}::MetadataOperand::Text(({text}).into())"
            )))
        }
        "integer" => {
            let mut values = comma_items(arguments.stream())?.into_iter();
            let ty_input = values.next().ok_or("expected metadata integer type")?;
            let ty = if embedded(&ty_input).is_some() {
                embedded(&ty_input).unwrap()
            } else {
                llvm_type_tokens(ty_input)?
            };
            let value = scalar(one_token(
                values.next().ok_or("expected metadata integer value")?,
                "metadata integer value",
            )?);
            if values.next().is_some() {
                return Err("unexpected metadata integer field".into());
            }
            Ok(code(format!(
                "{SYNTAX}::MetadataOperand::Integer {{ ty: {ty}, value: {value} }}"
            )))
        }
        _ => Err(format!("unsupported metadata operand `{name}`")),
    }
}

pub(super) fn metadata_tokens(input: TokenStream) -> Result<TokenStream, String> {
    let body = group(
        one_token(input, "metadata definition")?,
        Delimiter::Brace,
        "metadata definition",
    )?;
    let values = fields(body.stream(), &["id", "distinct", "operands"])?;
    let id = scalar(one_token(values[0].clone(), "metadata id")?);
    let distinct = scalar(one_token(values[1].clone(), "metadata distinct flag")?);
    let operands = group(
        one_token(values[2].clone(), "metadata operands")?,
        Delimiter::Bracket,
        "metadata operands",
    )?;
    let operands = vector(operands.stream(), metadata_operand_tokens)?;
    Ok(code(format!(
        "{SYNTAX}::MetadataDefinition::new({id}, {distinct}, {operands})"
    )))
}
