#![forbid(unsafe_code)]
//! Procedural syntax frontends for generated C and LLVM artifacts.
//!
//! This crate admits restricted Rust-shaped syntax and expands it into typed
//! nodes owned by `mal-backend`. Validation, builders, and rendering remain in
//! that crate.

use proc_macro::TokenStream;

mod c;
mod llvm;
mod shared;

/// Builds a C expression from Rust-shaped syntax.
#[proc_macro]
pub fn c_expr(input: TokenStream) -> TokenStream {
    c::c_expr(input)
}

/// Builds a C type from Rust-shaped syntax.
#[proc_macro]
pub fn c_type(input: TokenStream) -> TokenStream {
    c::c_type(input)
}

/// Builds a C statement from Rust-shaped syntax.
#[proc_macro]
pub fn c_statement(input: TokenStream) -> TokenStream {
    c::c_statement(input)
}

/// Builds a C block from Rust-shaped syntax.
#[proc_macro]
pub fn c_block(input: TokenStream) -> TokenStream {
    c::c_block(input)
}

/// Builds a C translation unit from Rust-shaped items.
#[proc_macro]
pub fn c_items(input: TokenStream) -> TokenStream {
    c::c_items(input)
}

/// Builds C struct or union fields from Rust-shaped declarations.
#[proc_macro]
pub fn c_record_fields(input: TokenStream) -> TokenStream {
    c::c_record_fields(input)
}

/// Builds a C struct or union definition from Rust-shaped syntax.
#[proc_macro]
pub fn c_record(input: TokenStream) -> TokenStream {
    c::c_record(input)
}

/// Builds C initializer fields from Rust-shaped syntax.
#[proc_macro]
pub fn c_initializers(input: TokenStream) -> TokenStream {
    c::c_initializers(input)
}

/// Builds a C preprocessor macro invocation.
#[proc_macro]
pub fn c_invocation(input: TokenStream) -> TokenStream {
    c::c_invocation(input)
}

/// Builds C switch cases from Rust-shaped match arms.
#[proc_macro]
pub fn c_switch_cases(input: TokenStream) -> TokenStream {
    c::c_switch_cases(input)
}

/// Builds a C function signature from Rust-shaped syntax.
#[proc_macro]
pub fn c_signature(input: TokenStream) -> TokenStream {
    c::c_signature(input)
}

/// Builds a C function parameter from Rust-shaped syntax.
#[proc_macro]
pub fn c_parameter(input: TokenStream) -> TokenStream {
    c::c_parameter(input)
}

/// Builds C function parameters from Rust-shaped syntax.
#[proc_macro]
pub fn c_parameters(input: TokenStream) -> TokenStream {
    c::c_parameters(input)
}

/// Builds a C function definition from Rust-shaped syntax.
#[proc_macro]
pub fn c_function(input: TokenStream) -> TokenStream {
    c::c_function(input)
}

/// Builds an LLVM type.
#[proc_macro]
pub fn llvm_type(input: TokenStream) -> TokenStream {
    llvm::llvm_type(input)
}

/// Builds an LLVM parameter.
#[proc_macro]
pub fn llvm_parameter(input: TokenStream) -> TokenStream {
    llvm::llvm_parameter(input)
}

/// Builds LLVM parameters.
#[proc_macro]
pub fn llvm_parameters(input: TokenStream) -> TokenStream {
    llvm::llvm_parameters(input)
}

/// Builds LLVM function attributes.
#[proc_macro]
pub fn llvm_function_attributes(input: TokenStream) -> TokenStream {
    llvm::llvm_function_attributes(input)
}

/// Builds an LLVM function signature.
#[proc_macro]
pub fn llvm_signature(input: TokenStream) -> TokenStream {
    llvm::llvm_signature(input)
}

/// Builds an LLVM function declaration.
#[proc_macro]
pub fn llvm_declaration(input: TokenStream) -> TokenStream {
    llvm::llvm_declaration(input)
}

/// Builds an LLVM constant.
#[proc_macro]
pub fn llvm_constant(input: TokenStream) -> TokenStream {
    llvm::llvm_constant(input)
}

/// Builds a typed LLVM constant.
#[proc_macro]
pub fn llvm_typed_constant(input: TokenStream) -> TokenStream {
    llvm::llvm_typed_constant(input)
}

/// Builds an LLVM instruction.
#[proc_macro]
pub fn llvm_instruction(input: TokenStream) -> TokenStream {
    llvm::llvm_instruction(input)
}

/// Builds an LLVM terminator.
#[proc_macro]
pub fn llvm_terminator(input: TokenStream) -> TokenStream {
    llvm::llvm_terminator(input)
}

/// Builds an LLVM global definition.
#[proc_macro]
pub fn llvm_global(input: TokenStream) -> TokenStream {
    llvm::llvm_global(input)
}

/// Builds an LLVM metadata operand.
#[proc_macro]
pub fn llvm_metadata_operand(input: TokenStream) -> TokenStream {
    llvm::llvm_metadata_operand(input)
}

/// Builds an LLVM metadata definition.
#[proc_macro]
pub fn llvm_metadata(input: TokenStream) -> TokenStream {
    llvm::llvm_metadata(input)
}

/// Builds and registers an LLVM instruction.
#[proc_macro]
pub fn emit_instruction(input: TokenStream) -> TokenStream {
    llvm::emit_instruction(input)
}

/// Builds and registers an LLVM terminator.
#[proc_macro]
pub fn emit_terminator(input: TokenStream) -> TokenStream {
    llvm::emit_terminator(input)
}
