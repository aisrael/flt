use std::path::PathBuf;

use crate::ast::Expr;
use crate::ast::Identifier;
use crate::ast::Statement;
use crate::parser::parse_statement;
use crate::runtime::Runtime;
use crate::runtime::SimpleRuntime;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

mod slash_commands;

pub use slash_commands::SlashCommand;
pub use slash_commands::SlashCommands;

/// A Repl handler is responsible for handling the REPL loop and dispatching commands.
pub trait ReplHandler {
    fn eval(&mut self, line: &str) -> eyre::Result<()>;
    /// Handles a slash command. Returns `Ok(false)` if the REPL should exit.
    fn handle_command(&mut self, rest: &str) -> eyre::Result<bool>;
    /// The prompt to display before reading the next line.
    fn prompt(&self) -> &str {
        "> "
    }
}

/// The generic Repl
pub struct Repl<H>
where
    H: ReplHandler,
{
    handler: H,
    slash_commands: SlashCommands,
    editor: DefaultEditor,
    history_path: Option<PathBuf>,
}

impl<H: ReplHandler> Repl<H> {
    /// Creates a REPL for `handler`, loading and saving history at `history_path`
    /// (`None` disables history).
    pub fn new(handler: H, history_path: Option<PathBuf>) -> eyre::Result<Self> {
        let config = rustyline::Config::builder()
            .max_history_size(HISTORY_DEPTH)
            .expect("valid history size")
            .auto_add_history(true)
            .build();
        let editor = DefaultEditor::with_config(config)?;
        let slash_commands = SlashCommands::new();
        let mut repl = Self {
            editor,
            slash_commands,
            handler,
            history_path,
        };
        repl.load_history()?;
        Ok(repl)
    }

    pub fn add_slash_command(&mut self, command: SlashCommand) {
        self.slash_commands.add_command(command);
    }

    fn load_history(&mut self) -> eyre::Result<()> {
        let Some(history_path) = self.history_path.as_deref() else {
            return Ok(());
        };
        if history_path.exists() {
            println!("Loading REPL history from: {:?}", history_path);
            self.editor.load_history(history_path)?;
        }
        Ok(())
    }

    fn save_history(&mut self) -> eyre::Result<()> {
        let Some(history_path) = self.history_path.as_deref() else {
            return Ok(());
        };
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        self.editor.save_history(history_path)?;
        Ok(())
    }

    pub fn run(&mut self) -> eyre::Result<()> {
        let repl_result = self.repl_loop();
        let _ = self.save_history();
        repl_result
    }

    fn repl_loop(&mut self) -> eyre::Result<()> {
        loop {
            let prompt = self.handler.prompt().to_string();
            let line = match self.editor.readline(&prompt) {
                Ok(line) => line,
                Err(ReadlineError::Eof) => break Ok(()),
                Err(ReadlineError::Interrupted) => continue,
                Err(e) => return Err(e.into()),
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix('/') {
                if !self.handler.handle_command(rest)? {
                    break Ok(());
                }
            } else {
                self.handler.eval(line)?;
            }
            println!();
        }
    }
}

/// Maximum number of inputs to keep in REPL history.
const HISTORY_DEPTH: usize = 1000;

/// The default history location for the `flt` REPL.
pub fn default_history_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("flt").join("history"))
}

/// The concrete implementation of the REPL context for `flt`
#[derive(Default)]
pub struct FltRepl {
    runtime: SimpleRuntime,
}

