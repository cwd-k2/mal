#![forbid(unsafe_code)]

//! The `malc` command boundary.
//!
//! [`cli`] maps arguments to use cases. [`driver`] owns filesystem output and pinned Clang/LLD processes while the
//! syntax, frontend, and backend crates retain language and artifact-generation policy.

pub mod cli;
pub mod driver;

/// Returns the compiler and language version line shared by `--version` and tests.
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
