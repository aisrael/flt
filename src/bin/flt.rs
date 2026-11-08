use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use flt::repl::default_history_path;
use flt::repl::run_file;
use flt::repl::run_program;
use flt::repl::FltRepl;
use flt::repl::Repl;

// Returns the library version, which reflects the crate version
pub fn version() -> String {
    clap::crate_version!().to_string()
}

#[derive(Parser)]
#[command(
    name = "flt",
    disable_version_flag = true,
    args_conflicts_with_subcommands = true
)]
struct Cli {
    /// Evaluate PROGRAM as an flt program, print the value of its last statement, and exit
    #[arg(short = 'e', value_name = "PROGRAM")]
    eval: Option<String>,
    /// Run the flt program in the file PROGRAM (by convention, a file that ends in `.flt`) and exit.
    /// Unlike `-e`, it does not print the value of the last statement
    #[arg(value_name = "PROGRAM", conflicts_with = "eval")]
    program: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print the version
    Version,
}

/// Prints the error, if any, to standard error and gives the exit code
fn exit_code(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(Command::Version) = cli.command {
        println!("flt version {}", version());
        return ExitCode::SUCCESS;
    }
    if let Some(program) = cli.eval {
        return exit_code(run_program(&program));
    }
    if let Some(path) = cli.program {
        return exit_code(run_file(&path));
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
