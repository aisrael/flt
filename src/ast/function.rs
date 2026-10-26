//! AST types for function calls.

use std::fmt::Display;

use super::expr::Expr;
use super::expr::KeyValue;
use super::identifier::Identifier;

/// A function call: name, positional arguments, then optional keyword arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    /// Positional arguments (must come first).
    pub positional_args: Vec<Expr>,
    /// Trailing keyword arguments (e.g. `foo(1, bar: true)`), in source order. A key holds its
    /// unquoted text, and no key appears twice.
    pub keyword_args: Vec<KeyValue>,
}

/// Displays the call as `name(p1, p2, k: v)`.
impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        write!(f, "(")?;
        let positional = self.positional_args.iter().map(|arg| arg as &dyn Display);
        let keywords = self.keyword_args.iter().map(|kv| kv as &dyn Display);
        for (i, arg) in positional.chain(keywords).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.fmt(f)?;
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
