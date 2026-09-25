#![forbid(unsafe_code)]

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
