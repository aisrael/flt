use bigdecimal::BigDecimal;

use super::identifier::Identifier;
use super::literal::Literal;
use super::operands::BinaryOp;
use super::operands::UnaryOp;

/// An expression in the language.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal value (number, string, or boolean).
    Literal(Literal),
    /// An identifier.
    Ident(String),
    /// A unary expression with an operator and operand.
    UnaryExpr(UnaryOp, Box<Expr>),
    /// A binary expression with left, operator, and right operands.
    BinaryExpr(Box<Expr>, BinaryOp, Box<Expr>),
    /// A function call: name and arguments.
    FunctionCall(Identifier, Vec<Expr>),
    /// A parenthesized expression.
    Parenthesized(Box<Expr>),
}

impl Expr {
    /// Constructs a boolean literal expression.
    pub fn literal_boolean(b: bool) -> Self {
        Expr::Literal(Literal::boolean(b))
    }

    /// Constructs a symbol literal expression (e.g. `:foo` or `:"hello"`).
    pub fn literal_symbol(s: impl Into<String>) -> Self {
        Expr::Literal(Literal::symbol(s))
    }

    /// Constructs a string literal expression (e.g. `"hello"`).
    pub fn literal_string(s: impl Into<String>) -> Self {
        Expr::Literal(Literal::string(s))
    }

    /// Constructs a number literal expression from a string (e.g. `"3.14"`).
    pub fn literal_number(n: impl Into<BigDecimal>) -> Self {
        Expr::Literal(Literal::number(n))
    }

    /// Constructs an identifier expression.
    pub fn ident(s: impl Into<String>) -> Self {
        Expr::Ident(s.into())
    }

    /// Constructs a unary expression.
    pub fn unary_expr(op: UnaryOp, expr: Expr) -> Self {
        Expr::UnaryExpr(op, Box::new(expr))
    }

    /// Constructs a binary expression.
    pub fn binary_expr(left: Expr, op: BinaryOp, right: Expr) -> Self {
        Expr::BinaryExpr(Box::new(left), op, Box::new(right))
    }

    /// Constructs a function call expression.
    pub fn function_call(
        name: impl TryInto<Identifier, Error = crate::Error>,
        args: Vec<Expr>,
    ) -> Self {
        Expr::FunctionCall(name.try_into().expect("failed to convert identifier"), args)
    }

    /// Constructs a parenthesized expression.
    pub fn parenthesized(expr: Expr) -> Self {
        Expr::Parenthesized(Box::new(expr))
    }
}