impl FltRepl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a full statement from `line`, treating any leftover, non-whitespace
    /// input after the statement as a parse error.
    fn parse_full(line: &str) -> Result<Statement, String> {
        match parse_statement(line) {
            Ok((remainder, statement)) => {
                let remainder = remainder.trim();
                if remainder.is_empty() {
                    Ok(statement)
                } else {
                    Err(format!(
                        "parse error: unexpected input after statement: {:?}",
                        remainder
                    ))
                }
            }
            Err(e) => Err(format!("parse error: {:?}", e)),
        }
    }

    /// Dispatches a slash command. Returns `false` if the REPL should exit.
    fn handle_command(&mut self, rest: &str) -> bool {
        let (cmd, args) = match rest.split_once(char::is_whitespace) {
            Some((cmd, args)) => (cmd, args.trim()),
            None => (rest, ""),
        };
        match cmd {
            "quit" | "q" => return false,
            "help" | "h" => Self::handle_help(),
            "parse" => Self::handle_parse(args),
            "unset" => self.handle_unset(args),
            "inspect" | "i" => self.handle_inspect(args),
            _ => eprintln!("unknown command: /{cmd}"),
        }
        true
    }

    /// Prints a list of available REPL commands.
    fn handle_help() {
        println!("Available commands:");
        println!("  /parse <expression>          Parse an expression and print its AST without evaluating it");
        println!("  /unset <identifier>          Remove a bound variable");
        println!("  /inspect <expression> (/i)   Parse and evaluate an expression, reporting on bound variables and literal types");
        println!("  /help (/h)                   Show this help message");
        println!("  /quit (/q)                   Exit the REPL");
    }

    /// Parses `args` and prints the resulting AST without evaluating it.
    fn handle_parse(args: &str) {
        if args.is_empty() {
            eprintln!("usage: /parse <expression>");
            return;
        }
        match Self::parse_full(args) {
            Ok(statement) => println!("{:?}", statement),
            Err(msg) => eprintln!("{}", msg),
        }
    }

    /// Removes a previously bound variable, so later references raise an unbound identifier error.
    fn handle_unset(&mut self, args: &str) {
        if args.is_empty() {
            eprintln!("usage: /unset <identifier>");
            return;
        }
        match Identifier::try_from(args.trim()) {
            Ok(ident) => {
                if self.runtime.global_scope.remove_variable(&ident).is_none() {
                    eprintln!("unbound identifier: {}", ident);
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }

    /// Parses and evaluates `args`, reporting specially on bare variable
    /// references and literal expressions.
    fn handle_inspect(&mut self, args: &str) {
        if args.is_empty() {
            eprintln!("usage: /inspect <expression>");
            return;
        }
        let statement = match Self::parse_full(args) {
            Ok(statement) => statement,
            Err(msg) => {
                eprintln!("{}", msg);
                return;
            }
        };

        if let Statement::Expr(Expr::Ident(name)) = &statement {
            match self.runtime.global_scope.get_variable(name) {
                Some(value) => println!("(variable) {}", value),
                None => println!("Unbound variable"),
            }
            return;
        }

        let is_literal = matches!(
            &statement,
            Statement::Expr(Expr::Literal(_) | Expr::ArrayLiteral(_) | Expr::MapLiteral(_))
        );

        match self.runtime.eval(&statement) {
            Ok(value) if is_literal => match value.type_of() {
                Ok(ty) => println!("{}", ty),
                Err(e) => eprintln!("eval error: {:?}", e),
            },
            Ok(value) => println!("{}", value),
            Err(e) => eprintln!("eval error: {:?}", e),
        }
    }
}

impl ReplHandler for FltRepl {
    fn eval(&mut self, line: &str) -> eyre::Result<()> {
        match Self::parse_full(line) {
            Ok(statement) => match self.runtime.eval(&statement) {
                Ok(val) => println!("{}", val),
                Err(e) => eprintln!("eval error: {:?}", e),
            },
            Err(msg) => eprintln!("{}", msg),
        }
        Ok(())
    }

    fn handle_command(&mut self, rest: &str) -> eyre::Result<bool> {
        Ok(FltRepl::handle_command(self, rest))
    }
}

#[cfg(test)]
mod tests {
    use rustyline::history::History;

    use super::*;

    struct TestHandler;

    impl ReplHandler for TestHandler {
        fn eval(&mut self, _line: &str) -> eyre::Result<()> {
            Ok(())
        }

        fn handle_command(&mut self, _rest: &str) -> eyre::Result<bool> {
            Ok(true)
        }
    }

    struct PromptHandler;

    impl ReplHandler for PromptHandler {
        fn eval(&mut self, _line: &str) -> eyre::Result<()> {
            Ok(())
        }

        fn handle_command(&mut self, _rest: &str) -> eyre::Result<bool> {
            Ok(true)
        }

        fn prompt(&self) -> &str {
            "... "
        }
    }

    #[test]
    fn test_default_prompt() {
        assert_eq!("> ", TestHandler.prompt());
    }

    #[test]
    fn test_custom_prompt() {
        assert_eq!("... ", PromptHandler.prompt());
    }

    #[test]
    fn test_no_history_path_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut repl = Repl::new(TestHandler, None).unwrap();
        repl.editor.add_history_entry("1 + 1").unwrap();
        repl.save_history().unwrap();
        assert_eq!(0, std::fs::read_dir(dir.path()).unwrap().count());
    }

    #[test]
    fn test_missing_history_file_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("history");
        let repl = Repl::new(TestHandler, Some(history_path.clone())).unwrap();
        assert_eq!(0, repl.editor.history().len());
        assert!(!history_path.exists());
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("nested").join("history");
        let mut repl = Repl::new(TestHandler, Some(history_path.clone())).unwrap();
        repl.editor.add_history_entry("1 + 1").unwrap();
        repl.save_history().unwrap();
        assert!(history_path.exists());
    }

    #[test]
    fn test_history_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("history");
        let mut repl = Repl::new(TestHandler, Some(history_path.clone())).unwrap();
        repl.editor.add_history_entry("1 + 1").unwrap();
        repl.editor.add_history_entry("x = 2").unwrap();
        repl.save_history().unwrap();

        let reloaded = Repl::new(TestHandler, Some(history_path)).unwrap();
        let history = reloaded.editor.history();
        assert_eq!(2, history.len());
    }
}
