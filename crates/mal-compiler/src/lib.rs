#![forbid(unsafe_code)]

pub mod anf;
mod backend;
pub mod cli;
pub mod closure;
pub mod control;
pub mod core;
pub mod driver;
mod execution;
pub mod pipeline;

pub fn version_line() -> String {
    format!(
        "malc {} (language v{})",
        env!("CARGO_PKG_VERSION"),
        mal_syntax::LANGUAGE_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_targets_v06() {
        assert_eq!(mal_syntax::LANGUAGE_VERSION, "0.6");
        assert!(version_line().contains("language v0.6"));
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            include_str!("../../../VERSION").trim()
        );
    }
}
