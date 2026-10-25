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
    /// An error raised by a host's native function, shown unchanged
    #[error(transparent)]
    Native(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    /// Wraps a host's own error, so that a native function can fail with it
    pub fn native(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::Native(Box::new(error))
    }
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
    /// A native function was given a value of the wrong type, e.g. for a native or number argument
    #[error("Expected {expected}, found {found}")]
    UnexpectedType { expected: String, found: String },
    /// A number doesn't fit the Rust number type a native function reads it as
    #[error("Cannot convert {number} to {target}")]
    CannotConvert {
        number: String,
        target: &'static str,
    },
}

/// Why a name can't be bound as a variable
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtectedName {
    /// A built-in type name, e.g. `Number`
    BuiltinType,
    /// The name of a function in the global scope, built-in or registered by the host
    Function,
    /// The name of a native type registered by the host
    NativeType,
}

impl std::fmt::Display for ProtectedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProtectedName::BuiltinType => "a built-in type",
            ProtectedName::Function => "a function",
            ProtectedName::NativeType => "a native type",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host's own error type
    #[derive(Debug, PartialEq)]
    struct DiskFull(u8);

    impl std::fmt::Display for DiskFull {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "disk {} is full", self.0)
        }
    }

    impl std::error::Error for DiskFull {}

    #[test]
    fn test_native_error_displays_as_the_host_error() {
        assert_eq!(Error::native(DiskFull(2)).to_string(), "disk 2 is full");
    }

    #[test]
    fn test_native_error_downcasts_to_the_host_error() {
        match Error::native(DiskFull(2)) {
            Error::Native(error) => {
                assert_eq!(error.downcast_ref::<DiskFull>(), Some(&DiskFull(2)))
            }
            other => panic!("expected a native error, got {other:?}"),
        }
    }

    #[test]
    fn test_runtime_errors_for_native_values() {
        let unexpected = Error::RuntimeError(RuntimeError::UnexpectedType {
            expected: "Date".to_string(),
            found: "Number".to_string(),
        });
        assert_eq!(
            unexpected.to_string(),
            "Runtime Error: Expected Date, found Number"
        );
        let cannot_convert = Error::RuntimeError(RuntimeError::CannotConvert {
            number: "2.5".to_string(),
            target: "u32",
        });
        assert_eq!(
            cannot_convert.to_string(),
            "Runtime Error: Cannot convert 2.5 to u32"
        );
        assert_eq!(ProtectedName::NativeType.to_string(), "a native type");
    }
}
