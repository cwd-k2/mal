#![forbid(unsafe_code)]

mod protocol;
mod server;

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("mal-lsp: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<ExitCode> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut server = server::Server::new();

    while let Some(message) = protocol::read_message(&mut reader)? {
        let outcome = server.handle(message);
        for response in outcome.messages {
            protocol::write_message(&mut writer, &response)?;
        }
        if let Some(success) = outcome.exit {
            return Ok(if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            });
        }
    }
    Ok(ExitCode::SUCCESS)
}
