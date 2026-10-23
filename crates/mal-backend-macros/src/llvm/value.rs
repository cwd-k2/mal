//! LLVM values and constants.

use proc_macro::{Delimiter, Group, TokenStream};

use crate::shared::Cursor;

use super::support::*;
use super::ty::*;

pub(super) fn typed_value_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(code(format!("Some({expression})")));
    }
    let tuple = group(
        one_token(input, "typed LLVM value")?,
        Delimiter::Parenthesis,
        "typed LLVM value",
    )?;
    let mut parts = comma_items(tuple.stream())?.into_iter();
    let ty = wrapped_type(parts.next().ok_or("expected typed value type")?)?;
    let value = scalar(one_token(
        parts.next().ok_or("expected typed value operand")?,
        "typed value operand",
    )?);
    if parts.next().is_some() {
        return Err("unexpected typed value field".into());
    }
    Ok(code(format!("{SYNTAX}::TypedValue::new({ty}, {value})")))
}

pub(super) fn typed_values(input: TokenStream) -> Result<TokenStream, String> {
    fallible_vector(input, typed_value_tokens)
}

pub(super) fn typed_constant_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(code(format!("Some({expression})")));
    }
    let tuple = group(
        one_token(input, "typed LLVM constant")?,
        Delimiter::Parenthesis,
        "typed LLVM constant",
    )?;
    let mut parts = comma_items(tuple.stream())?.into_iter();
    let ty = wrapped_type(parts.next().ok_or("expected typed constant type")?)?;
    let value = parts.next().ok_or("expected typed constant value")?;
    if parts.next().is_some() {
        return Err("unexpected typed constant field".into());
    }
    if let Some(expression) = embedded(&value) {
        return Ok(code(format!(
            "Some({SYNTAX}::TypedConstant::new({ty}, {expression}))"
        )));
    }
    let value = constant_tokens(value)?;
    Ok(code(format!(
        "({value}).map(|__mal_constant| {SYNTAX}::TypedConstant::new({ty}, __mal_constant))"
    )))
}

pub(super) fn typed_constants(input: TokenStream) -> Result<TokenStream, String> {
    fallible_vector(input, typed_constant_tokens)
}

pub(super) fn named_form(input: TokenStream) -> Result<(String, Group), String> {
    let mut input = Cursor::new(input);
    let name = input.next().ok_or("expected LLVM form")?.to_string();
    let body = group(
        input.next().ok_or("expected LLVM form body")?,
        Delimiter::Brace,
        "LLVM form body",
    )?;
    if !input.is_empty() {
        return Err("unexpected token after LLVM form".into());
    }
    Ok((name, body))
}

