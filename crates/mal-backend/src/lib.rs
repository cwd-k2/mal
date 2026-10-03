#![forbid(unsafe_code)]
#![recursion_limit = "512"]

//! Pure lowering and artifact generation for checked mal programs.
//!
//! The pipeline ends in LLVM text, generated C, and selected C11 runtime sources. File placement and toolchain process
//! execution remain the responsibility of `mal-compiler`.

mod anf;
mod backend;
mod call_pattern;
mod closure;
#[cfg(any(debug_assertions, test))]
mod continuation_specialization;
mod control;
mod core;
mod execution;
mod flow;
pub mod pipeline;

#[cfg(test)]
mod tests;
