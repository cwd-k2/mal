#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let outcome = mal_fmt::cli::execute(std::env::args_os().skip(1));
    let _ = io::stdout().write_all(outcome.stdout.as_bytes());
    let _ = io::stderr().write_all(outcome.stderr.as_bytes());
    ExitCode::from(outcome.status.code())
}
