use std::path::PathBuf;

use crate::ast::Identifier;
use crate::ast::Statement;
use crate::parser::parse_statement;
use crate::runtime::Runtime;
use crate::runtime::SimpleRuntime;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Maximum number of inputs to keep in REPL history.
const HISTORY_DEPTH: usize = 1000;

fn repl_history_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("flt").join("history"))
}

pub struct Repl {
    editor: DefaultEditor,
    runtime: SimpleRuntime,
}

impl Repl {
    pub fn new() -> Result<Self, ReadlineError> {
        let config = rustyline::Config::builder()
            .max_history_size(HISTORY_DEPTH)
            .expect("valid history size")
            .auto_add_history(true)
            .build();
        let editor = DefaultEditor::with_config(config)?;
        let mut repl = Self {
            editor,
            runtime: SimpleRuntime::default(),
        };
        repl.load_history()?;
        Ok(repl)
    }

    fn load_history(&mut self) -> Result<(), ReadlineError> {
        let Some(history_path) = repl_history_path() else {
            return Ok(());
        };
        if history_path.exists() {
            println!("Loading REPL history from: {:?}", history_path);
            self.editor.load_history(&history_path)?;
        }
        Ok(())
    }

    fn save_history(&mut self) -> Result<(), ReadlineError> {
        let Some(history_path) = repl_history_path() else {
            return Ok(());
        };
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        self.editor.save_history(&history_path)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), ReadlineError> {
        let repl_result = self.repl_loop();
        let _ = self.save_history();
        repl_result
    }

    fn repl_loop(&mut self) -> Result<(), ReadlineError> {
        loop {
            let line = match self.editor.readline("> ") {
                Ok(line) => line,
                Err(ReadlineError::Eof) => break Ok(()),
                Err(ReadlineError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix('/') {
                self.handle_command(rest);
            } else {
                match Self::parse_full(line) {
                    Ok(statement) => match self.runtime.eval(&statement) {
                        Ok(val) => println!("{}", val),
                        Err(e) => eprintln!("eval error: {:?}", e),
                    },
                    Err(msg) => eprintln!("{}", msg),
                }
            }
            println!();
        }
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

    fn handle_command(&mut self, rest: &str) {
        let (cmd, args) = match rest.split_once(char::is_whitespace) {
            Some((cmd, args)) => (cmd, args.trim()),
            None => (rest, ""),
        };
        match cmd {
            "parse" => Self::handle_parse(args),
            "unset" => self.handle_unset(args),
            _ => eprintln!("unknown command: /{cmd}"),
        }
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
}
