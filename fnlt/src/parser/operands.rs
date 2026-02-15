use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::IResult;

use crate::ast::BinaryOp;
use crate::ast::UnaryOp;

/// Parses a unary operand: `!`, `+`, or `-`.
pub fn parse_unary_op(input: &str) -> IResult<&str, UnaryOp> {
    alt((
        value(UnaryOp::Not, tag("!")),
        value(UnaryOp::Plus, tag("+")),
        value(UnaryOp::Minus, tag("-")),
    ))(input)
}

/// Parses a binary operand. Longer tokens must be tried first (`&&` before `&`, `||` before `|`, `^^` before `^`, `|>` before `|`).
pub fn parse_binary_op(input: &str) -> IResult<&str, BinaryOp> {
    alt((
        value(BinaryOp::BitAnd, tag("&")),
        value(BinaryOp::BitOr, tag("|")),
        value(BinaryOp::BitXor, tag("^")),
        value(BinaryOp::Mul, tag("*")),
        value(BinaryOp::Add, tag("+")),
        value(BinaryOp::Div, tag("/")),
        value(BinaryOp::Sub, tag("-")),
        value(BinaryOp::And, tag("&&")),
        value(BinaryOp::Or, tag("||")),
        value(BinaryOp::Xor, tag("^^")),
        value(BinaryOp::Pipe, tag("|>")),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unary_op() {
        assert_eq!(parse_unary_op("!"), Ok(("", UnaryOp::Not)));
        assert_eq!(parse_unary_op("+"), Ok(("", UnaryOp::Plus)));
        assert_eq!(parse_unary_op("-"), Ok(("", UnaryOp::Minus)));
    }

    #[test]
    fn test_parse_unary_op_with_remainder() {
        assert_eq!(parse_unary_op("!x"), Ok(("x", UnaryOp::Not)));
        assert_eq!(parse_unary_op("+ 1"), Ok((" 1", UnaryOp::Plus)));
    }

    #[test]
    fn test_parse_binary_op_single_char() {
        assert_eq!(parse_binary_op("+"), Ok(("", BinaryOp::Add)));
        assert_eq!(parse_binary_op("-"), Ok(("", BinaryOp::Sub)));
        assert_eq!(parse_binary_op("*"), Ok(("", BinaryOp::Mul)));
        assert_eq!(parse_binary_op("/"), Ok(("", BinaryOp::Div)));
        assert_eq!(parse_binary_op("&"), Ok(("", BinaryOp::BitAnd)));
        assert_eq!(parse_binary_op("|"), Ok(("", BinaryOp::BitOr)));
        assert_eq!(parse_binary_op("^"), Ok(("", BinaryOp::BitXor)));
    }

    #[test]
    fn test_parse_binary_op_double_char() {
        assert_eq!(parse_binary_op("&&"), Ok(("", BinaryOp::And)));
        assert_eq!(parse_binary_op("||"), Ok(("", BinaryOp::Or)));
        assert_eq!(parse_binary_op("^^"), Ok(("", BinaryOp::Xor)));
        assert_eq!(parse_binary_op("|>"), Ok(("", BinaryOp::Pipe)));
    }

    #[test]
    fn test_parse_binary_op_double_char_not_consumed_as_single() {
        // `&&` should parse as And, not as two BitAnd
        assert_eq!(parse_binary_op("&&"), Ok(("", BinaryOp::And)));
        assert_eq!(parse_binary_op("||"), Ok(("", BinaryOp::Or)));
        assert_eq!(parse_binary_op("^^"), Ok(("", BinaryOp::Xor)));
    }

    #[test]
    fn test_parse_binary_op_with_remainder() {
        assert_eq!(parse_binary_op("+ 1"), Ok((" 1", BinaryOp::Add)));
        assert_eq!(parse_binary_op("&& true"), Ok((" true", BinaryOp::And)));
    }
}
