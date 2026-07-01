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
    #[error("Function calls not yet supported")]
    UnsupportedFunctionCall,
    #[error("Function {name} expected {expected} argument(s), found {found}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
}
