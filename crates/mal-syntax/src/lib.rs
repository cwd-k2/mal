#![forbid(unsafe_code)]

//! Source admission shared by every mal tool.
//!
//! This crate owns source identity, diagnostics, lexing, parsing, requirement paths, and source-graph loading. It has
//! no knowledge of name resolution or types, so formatter and editor syntax features can use it without the compiler.

pub mod ast;
pub mod diagnostic;
pub mod graph;
pub mod lexer;
pub mod parser;
pub mod requirement;
pub mod source;

/// Canonical language name used by tools and generated metadata.
pub const LANGUAGE_NAME: &str = "mal";
/// Language specification version implemented by this workspace.
pub const LANGUAGE_VERSION: &str = "0.7";
