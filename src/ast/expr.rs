use std::fmt::Display;

use bigdecimal::BigDecimal;

use super::identifier::Identifier;
use super::literal::Literal;
use super::operands::BinaryOp;
use super::operands::UnaryOp;
use crate::utils::escape_string;

/// A key-value pair in a map literal.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValue {
    pub key: String,
    pub value: Expr,
}

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
    /// A map literal: `{ key: value, ... }`.
    MapLiteral(Vec<KeyValue>),
    /// An array literal: `[ expr, ... ]`.
    ArrayLiteral(Vec<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Ident(ident) => write!(f, "{}", ident),
            Expr::UnaryExpr(op, expr) => write!(f, "{op}{expr}"),
            Expr::BinaryExpr(left, op, right) => write!(f, "{left} {op} {right}"),
            Expr::FunctionCall(name, args) => {
                let args = args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{name}({args})")
            }
            Expr::Parenthesized(expr) => write!(f, "({expr})"),
            Expr::MapLiteral(entries) => {
                write!(f, "{{ ")?;
                for (i, kv) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if kv.key.contains(|c: char| !c.is_alphanumeric() && c != '_') {
                        write!(f, "\"{}\": {}", escape_string(&kv.key), kv.value)?;
                    } else {
                        write!(f, "{}: {}", kv.key, kv.value)?;
                    }
                }
                write!(f, " }}")
            }
            Expr::ArrayLiteral(elems) => {
                write!(f, "[ ")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{e}")?;
                }
                write!(f, " ]")
            }
        }
    }
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

    /// Constructs a map literal expression.
    pub fn map_literal(entries: Vec<(impl Into<String>, Expr)>) -> Self {
        Expr::MapLiteral(
            entries
                .into_iter()
                .map(|(k, v)| KeyValue {
                    key: k.into(),
                    value: v,
                })
                .collect(),
        )
    }

    /// Constructs an array literal expression.
    pub fn array_literal(elems: Vec<Expr>) -> Self {
        Expr::ArrayLiteral(elems)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use super::super::operands::BinaryOp;
    use super::super::operands::UnaryOp;
    use super::Expr;

    fn n(s: &str) -> BigDecimal {
        BigDecimal::from_str(s).unwrap()
    }

    #[test]
    fn test_display_literal_number() {
        assert_eq!(Expr::literal_number(n("42")).to_string(), "42");
        assert_eq!(Expr::literal_number(n("3.14")).to_string(), "3.14");
    }

    #[test]
    fn test_display_literal_string() {
        assert_eq!(Expr::literal_string("hello").to_string(), "\"hello\"");
    }

    #[test]
    fn test_display_literal_boolean() {
        assert_eq!(Expr::literal_boolean(true).to_string(), "true");
        assert_eq!(Expr::literal_boolean(false).to_string(), "false");
    }

    #[test]
    fn test_display_literal_symbol() {
        assert_eq!(Expr::literal_symbol("foo").to_string(), "foo");
    }

    #[test]
    fn test_display_ident() {
        assert_eq!(Expr::ident("x").to_string(), "x");
        assert_eq!(Expr::ident("foo-bar").to_string(), "foo-bar");
    }

    #[test]
    fn test_display_unary_expr() {
        assert_eq!(
            Expr::unary_expr(UnaryOp::Not, Expr::literal_boolean(true)).to_string(),
            "!true"
        );
        assert_eq!(
            Expr::unary_expr(UnaryOp::Minus, Expr::literal_number(n("42"))).to_string(),
            "-42"
        );
        assert_eq!(
            Expr::unary_expr(UnaryOp::Plus, Expr::literal_number(n("1"))).to_string(),
            "+1"
        );
    }

    #[test]
    fn test_display_binary_expr() {
        assert_eq!(
            Expr::binary_expr(
                Expr::literal_number(n("1")),
                BinaryOp::Add,
                Expr::literal_number(n("2"))
            )
            .to_string(),
            "1 + 2"
        );
        assert_eq!(
            Expr::binary_expr(
                Expr::literal_string("foo"),
                BinaryOp::Concat,
                Expr::literal_string("bar")
            )
            .to_string(),
            "\"foo\" <> \"bar\""
        );
        assert_eq!(
            Expr::binary_expr(Expr::ident("a"), BinaryOp::Pipe, Expr::ident("b")).to_string(),
            "a |> b"
        );
    }

    #[test]
    fn test_display_function_call() {
        assert_eq!(Expr::function_call("foo", vec![]).to_string(), "foo()");
        assert_eq!(
            Expr::function_call(
                "add",
                vec![Expr::literal_number(n("1")), Expr::literal_number(n("2"))]
            )
            .to_string(),
            "add(1, 2)"
        );
        assert_eq!(
            Expr::function_call(
                "concat",
                vec![
                    Expr::literal_string("hello"),
                    Expr::ident("name"),
                    Expr::literal_string("!")
                ]
            )
            .to_string(),
            "concat(\"hello\", name, \"!\")"
        );
    }

    #[test]
    fn test_display_parenthesized() {
        assert_eq!(
            Expr::parenthesized(Expr::literal_number(n("42"))).to_string(),
            "(42)"
        );
        assert_eq!(
            Expr::parenthesized(Expr::binary_expr(
                Expr::literal_number(n("1")),
                BinaryOp::Add,
                Expr::literal_number(n("2"))
            ))
            .to_string(),
            "(1 + 2)"
        );
    }

    #[test]
    fn test_display_nested() {
        let expr = Expr::binary_expr(
            Expr::parenthesized(Expr::binary_expr(
                Expr::literal_number(n("1")),
                BinaryOp::Add,
                Expr::literal_number(n("2")),
            )),
            BinaryOp::Mul,
            Expr::literal_number(n("3")),
        );
        assert_eq!(expr.to_string(), "(1 + 2) * 3");
    }
}
