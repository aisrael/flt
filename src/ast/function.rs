//! AST types for function calls.

use super::expr::Expr;
use super::expr::KeyValue;
use super::identifier::Identifier;

/// A function call: name, positional arguments, then optional key-value pairs.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    /// Positional arguments (must come first).
    pub positional_args: Vec<Expr>,
    /// Trailing key-value pairs (e.g. `foo(1, bar: true)`).
    pub keyword_args: Vec<KeyValue>,
}

impl FunctionCall {
    /// Converts the argument list to the form used in `Expr::FunctionCall`: positional
    /// args first, with keyword args collected into a single `MapLiteral` as the
    /// final argument if present.
    pub fn args_as_exprs(&self) -> Vec<Expr> {
        let mut exprs = self.positional_args.clone();
        if !self.keyword_args.is_empty() {
            exprs.push(Expr::MapLiteral(self.keyword_args.clone()));
        }
        exprs
    }
}
