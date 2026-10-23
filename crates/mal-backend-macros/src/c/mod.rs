//! Parser for the Rust-shaped C construction language.

use proc_macro::TokenStream;

use crate::shared::parse;

mod expression;
mod function;
mod item;
mod record;
mod statement;
mod support;
mod ty;

pub(super) fn c_expr(input: TokenStream) -> TokenStream {
    parse(input, expression::c_expression)
}

pub(super) fn c_type(input: TokenStream) -> TokenStream {
    parse(input, ty::c_type_syntax)
}

pub(super) fn c_statement(input: TokenStream) -> TokenStream {
    parse(input, statement::c_statement_syntax)
}

pub(super) fn c_block(input: TokenStream) -> TokenStream {
    parse(input, statement::c_block_syntax)
}

pub(super) fn c_items(input: TokenStream) -> TokenStream {
    parse(input, item::c_items_syntax)
}

pub(super) fn c_record_fields(input: TokenStream) -> TokenStream {
    parse(input, record::c_record_fields_syntax)
}

pub(super) fn c_record(input: TokenStream) -> TokenStream {
    parse(input, record::c_record_syntax)
}

pub(super) fn c_initializers(input: TokenStream) -> TokenStream {
    parse(input, expression::c_initializer_fields)
}

pub(super) fn c_invocation(input: TokenStream) -> TokenStream {
    parse(input, item::c_invocation_syntax)
}

pub(super) fn c_switch_cases(input: TokenStream) -> TokenStream {
    parse(input, statement::c_switch_arms)
}

pub(super) fn c_signature(input: TokenStream) -> TokenStream {
    parse(input, function::c_signature_syntax)
}

pub(super) fn c_parameter(input: TokenStream) -> TokenStream {
    parse(input, function::c_parameter_syntax)
}

pub(super) fn c_parameters(input: TokenStream) -> TokenStream {
    parse(input, function::c_parameters_syntax)
}

pub(super) fn c_function(input: TokenStream) -> TokenStream {
    parse(input, function::c_function_syntax)
}
