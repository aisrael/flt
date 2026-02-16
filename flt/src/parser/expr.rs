use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::map;
use nom::combinator::verify;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

use super::function::parse_function_call;
use super::identifier::parse_identifier;
use super::literal::parse_literal;
use super::operands::parse_binary_op;
use super::operands::parse_unary_op;
use crate::ast::BinaryOp;
use crate::ast::Expr;

/// Parses a primary expression: literal, identifier, function call, or parenthesized expression.
fn parse_primary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(parse_literal, Expr::Literal),
        map(parse_function_call(parse_or), |(name, args)| {
            Expr::FunctionCall(name, args)
        }),
        map(parse_identifier, Expr::ident),
        map(
            delimited(
                tuple((multispace0, tag("("), multispace0)),
                parse_or,
                tuple((multispace0, tag(")"), multispace0)),
            ),
            Expr::parenthesized,
        ),
    ))(input)
}

/// Parses a unary expression: optionally prefixed with `!`, `+`, or `-`.
fn parse_unary(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;
    alt((
        map(
            tuple((parse_unary_op, multispace0, parse_unary)),
            |(op, _, e)| Expr::unary_expr(op, e),
        ),
        parse_primary,
    ))(input)
}

/// Parses binary expressions: `Expr` then `BinaryOp` then `Expr`, with left-associative folding.
/// `next` parses the higher-precedence operand; `allowed` restricts which operators this level accepts.
/// Precedence (lowest to highest): ||, &&, ^^, |, ^, &, +/-, *, /
fn parse_binary_level<'a>(
    input: &'a str,
    next: fn(&str) -> IResult<&str, Expr>,
    allowed: &[BinaryOp],
) -> IResult<&'a str, Expr> {
    let parse_expr_binary_op_expr = tuple((
        next,
        many0(tuple((
            multispace0,
            verify(parse_binary_op, |o: &BinaryOp| allowed.contains(o)),
            multispace0,
            next,
        ))),
    ));
    map(
        parse_expr_binary_op_expr,
        |(left, pairs): (Expr, Vec<(_, BinaryOp, _, Expr)>)| {
            pairs.into_iter().fold(left, |acc, (_, op, _, right)| {
                Expr::binary_expr(acc, op, right)
            })
        },
    )(input)
}

fn parse_pipe(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_or, &[BinaryOp::Pipe])
}

fn parse_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_and, &[BinaryOp::Or])
}

fn parse_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_xor, &[BinaryOp::And])
}

fn parse_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_or, &[BinaryOp::Xor])
}

fn parse_bit_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_xor, &[BinaryOp::BitOr])
}

fn parse_bit_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_and, &[BinaryOp::BitXor])
}

fn parse_bit_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_add_sub, &[BinaryOp::BitAnd])
}

fn parse_add_sub(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_mul_div, &[BinaryOp::Add, BinaryOp::Sub])
}

fn parse_mul_div(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_unary, &[BinaryOp::Mul, BinaryOp::Div])
}

/// Parses an expression: unary and binary with proper precedence.
pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?;
    let (input, expr) = parse_pipe(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, expr))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use bigdecimal::BigDecimal;

    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::ast::UnaryOp;

    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_expr("42"), Ok(("", Expr::literal_number(42))));
        assert_eq!(
            parse_expr("3.14"),
            Ok((
                "",
                Expr::literal_number(
                    BigDecimal::from_str("3.14").expect("unable to parse 3.14 into BigDecimal")
                )
            ))
        );
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expr("foo"), Ok(("", Expr::ident("foo"))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_expr(r#""hello""#),
            Ok(("", Expr::literal_string("hello")))
        );
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_expr(":foo"), Ok(("", Expr::literal_symbol("foo"))));
    }

    #[test]
    fn test_parse_unary() {
        assert_eq!(
            parse_expr("!x"),
            Ok(("", Expr::unary_expr(UnaryOp::Not, Expr::ident("x"))))
        );
        assert_eq!(
            parse_expr("-42"),
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Minus, Expr::literal_number(42))
            ))
        );
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(
            parse_expr("1 + 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::literal_number(2)
                )
            ))
        );
        assert_eq!(
            parse_expr("10 * 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(10),
                    BinaryOp::Mul,
                    Expr::literal_number(2)
                )
            ))
        );
    }

    #[test]
    fn test_parse_precedence() {
        // * has higher precedence than +
        assert_eq!(
            parse_expr("1 + 2 * 3"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::binary_expr(
                        Expr::literal_number(2),
                        BinaryOp::Mul,
                        Expr::literal_number(3)
                    )
                )
            ))
        );
    }

    use crate::ast::Identifier;

    #[test]
    fn test_parse_function_call() {
        assert_eq!(
            parse_expr("foo()"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![]
                )
            ))
        );
        assert_eq!(
            parse_expr("bar(1)"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("bar").expect("invalid identifier"),
                    vec![Expr::literal_number(1)]
                )
            ))
        );
        assert_eq!(
            parse_expr("add(1, 2)"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("add").expect("invalid identifier"),
                    vec![Expr::literal_number(1), Expr::literal_number(2)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe() {
        assert_eq!(
            parse_expr("1 |> add(2)"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Pipe,
                    Expr::function_call("add", vec![Expr::literal_number(2)])
                )
            ))
        );
        assert_eq!(
            parse_expr("a |> b |> c"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(Expr::ident("a"), BinaryOp::Pipe, Expr::ident("b")),
                    BinaryOp::Pipe,
                    Expr::ident("c")
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe_with_function_calls() {
        assert_eq!(
            parse_expr(r#"READ("input") |> WRITE("output")"#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::function_call("READ", vec![Expr::literal_string("input")]),
                    BinaryOp::Pipe,
                    Expr::function_call("WRITE", vec![Expr::literal_string("output")])
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe_with_function_calls_and_symbols() {
        assert_eq!(
            parse_expr(r#"READ("input") |> SELECT(:id, :email, :name) |> WRITE("output")"#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(
                        Expr::function_call("READ", vec![Expr::literal_string("input")]),
                        BinaryOp::Pipe,
                        Expr::function_call(
                            "SELECT",
                            vec![
                                Expr::literal_symbol("id"),
                                Expr::literal_symbol("email"),
                                Expr::literal_symbol("name")
                            ]
                        ),
                    ),
                    BinaryOp::Pipe,
                    Expr::function_call("WRITE", vec![Expr::literal_string("output")])
                )
            ))
        );
    }

    #[test]
    fn test_parse_parenthesized() {
        assert_eq!(
            parse_expr("(1 + 2) * 3"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::parenthesized(Expr::binary_expr(
                        Expr::literal_number(1),
                        BinaryOp::Add,
                        Expr::literal_number(2)
                    )),
                    BinaryOp::Mul,
                    Expr::literal_number(3)
                )
            ))
        );
    }
}
