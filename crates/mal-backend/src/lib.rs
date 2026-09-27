#![forbid(unsafe_code)]
#![recursion_limit = "512"]

mod anf;
mod backend;
mod call_pattern;
mod closure;
mod control;
mod core;
mod execution;
mod flow;
pub mod pipeline;

#[cfg(test)]
mod tests;
