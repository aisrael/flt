use std::process::ExitCode;

use flt::repl::run_repl;
use rustyline::error::ReadlineError;

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("version") {
        println!("flt version {}", version());
        return ExitCode::SUCCESS;
    }

    match run_repl() {
        Ok(()) => ExitCode::SUCCESS,
        Err(ReadlineError::Interrupted) => {
            println!("\nExiting.");
            ExitCode::SUCCESS
        }
        Err(ReadlineError::Eof) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            ExitCode::FAILURE
        }
    }
}
