use std::fmt::Display;

use super::expr::Expr;
use super::identifier::Identifier;

/// A statement in the language.
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// A let binding: `let ident = expr`.
    Let(Identifier, Expr),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(ident, expr) => write!(f, "let {} = {}", ident, expr),
        }
    }
}
