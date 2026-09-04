#![forbid(unsafe_code)]

use std::process::ExitCode;

const HELP: &str = "malc — reference compiler for mal v0.4

Usage:
  malc --help
  malc --version

Compilation commands will be added milestone-by-milestone; see docs/implementation/m0.md.
";

fn main() -> ExitCode {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();

    match (arguments.next(), arguments.next()) {
        (None, None) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        (Some(argument), None) if argument == "--help" || argument == "-h" => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        (Some(argument), None) if argument == "--version" || argument == "-V" => {
            println!("{}", malc::version_line());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("malc: compilation commands are not implemented yet");
            eprintln!("Try 'malc --help' for the current interface.");
            ExitCode::from(2)
        }
    }
}
