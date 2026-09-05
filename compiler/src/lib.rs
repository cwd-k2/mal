#![forbid(unsafe_code)]

pub mod anf;
pub mod ast;
pub mod c_emit;
pub mod check;
pub mod cli;
pub mod closure;
pub mod core;
pub mod diagnostic;
pub mod driver;
pub mod editor;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod pipeline;
pub mod resolve;
pub mod source;

pub const LANGUAGE_NAME: &str = "mal";
pub const LANGUAGE_VERSION: &str = "0.5";

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
    fn version_targets_v05() {
        assert_eq!(LANGUAGE_VERSION, "0.5");
        assert!(version_line().contains("language v0.5"));
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            include_str!("../../VERSION").trim()
        );
    }
}
