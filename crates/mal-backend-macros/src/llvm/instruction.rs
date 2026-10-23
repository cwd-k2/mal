//! LLVM instructions and terminators.

use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::support::*;
use super::ty::*;
use super::value::*;

pub(super) fn instruction_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(code(format!("Some({expression})")));
    }
    let mut input = Cursor::new(input);
    let result = if matches!(input.peek(), Some(TokenTree::Ident(token)) if token.to_string() == "let")
    {
        input.next();
        let result = scalar(input.next().ok_or("expected instruction result")?);
        if !input.eat_punct('=') {
            return Err("expected `=` after instruction result".into());
        }
        Some(result)
    } else {
        None
    };
    let name = input.next().ok_or("expected LLVM instruction")?.to_string();
    let body = group(
        input.next().ok_or("expected instruction fields")?,
        Delimiter::Brace,
        "instruction fields",
    )?;
    if !input.eat_punct(';') || !input.is_empty() {
        return Err("expected `;` after LLVM instruction".into());
    }
    let required_result = || {
        result
            .clone()
            .ok_or_else(|| format!("`{name}` requires a result"))
    };
    match name.as_str() {
        "alloca" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["ty", "alignment"])?;
            let ty = wrapped_type(values[0].clone())?;
            let alignment = scalar(one_token(values[1].clone(), "alignment")?);
            Ok(code(format!(
                "{SYNTAX}::Instruction::alloca({result}, {ty}, {alignment})"
            )))
        }
        "load" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["ty", "pointer", "alignment", "metadata"])?;
            let ty = wrapped_type(values[0].clone())?;
            let pointer = scalar(one_token(values[1].clone(), "pointer")?);
            let alignment = scalar(one_token(values[2].clone(), "alignment")?);
            let metadata = sequence(values[3].clone())?;
            Ok(code(format!(
                "{SYNTAX}::Instruction::load({result}, {ty}, {pointer}, {alignment}, {metadata})"
            )))
        }
        "store" => {
            if result.is_some() {
                return Err("`store` does not produce a result".into());
            }
            let values = fields(
                body.stream(),
                &["value", "pointer", "alignment", "metadata"],
            )?;
            let tuple = group(
                one_token(values[0].clone(), "stored value")?,
                Delimiter::Parenthesis,
                "stored value",
            )?;
            let mut parts = comma_items(tuple.stream())?.into_iter();
            let ty = wrapped_type(parts.next().ok_or("expected stored value type")?)?;
            let value = scalar(one_token(
                parts.next().ok_or("expected stored value")?,
                "stored value",
            )?);
            if parts.next().is_some() {
                return Err("unexpected stored value field".into());
            }
            let pointer = scalar(one_token(values[1].clone(), "pointer")?);
            let alignment = scalar(one_token(values[2].clone(), "alignment")?);
            let metadata = sequence(values[3].clone())?;
            Ok(code(format!(
                "{SYNTAX}::Instruction::store({ty}, {value}, {pointer}, {alignment}, {metadata})"
            )))
        }
        "call" => {
            let values = fields(
                body.stream(),
                &["tail", "result_type", "callee", "arguments"],
            )?;
            let tail = scalar(one_token(values[0].clone(), "tail flag")?);
            let result_type = wrapped_type(values[1].clone())?;
            let callee = callee(values[2].clone())?;
            let arguments = group(
                one_token(values[3].clone(), "call arguments")?,
                Delimiter::Bracket,
                "call arguments",
            )?;
            let arguments = typed_values(arguments.stream())?;
            let result = result.map_or_else(
                || "Option::<String>::None".into(),
                |result| format!("Some({result})"),
            );
            Ok(code(format!(
                "({callee}).and_then(|__mal_callee| ({arguments}).and_then(|__mal_arguments| {SYNTAX}::Instruction::call({result}, {tail}, {result_type}, __mal_callee, __mal_arguments)))"
            )))
        }
        "unary" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["operator", "value"])?;
            let operator = scalar(one_token(values[0].clone(), "unary operator")?);
            let value = typed_value_tokens(values[1].clone())?;
            Ok(code(format!(
                "({value}).and_then(|__mal_value| {SYNTAX}::Instruction::unary({result}, {operator}, __mal_value))"
            )))
        }
        "cast" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["operator", "value", "to"])?;
            let operator = scalar(one_token(values[0].clone(), "cast operator")?);
            let value = typed_value_tokens(values[1].clone())?;
            let target = wrapped_type(values[2].clone())?;
            Ok(code(format!(
                "({value}).and_then(|__mal_value| {SYNTAX}::Instruction::cast({result}, {operator}, __mal_value, {target}))"
            )))
        }
        "extract_value" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["aggregate", "indices"])?;
            let aggregate = typed_value_tokens(values[0].clone())?;
            let indices = indices(values[1].clone())?;
            Ok(code(format!(
                "({aggregate}).and_then(|__mal_aggregate| {SYNTAX}::Instruction::extract_value({result}, __mal_aggregate, {indices}))"
            )))
        }
        "binary" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["operator", "ty", "left", "right"])?;
            let operator = scalar(one_token(values[0].clone(), "binary operator")?);
            let ty = wrapped_type(values[1].clone())?;
            let left = scalar(one_token(values[2].clone(), "left operand")?);
            let right = scalar(one_token(values[3].clone(), "right operand")?);
            Ok(code(format!(
                "{SYNTAX}::Instruction::binary({result}, {operator}, {ty}, {left}, {right})"
            )))
        }
        "compare" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["kind", "predicate", "ty", "left", "right"])?;
            let kind = scalar(one_token(values[0].clone(), "comparison kind")?);
            let predicate = scalar(one_token(values[1].clone(), "comparison predicate")?);
            let ty = wrapped_type(values[2].clone())?;
            let left = scalar(one_token(values[3].clone(), "left operand")?);
            let right = scalar(one_token(values[4].clone(), "right operand")?);
            Ok(code(format!(
                "{SYNTAX}::Instruction::compare({result}, {kind}, {predicate}, {ty}, {left}, {right})"
            )))
        }
        "get_element_ptr" => {
            let result = required_result()?;
            let values = fields(
                body.stream(),
                &["inbounds", "element_type", "pointer", "indices"],
            )?;
            let inbounds = scalar(one_token(values[0].clone(), "inbounds flag")?);
            let element_type = wrapped_type(values[1].clone())?;
            let pointer = scalar(one_token(values[2].clone(), "pointer")?);
            let indices = group(
                one_token(values[3].clone(), "GEP indices")?,
                Delimiter::Bracket,
                "GEP indices",
            )?;
            let indices = typed_values(indices.stream())?;
            Ok(code(format!(
                "({indices}).and_then(|__mal_indices| {SYNTAX}::Instruction::get_element_ptr({result}, {inbounds}, {element_type}, {pointer}, __mal_indices))"
            )))
        }
        "insert_value" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["aggregate", "element", "indices"])?;
            let aggregate = typed_value_tokens(values[0].clone())?;
            let element = typed_value_tokens(values[1].clone())?;
            let indices = indices(values[2].clone())?;
            Ok(code(format!(
                "({aggregate}).zip({element}).and_then(|(__mal_aggregate, __mal_element)| {SYNTAX}::Instruction::insert_value({result}, __mal_aggregate, __mal_element, {indices}))"
            )))
        }
        "phi" => {
            let result = required_result()?;
            let values = fields(body.stream(), &["ty", "incoming"])?;
            let ty = wrapped_type(values[0].clone())?;
            let incoming = sequence(values[1].clone())?;
            Ok(code(format!(
                "{SYNTAX}::Instruction::phi({result}, {ty}, {incoming})"
            )))
        }
        _ => Err(format!("unsupported LLVM instruction `{name}`")),
    }
}

