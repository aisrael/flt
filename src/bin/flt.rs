use std::process::ExitCode;

use flt::repl::default_history_path;
use flt::repl::FltRepl;
use flt::repl::Repl;

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

    match Repl::new(FltRepl::new(), default_history_path()) {
        Ok(mut repl) => match repl.run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("Error initializing REPL: {:?}", e);
            ExitCode::FAILURE
        }
    }
}
