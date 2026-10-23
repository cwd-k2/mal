//! Parsers for the restricted LLVM construction language.

use proc_macro::{TokenStream, TokenTree};

use crate::shared::{ParseResult, parse};

mod function;
mod instruction;
mod module;
mod support;
mod ty;
mod value;

use self::instruction::{instruction_tokens, terminator_tokens};
use self::support::code;

pub(super) fn llvm_type(input: TokenStream) -> TokenStream {
    parse(input, ty::llvm_type_tokens)
}

pub(super) fn llvm_parameter(input: TokenStream) -> TokenStream {
    parse(input, ty::parameter_tokens)
}

pub(super) fn llvm_parameters(input: TokenStream) -> TokenStream {
    parse(input, ty::parameters)
}

pub(super) fn llvm_function_attributes(input: TokenStream) -> TokenStream {
    parse(input, ty::attributes)
}

pub(super) fn llvm_signature(input: TokenStream) -> TokenStream {
    parse(input, function::signature_tokens)
}

pub(super) fn llvm_declaration(input: TokenStream) -> TokenStream {
    parse(input, function::declaration_tokens)
}

pub(super) fn llvm_constant(input: TokenStream) -> TokenStream {
    parse(input, value::constant_tokens)
}

pub(super) fn llvm_typed_constant(input: TokenStream) -> TokenStream {
    parse(input, value::typed_constant_tokens)
}

pub(super) fn llvm_instruction(input: TokenStream) -> TokenStream {
    parse(input, instruction_tokens)
}

pub(super) fn llvm_terminator(input: TokenStream) -> TokenStream {
    parse(input, terminator_tokens)
}

pub(super) fn llvm_global(input: TokenStream) -> TokenStream {
    parse(input, module::global_tokens)
}

pub(super) fn llvm_metadata_operand(input: TokenStream) -> TokenStream {
    parse(input, module::metadata_operand_tokens)
}

pub(super) fn llvm_metadata(input: TokenStream) -> TokenStream {
    parse(input, module::metadata_tokens)
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
