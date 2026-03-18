use std::process::ExitCode;

use flt::parser::parse_statement;
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

fn run_repl() -> Result<(), ReadlineError> {
    let mut rl = DefaultEditor::new()?;
    let mut runtime = SimpleRuntime::default();
    loop {
        let line = rl.readline("> ")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match parse_statement(line) {
            Ok((remainder, statement)) => {
                let remainder = remainder.trim();
                if remainder.is_empty() {
                    match runtime.eval(&statement) {
                        Ok(val) => println!("{}", val),
                        Err(e) => eprintln!("eval error: {:?}", e),
                    }
                } else {
                    eprintln!(
                        "parse error: unexpected input after statement: {:?}",
                        remainder
                    );
                }
            }
            Err(e) => {
                eprintln!("parse error: {:?}", e);
            }
        }
        println!();
    }
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
