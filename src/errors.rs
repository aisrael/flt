//! The flt::Error enum
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("f64 value cannot be converted to number literal (NaN, Infinity, or out of range)")]
    F64ConversionError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    LexerError(String),
    #[error("Parser Error: {0}")]
    ParserError(String),
    #[error("Syntax Error: {0}")]
    SyntaxError(String),
    #[error("Runtime Error: {0}")]
    RuntimeError(RuntimeError),
    #[error("Interpreter Error: {0}")]
    InterpreterError(InterpreterError),
}

#[derive(Debug, Error, PartialEq)]
pub enum InterpreterError {
    #[error("Not yet implemented")]
    NotYetImplemented,
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Invalid Operand Type")]
    InvalidOperandType,
    #[error("Cannot compare {0} and {1}")]
    CannotCompare(String, String),
    #[error("Division By Zero")]
    DivisionByZero,
    #[error("Unbound identifier: {0}")]
    UnboundIdentifier(String),
    #[error("No such field: {0}")]
    NoSuchField(String),
    #[error("Cannot pipe into {0}")]
    InvalidPipeTarget(String),
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),
    #[error("{0} is not a function")]
    NotCallable(String),
    #[error("Function {name} expected {expected} argument(s), found {found}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
    #[error("Cannot bind {name}: it's {reason}")]
    CannotBind { name: String, reason: ProtectedName },
}

/// Why a name can't be bound as a variable
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtectedName {
    /// A built-in type name, e.g. `Number`
    BuiltinType,
    /// The name of a function in the global scope, built-in or registered by the host
    Function,
}

impl std::fmt::Display for ProtectedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProtectedName::BuiltinType => "a built-in type",
            ProtectedName::Function => "a function",
        };
        write!(f, "{s}")
    }
}
