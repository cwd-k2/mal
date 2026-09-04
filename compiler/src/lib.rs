#![forbid(unsafe_code)]

pub mod ast;
pub mod check;
pub mod cli;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod source;

pub const LANGUAGE_NAME: &str = "mal";
pub const LANGUAGE_VERSION: &str = "0.4";

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
    fn version_targets_v04() {
        assert_eq!(LANGUAGE_VERSION, "0.4");
        assert!(version_line().contains("language v0.4"));
    }
}