pub(super) fn constant_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(code(format!("Some({expression})")));
    }
    let tokens = input.clone().into_iter().collect::<Vec<_>>();
    if tokens.len() == 1 && tokens[0].to_string() == "zero" {
        return Ok(code(format!("Some({SYNTAX}::Constant::ZeroInitializer)")));
    }
    if tokens.len() == 2 && tokens[0].to_string() == "atom" {
        let arguments = group(tokens[1].clone(), Delimiter::Parenthesis, "atom operand")?;
        let value = scalar(one_token(arguments.stream(), "atom operand")?);
        return Ok(code(format!(
            "{SYNTAX}::Constant::atom(({value}).to_string())"
        )));
    }
    if tokens.len() == 2 && tokens[0].to_string() == "structure" {
        let arguments = group(tokens[1].clone(), Delimiter::Parenthesis, "constant fields")?;
        let fields = group(
            one_token(arguments.stream(), "constant field list")?,
            Delimiter::Bracket,
            "constant field list",
        )?;
        let fields = typed_constants(fields.stream())?;
        return Ok(code(format!(
            "({fields}).map({SYNTAX}::Constant::structure)"
        )));
    }
    let (name, body) = named_form(input)?;
    match name.as_str() {
        "get_element_ptr" => {
            let values = fields(body.stream(), &["element_type", "pointer", "indices"])?;
            let element_type = wrapped_type(values[0].clone())?;
            let pointer = typed_constant_tokens(values[1].clone())?;
            let indices = group(
                one_token(values[2].clone(), "constant indices")?,
                Delimiter::Bracket,
                "constant indices",
            )?;
            let indices = typed_constants(indices.stream())?;
            Ok(code(format!(
                "({pointer}).and_then(|__mal_pointer| ({indices}).map(|__mal_indices| {SYNTAX}::Constant::get_element_ptr({element_type}, __mal_pointer, __mal_indices)))"
            )))
        }
        "unary" => {
            let values = fields(body.stream(), &["operator", "operand"])?;
            let operator = scalar(one_token(values[0].clone(), "constant unary operator")?);
            if let Some(operand) = embedded(&values[1]) {
                Ok(code(format!(
                    "Some({SYNTAX}::Constant::unary({operator}, {operand}))"
                )))
            } else {
                let operand = typed_constant_tokens(values[1].clone())?;
                Ok(code(format!(
                    "({operand}).map(|__mal_operand| {SYNTAX}::Constant::unary({operator}, __mal_operand))"
                )))
            }
        }
        "binary" => {
            let values = fields(body.stream(), &["operator", "left", "right"])?;
            let operator = scalar(one_token(values[0].clone(), "constant binary operator")?);
            if let (Some(left), Some(right)) = (embedded(&values[1]), embedded(&values[2])) {
                Ok(code(format!(
                    "{SYNTAX}::Constant::binary({operator}, {left}, {right})"
                )))
            } else {
                let left = typed_constant_tokens(values[1].clone())?;
                let right = typed_constant_tokens(values[2].clone())?;
                Ok(code(format!(
                    "({left}).zip({right}).and_then(|(__mal_left, __mal_right)| {SYNTAX}::Constant::binary({operator}, __mal_left, __mal_right))"
                )))
            }
        }
        "cast" => {
            let values = fields(body.stream(), &["operator", "operand", "to"])?;
            let operator = scalar(one_token(values[0].clone(), "constant cast operator")?);
            let operand = embedded(&values[1]).ok_or("constant cast operand must be embedded")?;
            let target = wrapped_type(values[2].clone())?;
            Ok(code(format!(
                "Some({SYNTAX}::Constant::cast({operator}, {operand}, {target}))"
            )))
        }
        _ => Err(format!("unsupported LLVM constant `{name}`")),
    }
}

pub(super) fn sequence(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let list = group(
        one_token(input, "LLVM sequence")?,
        Delimiter::Bracket,
        "LLVM sequence",
    )?;
    if list.stream().is_empty() {
        Ok(code("[]"))
    } else {
        Err("only an empty static LLVM sequence is supported".into())
    }
}

pub(super) fn indices(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let list = group(
        one_token(input, "LLVM index list")?,
        Delimiter::Bracket,
        "LLVM index list",
    )?;
    let values = comma_items(list.stream())?
        .into_iter()
        .map(|item| Ok(scalar(one_token(item, "LLVM index")?).to_string()))
        .collect::<Result<Vec<_>, String>>()?;
    Ok(code(format!("[{}]", values.join(","))))
}

pub(super) fn callee(input: TokenStream) -> Result<TokenStream, String> {
    let tokens = input.into_iter().collect::<Vec<_>>();
    if tokens.len() != 2 {
        return Err("expected direct(...) or indirect(...) callee".into());
    }
    let constructor = match tokens[0].to_string().as_str() {
        "direct" => "direct",
        "indirect" => "indirect",
        _ => return Err("expected direct(...) or indirect(...) callee".into()),
    };
    let value = group(tokens[1].clone(), Delimiter::Parenthesis, "callee operand")?;
    let value = scalar(one_token(value.stream(), "callee operand")?);
    Ok(code(format!("{SYNTAX}::Callee::{constructor}({value})")))
}
