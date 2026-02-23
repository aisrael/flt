use std::fmt::Display;

/// Unary operand: `!`, `+`, `-`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
}

/// Binary operand: `+`, `-`, `*`, `/`, `&`, `&&`, `|`, `||`, `^`, `^^`, `|>`, `<>`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    And,
    BitOr,
    Or,
    BitXor,
    Xor,
    /// Elixir-style pipe: passes left as first argument to right.
    Pipe,
    /// String concatenation: concatenates two strings (e.g. `"foo" <> "bar"` → `"foobar"`).
    Concat,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::BitAnd => write!(f, "&"),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::BitOr => write!(f, "|"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::BitXor => write!(f, "^"),
            BinaryOp::Xor => write!(f, "^^"),
            BinaryOp::Pipe => write!(f, "|>"),
            BinaryOp::Concat => write!(f, "<>"),
        }
    }
}
