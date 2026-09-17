#![forbid(unsafe_code)]

pub mod anf;
pub mod ast;
mod backend;
pub mod check;
pub mod cli;
pub mod closure;
pub mod control;
pub mod core;
pub mod diagnostic;
pub mod driver;
pub mod editor;
mod execution;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod pipeline;
pub mod resolve;
pub mod source;

pub const LANGUAGE_NAME: &str = "mal";
pub const LANGUAGE_VERSION: &str = "0.6";

pub fn version_line() -> String {
    format!(
        "malc {} (language v{})",
        env!("CARGO_PKG_VERSION"),
        LANGUAGE_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_targets_v06() {
        assert_eq!(LANGUAGE_VERSION, "0.6");
        assert!(version_line().contains("language v0.6"));
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            include_str!("../../VERSION").trim()
        );
    }
}
