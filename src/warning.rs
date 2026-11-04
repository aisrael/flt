//! Warnings: legal code that is probably a mistake. A warning does not stop a statement.
use std::fmt::Display;

/// A warning that a runtime gives for code it runs ([ADR 024]).
///
/// [ADR 024]: https://github.com/aisrael/flt/blob/main/docs/adrs/024-unreachable-code-warning.md
#[derive(Clone, Debug, PartialEq)]
pub enum Warning {
    /// A function whose body has a statement after a `return` in the same list of statements
    UnreachableCode { function: String },
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::UnreachableCode { function } => {
                write!(
                    f,
                    "Unreachable code in {function}: statements after return never run"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unreachable_code_message() {
        let warning = Warning::UnreachableCode {
            function: "f".to_string(),
        };
        assert_eq!(
            warning.to_string(),
            "Unreachable code in f: statements after return never run"
        );
    }
}
