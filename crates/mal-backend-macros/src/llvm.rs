//! Parsers for the restricted LLVM construction language.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::shared::{Cursor, ParseResult, parse};

const SYNTAX: &str = "crate::backend::llvm::syntax";

fn code(source: impl AsRef<str>) -> TokenStream {
    source
        .as_ref()
        .parse()
        .expect("generated Rust tokens parse")
}

fn rust_expression(token: &TokenTree) -> Option<TokenStream> {
    let TokenTree::Group(group) = token else {
        return None;
    };
    if group.delimiter() != Delimiter::Brace {
        return None;
    }
    Some(group.stream())
}

fn scalar(token: TokenTree) -> TokenStream {
    rust_expression(&token).unwrap_or_else(|| [token].into_iter().collect())
}

fn one_token(input: TokenStream, description: &str) -> Result<TokenTree, String> {
    let mut tokens = input.into_iter();
    let token = tokens
        .next()
        .ok_or_else(|| format!("expected {description}"))?;
    if tokens.next().is_some() {
        return Err(format!("expected one {description}"));
    }
    Ok(token)
}

fn group(token: TokenTree, delimiter: Delimiter, description: &str) -> Result<Group, String> {
    match token {
        TokenTree::Group(group) if group.delimiter() == delimiter => Ok(group),
        _ => Err(format!("expected {description}")),
    }
}

fn split(input: TokenStream, separator: char) -> Vec<TokenStream> {
    let mut items = Vec::new();
    let mut current = TokenStream::new();
    for token in input {
        if matches!(&token, TokenTree::Punct(punctuation) if punctuation.as_char() == separator) {
            items.push(current);
            current = TokenStream::new();
        } else {
            current.extend([token]);
        }
    }
    items.push(current);
    items
}

fn comma_items(input: TokenStream) -> Result<Vec<TokenStream>, String> {
    let mut items = split(input, ',');
    if items.last().is_some_and(TokenStream::is_empty) {
        items.pop();
    }
    if items.iter().any(TokenStream::is_empty) {
        return Err("expected an item between commas".into());
    }
    Ok(items)
}

fn splice(item: &TokenStream) -> Option<TokenStream> {
    let tokens = item.clone().into_iter().collect::<Vec<_>>();
    if tokens.len() == 3
        && tokens[..2]
            .iter()
            .all(|token| matches!(token, TokenTree::Punct(token) if token.as_char() == '.'))
    {
        rust_expression(&tokens[2])
    } else {
        None
    }
}

fn embedded(item: &TokenStream) -> Option<TokenStream> {
    let token = one_token(item.clone(), "embedded Rust expression").ok()?;
    rust_expression(&token)
}

fn vector(
    input: TokenStream,
    mut static_item: impl FnMut(TokenStream) -> Result<TokenStream, String>,
) -> Result<TokenStream, String> {
    let mut statements = String::from("{ #[allow(unused_mut)] let mut __mal_items = Vec::new();");
    for item in comma_items(input)? {
        if let Some(expression) = splice(&item) {
            statements.push_str(&format!("__mal_items.extend({expression});"));
        } else if let Some(expression) = embedded(&item) {
            statements.push_str(&format!("__mal_items.push({expression});"));
        } else {
            statements.push_str(&format!("__mal_items.push({});", static_item(item)?));
        }
    }
    statements.push_str("__mal_items }");
    Ok(code(statements))
}

fn fallible_vector(
    input: TokenStream,
    mut static_item: impl FnMut(TokenStream) -> Result<TokenStream, String>,
) -> Result<TokenStream, String> {
    let mut statements =
        String::from("(|| { #[allow(unused_mut)] let mut __mal_items = Vec::new();");
    for item in comma_items(input)? {
        if let Some(expression) = splice(&item) {
            statements.push_str(&format!("__mal_items.extend({expression});"));
        } else if let Some(expression) = embedded(&item) {
            statements.push_str(&format!("__mal_items.push({expression});"));
        } else {
            statements.push_str(&format!("__mal_items.push(({})?);", static_item(item)?));
        }
    }
    statements.push_str("Some(__mal_items) })()");
    Ok(code(statements))
}

fn llvm_type_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn wrapped_type(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    llvm_type_tokens(input)
}

