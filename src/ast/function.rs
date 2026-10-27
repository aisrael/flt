//! AST types for function calls.

use std::fmt::Display;

use indexmap::IndexMap;

use super::expr::Expr;
use super::identifier::Identifier;
use crate::utils::escape_string;
use crate::utils::is_bare_name;

/// A call's keyword arguments, keyed by their unquoted text, in source order.
pub type KeywordArgs = IndexMap<String, Expr>;

/// A function call: name, positional arguments, then optional keyword arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    /// Positional arguments (must come first).
    pub positional_args: Vec<Expr>,
    /// Trailing keyword arguments (e.g. `foo(1, bar: true)`).
    pub keyword_args: KeywordArgs,
}

/// Displays the call as `name(p1, p2, k: v)`.
impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        write!(f, "(")?;
        for (i, arg) in self.positional_args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.fmt(f)?;
        }
        for (i, (key, value)) in self.keyword_args.iter().enumerate() {
            if i > 0 || !self.positional_args.is_empty() {
                write!(f, ", ")?;
            }
            if is_bare_name(key) {
                write!(f, "{key}: {value}")?;
            } else {
                write!(f, "\"{}\": {value}", escape_string(key))?;
            }
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;

    #[test]
    fn test_display() {
        assert_eq!(Expr::function_call("f", vec![]).to_string(), "f()");
        assert_eq!(
            Expr::function_call_with_keywords(
                "f",
                vec![Expr::literal_number(1)],
                vec![
                    ("k", Expr::literal_number(2)),
                    ("c d", Expr::literal_number(3))
                ],
            )
            .to_string(),
            r#"f(1, k: 2, "c d": 3)"#
        );
        assert_eq!(
            Expr::function_call_with_keywords("f", vec![], vec![("k", Expr::literal_number(2))])
                .to_string(),
            "f(k: 2)"
        );
    }
}
