use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

use crate::ast::Expr;
use crate::ast::Identifier;
use crate::ast::KeyValue;

use super::comment::{multispace0_or_comment, multispace1_or_comment};
use super::map::parse_kv_pair;
use super::parse_identifier;

enum FnArg<'a> {
    Positional(Expr),
    KeyValue(std::borrow::Cow<'a, str>, Expr),
}

fn parse_fn_arg<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, FnArg<'a>> {
    move |input: &'a str| {
        alt((
            map(parse_kv_pair(expr_parser), |(k, v)| FnArg::KeyValue(k, v)),
            map(expr_parser, FnArg::Positional),
        ))
        .parse(input)
    }
}

/// Positional args must all come before key-value pairs.
fn collect_fn_args(items: Vec<FnArg<'_>>) -> Result<Vec<Expr>, &'static str> {
    let mut args = Vec::new();
    let mut kv_pairs: Vec<KeyValue> = Vec::new();

    for item in items {
        match item {
            FnArg::Positional(expr) => {
                if !kv_pairs.is_empty() {
                    return Err("positional argument after key-value pair");
                }
                args.push(expr);
            }
            FnArg::KeyValue(key, value) => kv_pairs.push(KeyValue {
                key: key.into_owned(),
                value,
            }),
        }
    }

    if !kv_pairs.is_empty() {
        args.push(Expr::MapLiteral(kv_pairs));
    }

    Ok(args)
}

/// Parses a function call: `Identifier` `(` args `)` or `Identifier` args.
/// Parentheses are optional; without them, at least one argument is required.
/// Arguments are comma-separated expressions, with optional trailing key-value
/// pairs that are collected into a `MapLiteral` as the final argument.
pub fn parse_function_call(
    parse_expr: fn(&str) -> IResult<&str, Expr>,
) -> impl FnMut(&str) -> IResult<&str, (Identifier, Vec<Expr>)> {
    move |input: &str| {
        let (input, name) =
            map(parse_identifier, |s: &str| Identifier(s.to_string())).parse(input)?;
        let (input, args) = alt((
            preceded(
                multispace0_or_comment,
                delimited(
                    tag("("),
                    delimited(
                        multispace0_or_comment,
                        map_res(
                            separated_list0(
                                (multispace0_or_comment, tag(","), multispace0_or_comment),
                                parse_fn_arg(parse_expr),
                            ),
                            collect_fn_args,
                        ),
                        multispace0_or_comment,
                    ),
                    tag(")"),
                ),
            ),
            preceded(
                multispace1_or_comment,
                map_res(
                    separated_list1(
                        (multispace0_or_comment, tag(","), multispace0_or_comment),
                        parse_fn_arg(parse_expr),
                    ),
                    collect_fn_args,
                ),
            ),
        ))
        .parse(input)?;
        Ok((input, (name, args)))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use bigdecimal::BigDecimal;

    use super::parse_function_call;
    use crate::ast::Expr;
    use crate::parser::expr::parse_expr;

    use crate::ast::Identifier;

    #[test]
    fn test_parse_trim() {
        assert_eq!(
            parse_function_call(parse_expr)(r#"trim("string")"#),
            Ok((
                "",
                (
                    Identifier::try_from("trim").expect("invalid identifier"),
                    vec![Expr::literal_string("string")]
                )
            ))
        );
    }

    #[test]
    fn test_parse_floor() {
        assert_eq!(
            parse_function_call(parse_expr)("floor(3.14)"),
            Ok((
                "",
                (
                    Identifier::try_from("floor").expect("invalid identifier"),
                    vec![Expr::literal_number(
                        BigDecimal::from_str("3.14").expect("unable to parse 3.14 into BigDecimal")
                    )]
                )
            ))
        );
    }

    #[test]
    fn test_parse_ceil() {
        assert_eq!(
            parse_function_call(parse_expr)("ceil(3.14)"),
            Ok((
                "",
                (
                    Identifier::try_from("ceil").expect("invalid identifier"),
                    vec![Expr::literal_number(
                        BigDecimal::from_str("3.14").expect("unable to parse 3.14 into BigDecimal")
                    )]
                )
            ))
        );
    }

    #[test]
    fn test_parse_round() {
        assert_eq!(
            parse_function_call(parse_expr)("round(3.14, 2)"),
            Ok((
                "",
                (
                    Identifier::try_from("round").expect("invalid identifier"),
                    vec![
                        Expr::literal_number(
                            BigDecimal::from_str("3.14")
                                .expect("unable to parse 3.14 into BigDecimal")
                        ),
                        Expr::literal_number(2)
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_single_arg() {
        assert_eq!(
            parse_function_call(parse_expr)("add 1"),
            Ok((
                "",
                (
                    Identifier::try_from("add").expect("invalid identifier"),
                    vec![Expr::literal_number(1)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_multiple_args() {
        assert_eq!(
            parse_function_call(parse_expr)("add 1, 2"),
            Ok((
                "",
                (
                    Identifier::try_from("add").expect("invalid identifier"),
                    vec![Expr::literal_number(1), Expr::literal_number(2)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_with_trailing_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)("foo(1, optional: true)"),
            Ok((
                "",
                (
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![
                        Expr::literal_number(1),
                        Expr::map_literal(vec![("optional", Expr::literal_boolean(true))]),
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_parse_with_only_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)(r#"foo(name: "Alice", age: 30)"#),
            Ok((
                "",
                (
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![Expr::map_literal(vec![
                        ("name", Expr::literal_string("Alice")),
                        ("age", Expr::literal_number(30)),
                    ])]
                )
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_trailing_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)("foo 1, optional: true"),
            Ok((
                "",
                (
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![
                        Expr::literal_number(1),
                        Expr::map_literal(vec![("optional", Expr::literal_boolean(true))]),
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_parse_positional_after_kv_fails() {
        assert!(parse_function_call(parse_expr)("foo(a: 1, 2)").is_err());
    }

    #[test]
    fn test_parse_with_quoted_kv_key() {
        assert_eq!(
            parse_function_call(parse_expr)(r#"foo(1, "output file": "out.csv")"#),
            Ok((
                "",
                (
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![
                        Expr::literal_number(1),
                        Expr::map_literal(vec![("output file", Expr::literal_string("out.csv"),)]),
                    ]
                )
            ))
        );
    }
}
