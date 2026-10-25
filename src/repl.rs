use std::ffi::OsString;
use std::path::PathBuf;

use crate::ast::Expr;
use crate::ast::Identifier;
use crate::ast::Statement;
use crate::errors::RuntimeError;
use crate::parser::parse_statement;
use crate::runtime::Runtime;
use crate::runtime::SimpleRuntime;
use crate::Error;
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

/// Environment variable that overrides the default REPL history location.
pub const HISTORY_PATH_ENV_VAR: &str = "FLT_HISTORY_PATH";

/// The default history location for the `flt` REPL: `$FLT_HISTORY_PATH` if set
/// and non-empty, otherwise `flt/history` under the platform's local data directory.
pub fn default_history_path() -> Option<PathBuf> {
    history_path_from(std::env::var_os(HISTORY_PATH_ENV_VAR))
}

fn history_path_from(env_value: Option<OsString>) -> Option<PathBuf> {
    match env_value {
        Some(path) if !path.is_empty() => Some(PathBuf::from(path)),
        _ => dirs::data_local_dir().map(|dir| dir.join("flt").join("history")),
    }
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
        if let Err(msg) = self.unset(args) {
            eprintln!("{msg}");
        }
    }

    /// Removes the variable named by `args`, or explains why it can't: there's no such
    /// variable, or the name is a built-in type, a function or a native type, which can't be
    /// unset.
    fn unset(&mut self, args: &str) -> Result<(), String> {
        if args.is_empty() {
            return Err("usage: /unset <identifier>".to_string());
        }
        let ident = Identifier::try_from(args.trim()).map_err(|e| format!("{e:?}"))?;
        let global_scope = &mut self.runtime.global_scope;
        if global_scope.remove_variable(&ident).is_some() {
            return Ok(());
        }
        // A name that can't be bound is a built-in type, a function or a native type, not a
        // variable
        match global_scope.check_bindable(&ident) {
            Err(Error::RuntimeError(RuntimeError::CannotBind { name, reason })) => {
                Err(format!("Cannot unset {name}: it's {reason}"))
            }
            _ => Err(format!("unbound identifier: {ident}")),
        }
    }

    /// Reports on a bare identifier: a variable's value, a function's type, or a native type.
    /// A name is never more than one of these, since a variable can't use a function's or a
    /// type's name.
    fn inspect_identifier(&self, name: &str) -> String {
        let global_scope = &self.runtime.global_scope;
        if let Some(value) = global_scope.get_variable(name) {
            return format!("(variable) {value}");
        }
        if let Some(function) = global_scope.get_function(name) {
            return format!("(function) {}", function.r#type());
        }
        match self.runtime.get_native_type(name) {
            Some(native_type) => format!("(native type) {}", native_type.name()),
            None => "Unbound variable".to_string(),
        }
    }

    /// Parses and evaluates `args`, reporting specially on bare variable and function
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
            println!("{}", self.inspect_identifier(name));
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
    // rustyline's `FileHistory::save` temporarily changes the process-wide
    // umask, which can race with directory creation in parallel tests and
    // produce unwritable directories. Tests that touch the filesystem are
    // marked `#[serial(fs)]` so they don't run concurrently.
    use serial_test::serial;

    use super::*;
    use crate::runtime::native::fixtures::Meters;

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
    fn test_history_path_from_env_override() {
        assert_eq!(
            Some(PathBuf::from("/tmp/flt-history")),
            history_path_from(Some(OsString::from("/tmp/flt-history")))
        );
    }

    #[test]
    fn test_history_path_from_falls_back_to_default() {
        let default = dirs::data_local_dir().map(|dir| dir.join("flt").join("history"));
        assert_eq!(default, history_path_from(None));
        assert_eq!(default, history_path_from(Some(OsString::new())));
    }

    #[test]
    #[serial(fs)]
    fn test_no_history_path_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut repl = Repl::new(TestHandler, None).unwrap();
        repl.editor.add_history_entry("1 + 1").unwrap();
        repl.save_history().unwrap();
        assert_eq!(0, std::fs::read_dir(dir.path()).unwrap().count());
    }

    #[test]
    #[serial(fs)]
    fn test_missing_history_file_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("history");
        let repl = Repl::new(TestHandler, Some(history_path.clone())).unwrap();
        assert_eq!(0, repl.editor.history().len());
        assert!(!history_path.exists());
    }

    #[test]
    #[serial(fs)]
    fn test_save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("nested").join("history");
        let mut repl = Repl::new(TestHandler, Some(history_path.clone())).unwrap();
        repl.editor.add_history_entry("1 + 1").unwrap();
        repl.save_history().unwrap();
        assert!(history_path.exists());
    }

    #[test]
    #[serial(fs)]
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

    #[test]
    fn test_inspect_builtin_function() {
        let repl = FltRepl::new();
        assert_eq!(
            repl.inspect_identifier("typeof"),
            "(function) fn(Any) -> Type"
        );
    }

    #[test]
    fn test_inspect_function_after_refused_binding() {
        let mut repl = FltRepl::new();
        let statement = FltRepl::parse_full("let typeof = 1").unwrap();
        assert!(repl.runtime.eval(&statement).is_err());
        assert_eq!(
            repl.inspect_identifier("typeof"),
            "(function) fn(Any) -> Type"
        );
    }

    #[test]
    fn test_unset_variable() {
        let mut repl = FltRepl::new();
        let statement = FltRepl::parse_full("let x = 1").unwrap();
        repl.runtime.eval(&statement).unwrap();
        assert_eq!(repl.unset("x"), Ok(()));
        assert!(!repl.runtime.global_scope.has_variable("x"));
        let statement = FltRepl::parse_full("x").unwrap();
        assert_eq!(
            repl.runtime.eval(&statement).unwrap_err().to_string(),
            "Runtime Error: Unbound identifier: x"
        );
    }

    #[test]
    fn test_unset_unbound_identifier() {
        let mut repl = FltRepl::new();
        assert_eq!(
            repl.unset("nope"),
            Err("unbound identifier: nope".to_string())
        );
    }

    #[test]
    fn test_unset_builtin_type() {
        let mut repl = FltRepl::new();
        assert_eq!(
            repl.unset("Number"),
            Err("Cannot unset Number: it's a built-in type".to_string())
        );
    }

    #[test]
    fn test_unset_function_keeps_it() {
        let mut repl = FltRepl::new();
        assert_eq!(
            repl.unset("typeof"),
            Err("Cannot unset typeof: it's a function".to_string())
        );
        assert!(repl.runtime.global_scope.has_function("typeof"));
    }

    #[test]
    fn test_inspect_native_type() {
        let mut repl = FltRepl::new();
        repl.runtime.register_native_type::<Meters>();
        assert_eq!(repl.inspect_identifier("Meters"), "(native type) Meters");
    }

    #[test]
    fn test_unset_native_type() {
        let mut repl = FltRepl::new();
        repl.runtime.register_native_type::<Meters>();
        assert_eq!(
            repl.unset("Meters"),
            Err("Cannot unset Meters: it's a native type".to_string())
        );
        assert_eq!(
            repl.runtime.get_native_type("Meters").map(|t| t.name()),
            Some("Meters")
        );
    }

    #[test]
    fn test_unset_without_identifier() {
        let mut repl = FltRepl::new();
        assert_eq!(
            repl.unset(""),
            Err("usage: /unset <identifier>".to_string())
        );
    }

    #[test]
    fn test_inspect_variable() {
        let mut repl = FltRepl::new();
        let statement = FltRepl::parse_full("let x = 1").unwrap();
        repl.runtime.eval(&statement).unwrap();
        assert_eq!(repl.inspect_identifier("x"), "(variable) 1");
    }

    #[test]
    fn test_inspect_unbound_identifier() {
        let repl = FltRepl::new();
        assert_eq!(repl.inspect_identifier("nope"), "Unbound variable");
    }
}
