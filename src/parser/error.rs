//! Parser errors

use std::fmt;

use nom::error::ErrorKind;
use nom::error::FromExternalError;

use crate::utils::escape_string;
use crate::utils::is_bare_name;
use crate::Error;

/// The result of an flt parser: the remaining input and the parsed value, or a [`ParseError`]
pub type IResult<I, O> = nom::IResult<I, O, ParseError<I>>;

/// A parse error: where parsing failed, and why
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError<I> {
    /// The input where parsing failed
    pub input: I,
    pub kind: ParseErrorKind,
}

/// Why parsing failed
#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    /// A nom parser didn't match
    Nom(ErrorKind),
    /// A call passes the same keyword argument more than once, e.g. `f(n: 1, n: 2)`
    DuplicateKeyword { function: String, keyword: String },
}

impl<I> ParseError<I> {
    /// An error from a nom parser that didn't match
    pub fn new(input: I, kind: ErrorKind) -> Self {
        Self {
            input,
            kind: ParseErrorKind::Nom(kind),
        }
    }

    /// A call to `function` that repeats `keyword`
    pub fn duplicate_keyword(input: I, function: &str, keyword: &str) -> Self {
        Self {
            input,
            kind: ParseErrorKind::DuplicateKeyword {
                function: function.to_string(),
                keyword: keyword.to_string(),
            },
        }
    }
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::new(input, kind)
    }

    /// Keeps the innermost error, which says the most about what went wrong
    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for ParseError<I> {
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        Self::new(input, kind)
    }
}

/// A nom error prints as nom prints it. Other errors say what's wrong in flt's terms.
impl<I: fmt::Display> fmt::Display for ParseError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::Nom(kind) => write!(f, "error {kind:?} at: {}", self.input),
            ParseErrorKind::DuplicateKeyword { function, keyword } => {
                write!(f, "Duplicate keyword argument ")?;
                if is_bare_name(keyword) {
                    write!(f, "{keyword}")?;
                } else {
                    write!(f, "\"{}\"", escape_string(keyword))?;
                }
                write!(f, " in the call to {function}")
            }
        }
    }
}

impl ParseError<&str> {
    /// The message for an error that nom reports with `prefix`. A nom error reads as nom's
    /// own error does, as it did before flt had its own error type; other errors read as
    /// their message alone.
    fn message(&self, prefix: &str) -> String {
        match self.kind {
            ParseErrorKind::Nom(kind) => {
                format!("{prefix}: {:?}", nom::error::Error::new(self.input, kind))
            }
            _ => self.to_string(),
        }
    }
}

impl From<nom::Err<ParseError<&str>>> for Error {
    fn from(error: nom::Err<ParseError<&str>>) -> Self {
        Error::ParserError(match error {
            nom::Err::Error(error) => error.message("Parsing Error"),
            nom::Err::Failure(error) => error.message("Parsing Failure"),
            incomplete => incomplete.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_keyword_message() {
        assert_eq!(
            ParseError::duplicate_keyword("", "greet", "greeting").to_string(),
            "Duplicate keyword argument greeting in the call to greet"
        );
        assert_eq!(
            ParseError::duplicate_keyword("", "f", "c d").to_string(),
            r#"Duplicate keyword argument "c d" in the call to f"#
        );
    }

    #[test]
    fn test_into_error() {
        let error = Error::from(nom::Err::Failure(ParseError::duplicate_keyword(
            "", "f", "n",
        )));
        assert_eq!(
            error.to_string(),
            "Parser Error: Duplicate keyword argument n in the call to f"
        );
        let error = Error::from(nom::Err::Error(ParseError::new("x", ErrorKind::Tag)));
        assert_eq!(
            error.to_string(),
            r#"Parser Error: Parsing Error: Error { input: "x", code: Tag }"#
        );
    }
}
