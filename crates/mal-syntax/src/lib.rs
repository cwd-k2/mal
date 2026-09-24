#![forbid(unsafe_code)]

pub mod ast;
pub mod diagnostic;
pub mod graph;
pub mod lexer;
pub mod parser;
pub mod requirement;
pub mod source;

pub const LANGUAGE_NAME: &str = "mal";
pub const LANGUAGE_VERSION: &str = "0.6";
