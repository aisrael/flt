use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use flt::repl::default_history_path;
use flt::repl::run_program;
use flt::repl::FltRepl;
use flt::repl::Repl;

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

#[derive(Parser)]
#[command(name = "flt", disable_version_flag = true)]
struct Cli {
    /// Evaluate PROGRAM as an flt program, print the value of its last statement, and exit
    #[arg(short = 'e', value_name = "PROGRAM")]
    eval: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print the version
    Version,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(Command::Version) = cli.command {
        println!("flt version {}", version());
        return ExitCode::SUCCESS;
    }
    if let Some(program) = cli.eval {
        return match run_program(&program) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("Error: {message}");
                ExitCode::FAILURE
            }
        };
    }

    let history_path = if std::env::var_os("FLT_NO_HISTORY").is_some() {
        None
    } else {
        default_history_path()
    };

    match Repl::new(FltRepl::new(), history_path) {
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
