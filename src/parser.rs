//! The flt Parser

mod array;
mod boolean;
mod comment;
mod error;
mod expr;
mod function;
mod identifier;
mod keyword;
mod literal;
mod map;
mod number;
mod operands;
mod statement;
mod string;
mod symbol;
mod type_expr;

pub use crate::ast::BinaryOp;
pub use crate::ast::Expr;
pub use crate::ast::Literal;
pub use crate::ast::UnaryOp;
pub use error::IResult;
pub use error::ParseError;
pub use error::ParseErrorKind;
pub use expr::parse_expr;
pub use identifier::parse_identifier;
pub use literal::parse_literal;
pub use number::parse_number;
pub use operands::parse_binary_op;
pub use operands::parse_unary_op;
pub use statement::parse_statement;
pub use statement::parse_statements;
pub use string::parse_string;
pub use symbol::parse_symbol;
pub use type_expr::parse_type;
