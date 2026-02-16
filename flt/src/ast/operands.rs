/// Unary operand: `!`, `+`, `-`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
}

/// Binary operand: `+`, `-`, `*`, `/`, `&`, `&&`, `|`, `||`, `^`, `^^`, `|>`.
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
}