fn parameter_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn parameters(input: TokenStream) -> Result<TokenStream, String> {
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

fn attributes(input: TokenStream) -> Result<TokenStream, String> {
    vector(input, function_attribute)
}

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

fn signature_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn declaration_tokens(input: TokenStream) -> Result<TokenStream, String> {
    if let Some(expression) = embedded(&input) {
        return Ok(expression);
    }
    let header = function_header(input, true)?;
    Ok(code(format!(
        "{SYNTAX}::FunctionDeclaration::new({}, {}, {}).with_attributes({})",
        header.result, header.name, header.parameters, header.attributes
    )))
}

fn named_fields(input: TokenStream) -> Result<Vec<(String, TokenStream)>, String> {
    comma_items(input)?
        .into_iter()
        .map(|item| {
            let tokens = item.into_iter().collect::<Vec<_>>();
            let colon = tokens
                .iter()
                .position(
                    |token| matches!(token, TokenTree::Punct(token) if token.as_char() == ':'),
                )
                .ok_or("expected `:` after LLVM field name")?;
            if colon != 1 {
                return Err("expected one LLVM field name".into());
            }
            Ok((
                tokens[0].to_string(),
                tokens[colon + 1..].iter().cloned().collect(),
            ))
        })
        .collect()
}

fn fields(input: TokenStream, expected: &[&str]) -> Result<Vec<TokenStream>, String> {
    let fields = named_fields(input)?;
    if fields.len() != expected.len()
        || fields
            .iter()
            .zip(expected)
            .any(|((actual, _), expected)| actual != expected)
    {
        return Err(format!(
            "expected LLVM fields `{}` in this order",
            expected.join(", ")
        ));
    }
    Ok(fields.into_iter().map(|(_, value)| value).collect())
}

fn typed_value_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn typed_values(input: TokenStream) -> Result<TokenStream, String> {
    fallible_vector(input, typed_value_tokens)
}

fn typed_constant_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn typed_constants(input: TokenStream) -> Result<TokenStream, String> {
    fallible_vector(input, typed_constant_tokens)
}

fn named_form(input: TokenStream) -> Result<(String, Group), String> {
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

fn constant_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn sequence(input: TokenStream) -> Result<TokenStream, String> {
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

fn indices(input: TokenStream) -> Result<TokenStream, String> {
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

fn callee(input: TokenStream) -> Result<TokenStream, String> {
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

fn instruction_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn terminator_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn global_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn metadata_operand_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

fn metadata_tokens(input: TokenStream) -> Result<TokenStream, String> {
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

pub(super) fn llvm_type(input: TokenStream) -> TokenStream {
    parse(input, llvm_type_tokens)
}

pub(super) fn llvm_parameter(input: TokenStream) -> TokenStream {
    parse(input, parameter_tokens)
}

pub(super) fn llvm_parameters(input: TokenStream) -> TokenStream {
    parse(input, parameters)
}

pub(super) fn llvm_function_attributes(input: TokenStream) -> TokenStream {
    parse(input, attributes)
}

pub(super) fn llvm_signature(input: TokenStream) -> TokenStream {
    parse(input, signature_tokens)
}

pub(super) fn llvm_declaration(input: TokenStream) -> TokenStream {
    parse(input, declaration_tokens)
}

pub(super) fn llvm_constant(input: TokenStream) -> TokenStream {
    parse(input, constant_tokens)
}

pub(super) fn llvm_typed_constant(input: TokenStream) -> TokenStream {
    parse(input, typed_constant_tokens)
}

pub(super) fn llvm_instruction(input: TokenStream) -> TokenStream {
    parse(input, instruction_tokens)
}

pub(super) fn llvm_terminator(input: TokenStream) -> TokenStream {
    parse(input, terminator_tokens)
}

pub(super) fn llvm_global(input: TokenStream) -> TokenStream {
    parse(input, global_tokens)
}

pub(super) fn llvm_metadata_operand(input: TokenStream) -> TokenStream {
    parse(input, metadata_operand_tokens)
}

pub(super) fn llvm_metadata(input: TokenStream) -> TokenStream {
    parse(input, metadata_tokens)
}

fn emitter_and_syntax(input: TokenStream) -> Result<(TokenStream, TokenStream), String> {
    let mut emitter = TokenStream::new();
    let mut syntax = TokenStream::new();
    let mut separator = false;
    for token in input {
        if !separator
            && matches!(&token, TokenTree::Punct(punctuation) if punctuation.as_char() == ';')
        {
            separator = true;
        } else if separator {
            syntax.extend([token]);
        } else {
            emitter.extend([token]);
        }
    }
    if !separator || emitter.is_empty() || syntax.is_empty() {
        return Err("expected `emitter; LLVM syntax`".into());
    }
    Ok((emitter, syntax))
}

pub(super) fn emit_instruction(input: TokenStream) -> TokenStream {
    parse(input, |input| -> ParseResult {
        let (emitter, syntax) = emitter_and_syntax(input)?;
        let instruction = instruction_tokens(syntax)?;
        Ok(code(format!(
            "{emitter}.structured_instruction({instruction})"
        )))
    })
}

pub(super) fn emit_terminator(input: TokenStream) -> TokenStream {
    parse(input, |input| -> ParseResult {
        let (emitter, syntax) = emitter_and_syntax(input)?;
        let terminator = terminator_tokens(syntax)?;
        Ok(code(format!("{emitter}.terminate({terminator})")))
    })
}
