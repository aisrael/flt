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
    /// A host's handler using flt's API wrongly: a bug in the host, not in the flt source
    #[error("Handler Error: {0}")]
    HandlerError(HandlerError),
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

/// A native function's handler using flt's API wrongly, raised at the interface between flt and
/// the host's handlers. Unlike [`Error::Native`], which carries the host's own errors, it's flt
/// reporting a bug in the handler.
#[derive(Debug, Error, PartialEq)]
pub enum HandlerError {
    /// The handler read a name that the overload that matched doesn't declare
    #[error("Function {name} has no argument {argument}")]
    UndeclaredArgument { name: String, argument: String },
    /// The handler read a variadic argument, which has any number of values, as one value
    #[error("Argument {argument} of {name} is variadic; read it with Arguments::rest")]
    VariadicArgument { name: String, argument: String },
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
    /// No signature of the function takes that many positional arguments
    #[error("Function {name} expected {expected} argument(s), found {found}")]
    ArgumentCountMismatch {
        name: String,
        expected: ArgumentCounts,
        found: usize,
    },
    /// The one signature that takes that many positional arguments doesn't accept this one
    #[error("Argument {argument} of {name} expects {expected}, found {found}")]
    ArgumentTypeMismatch {
        name: String,
        argument: String,
        expected: String,
        found: String,
    },
    /// Several signatures take that many positional arguments, and none accepts the call
    #[error("No overload of {name} accepts ({})", .found.join(", "))]
    NoMatchingOverload { name: String, found: Vec<String> },
    #[error("Function {name} got an unexpected keyword argument: {key}")]
    UnexpectedKeywordArgument { name: String, key: String },
    #[error("Keyword argument {key} of {name} expects {expected}, found {found}")]
    KeywordTypeMismatch {
        name: String,
        key: String,
        expected: String,
        found: String,
    },
    /// A function's result isn't of its signature's return type
    #[error("Function {name} returned {found}, expected {expected}")]
    ReturnTypeMismatch {
        name: String,
        expected: String,
        found: String,
    },
    #[error("Cannot bind {name}: it's {reason}")]
    CannotBind { name: String, reason: ProtectedName },
    /// A variable declared with a type is bound to a value of another type
    #[error("Variable {name} expects {expected}, found {found}")]
    VariableTypeMismatch {
        name: String,
        expected: String,
        found: String,
    },
    /// A type in a declaration names something that isn't a type
    #[error("Unknown type: {0}")]
    UnknownType(String),
    /// A host registered a function or a native type (`kind`) under a name that's in use
    #[error("Cannot register {kind} {name}: it's {reason}")]
    CannotRegister {
        kind: &'static str,
        name: String,
        reason: ProtectedName,
    },
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

/// How many positional arguments a signature takes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArgumentCount {
    Exactly(usize),
    /// A signature with a variadic argument takes at least this many
    AtLeast(usize),
}

impl ArgumentCount {
    fn count(self) -> usize {
        match self {
            ArgumentCount::Exactly(n) | ArgumentCount::AtLeast(n) => n,
        }
    }

    fn covers(self, other: ArgumentCount) -> bool {
        match self {
            ArgumentCount::Exactly(n) => other == ArgumentCount::Exactly(n),
            ArgumentCount::AtLeast(n) => other.count() >= n,
        }
    }
}

impl std::fmt::Display for ArgumentCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentCount::Exactly(n) => write!(f, "{n}"),
            ArgumentCount::AtLeast(n) => write!(f, "at least {n}"),
        }
    }
}

/// Every count of positional arguments a function's signatures take, smallest first, e.g.
/// `1, 2 or at least 3`. A count another one already covers is left out.
#[derive(Clone, Debug, PartialEq)]
pub struct ArgumentCounts(Vec<ArgumentCount>);

impl ArgumentCounts {
    pub fn new(counts: impl IntoIterator<Item = ArgumentCount>) -> Self {
        let mut counts: Vec<ArgumentCount> = counts.into_iter().collect();
        // A minimum sorts before the same exact count, which it covers
        counts.sort_by_key(|count| (count.count(), matches!(count, ArgumentCount::Exactly(_))));
        let mut kept: Vec<ArgumentCount> = Vec::new();
        for count in counts {
            if !kept.iter().any(|earlier| earlier.covers(count)) {
                kept.push(count);
            }
        }
        Self(kept)
    }

    /// The counts, smallest first
    pub fn counts(&self) -> &[ArgumentCount] {
        &self.0
    }
}

impl std::fmt::Display for ArgumentCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, count) in self.0.iter().enumerate() {
            if i > 0 {
                let separator = if i == self.0.len() - 1 { " or " } else { ", " };
                write!(f, "{separator}")?;
            }
            write!(f, "{count}")?;
        }
        Ok(())
    }
}

/// What a name already is, which stops it from being bound as a variable or registered
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtectedName {
    /// A built-in type name, e.g. `Number`
    BuiltinType,
    /// The name of a function in the global scope, built-in or registered by the host
    Function,
    /// The name of a native type registered by the host
    NativeType,
    /// The name of a variable, which a function or a native type can't use
    Variable,
}

impl std::fmt::Display for ProtectedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProtectedName::BuiltinType => "a built-in type",
            ProtectedName::Function => "a function",
            ProtectedName::NativeType => "a native type",
            ProtectedName::Variable => "a variable",
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

    #[test]
    fn test_cannot_register_error_message() {
        for (kind, name, reason, expected) in [
            (
                "function",
                "Number",
                ProtectedName::BuiltinType,
                "Cannot register function Number: it's a built-in type",
            ),
            (
                "function",
                "x",
                ProtectedName::Variable,
                "Cannot register function x: it's a variable",
            ),
            (
                "native type",
                "typeof",
                ProtectedName::Function,
                "Cannot register native type typeof: it's a function",
            ),
            (
                "native type",
                "Date",
                ProtectedName::NativeType,
                "Cannot register native type Date: it's a native type",
            ),
        ] {
            let error = Error::RuntimeError(RuntimeError::CannotRegister {
                kind,
                name: name.to_string(),
                reason,
            });
            assert_eq!(error.to_string(), format!("Runtime Error: {expected}"));
        }
    }
}