fn switch_cases(input: TokenStream) -> Result<TokenStream, String> {
    let mut statements = String::from(
        "{ #[allow(unused_mut)] let mut __mal_cases: Vec<(String, String)> = Vec::new();",
    );
    for item in comma_items(input)? {
        if let Some(expression) = splice(&item) {
            statements.push_str(&format!("__mal_cases.extend({expression});"));
            continue;
        }
        let tokens = item.into_iter().collect::<Vec<_>>();
        let arrow = tokens
            .windows(2)
            .position(|tokens| {
                matches!(&tokens[0], TokenTree::Punct(token) if token.as_char() == '=')
                    && matches!(&tokens[1], TokenTree::Punct(token) if token.as_char() == '>')
            })
            .ok_or("expected `=>` in LLVM switch case")?;
        let value = scalar(one_token(
            tokens[..arrow].iter().cloned().collect(),
            "switch value",
        )?);
        let target = scalar(one_token(
            tokens[arrow + 2..].iter().cloned().collect(),
            "switch target",
        )?);
        statements.push_str(&format!(
            "__mal_cases.push((({value}).to_string(), ({target}).to_string()));"
        ));
    }
    statements.push_str("__mal_cases }");
    Ok(code(statements))
}

pub(super) fn terminator_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(code(format!("Some({expression})")));
    }
    let mut input = Cursor::new(input);
    let name = input.next().ok_or("expected LLVM terminator")?.to_string();
    match name.as_str() {
        "return" => {
            if input.eat_punct(';') && input.is_empty() {
                return Ok(code(format!("Some({SYNTAX}::Terminator::return_void())")));
            }
            let value = group(
                input.next().ok_or("expected return value")?,
                Delimiter::Parenthesis,
                "return value",
            )?;
            if !input.eat_punct(';') || !input.is_empty() {
                return Err("expected `;` after LLVM return".into());
            }
            let mut parts = comma_items(value.stream())?.into_iter();
            let ty = wrapped_type(parts.next().ok_or("expected return type")?)?;
            let value = scalar(one_token(
                parts.next().ok_or("expected return operand")?,
                "return operand",
            )?);
            if parts.next().is_some() {
                return Err("unexpected return value field".into());
            }
            Ok(code(format!(
                "{SYNTAX}::Terminator::return_value({ty}, {value})"
            )))
        }
        "unreachable" => {
            if !input.eat_punct(';') || !input.is_empty() {
                return Err("expected `unreachable;`".into());
            }
            Ok(code(format!("Some({SYNTAX}::Terminator::unreachable())")))
        }
        "branch" => {
            let body = group(
                input.next().ok_or("expected branch fields")?,
                Delimiter::Brace,
                "branch fields",
            )?;
            if !input.eat_punct(';') || !input.is_empty() {
                return Err("expected `;` after branch".into());
            }
            let values = named_fields(body.stream())?;
            if values.len() == 1 && values[0].0 == "target" {
                let target = scalar(one_token(values[0].1.clone(), "branch target")?);
                Ok(code(format!("{SYNTAX}::Terminator::branch({target})")))
            } else {
                let values = fields(body.stream(), &["condition", "then", "otherwise"])?;
                let condition = scalar(one_token(values[0].clone(), "branch condition")?);
                let then_target = scalar(one_token(values[1].clone(), "then target")?);
                let else_target = scalar(one_token(values[2].clone(), "otherwise target")?);
                Ok(code(format!(
                    "{SYNTAX}::Terminator::conditional_branch({condition}, {then_target}, {else_target})"
                )))
            }
        }
        "switch" => {
            let value = group(
                input.next().ok_or("expected switch value")?,
                Delimiter::Parenthesis,
                "switch value",
            )?;
            let mut parts = comma_items(value.stream())?.into_iter();
            let ty = wrapped_type(parts.next().ok_or("expected switch type")?)?;
            let value = scalar(one_token(
                parts.next().ok_or("expected switch operand")?,
                "switch operand",
            )?);
            if parts.next().is_some() {
                return Err("unexpected switch value field".into());
            }
            let body = group(
                input.next().ok_or("expected switch fields")?,
                Delimiter::Brace,
                "switch fields",
            )?;
            if !input.eat_punct(';') || !input.is_empty() {
                return Err("expected `;` after switch".into());
            }
            let values = fields(body.stream(), &["cases", "default"])?;
            let cases = group(
                one_token(values[0].clone(), "switch cases")?,
                Delimiter::Bracket,
                "switch cases",
            )?;
            let cases = switch_cases(cases.stream())?;
            let default = scalar(one_token(values[1].clone(), "default target")?);
            Ok(code(format!(
                "{SYNTAX}::Terminator::switch({ty}, {value}, {default}, {cases})"
            )))
        }
        _ => Err(format!("unsupported LLVM terminator `{name}`")),
    }
}
