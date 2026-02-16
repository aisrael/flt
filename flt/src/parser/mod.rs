//! The flt Parser

mod boolean;
mod expr;
mod function;
mod identifier;
mod literal;
mod number;
mod operands;
mod string;
mod symbol;

pub use crate::ast::BinaryOp;
pub use crate::ast::Expr;
pub use crate::ast::Literal;
pub use crate::ast::UnaryOp;
pub use expr::parse_expr;
pub use identifier::parse_identifier;
pub use literal::parse_literal;
pub use number::parse_number;
pub use operands::parse_binary_op;
pub use operands::parse_unary_op;
pub use string::parse_string;
pub use symbol::parse_symbol;
