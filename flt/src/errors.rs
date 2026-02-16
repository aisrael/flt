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
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Invalid Operand Type")]
    InvalidOperandType,
    #[error("Cannot compare {0} and {1}")]
    CannotCompare(String, String),
    #[error("Division By Zero")]
    DivisionByZero,
}
