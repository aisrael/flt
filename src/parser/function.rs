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
use crate::ast::FunctionCall;
use crate::ast::Identifier;
use crate::ast::KeyValue;

use super::comment::{multispace0_or_comment, multispace1_or_comment};
use super::map::parse_kv_pair;
use super::parse_identifier;

/// Internal enum used only during parsing to represent one arg (positional or key-value).
enum ParsedArg {
    Positional(Expr),
    KeyValue(KeyValue),
}

fn parse_arg<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, ParsedArg> {
    move |input: &'a str| {
        alt((
            map(parse_kv_pair(expr_parser), |(k, v)| {
                ParsedArg::KeyValue(KeyValue {
                    key: k.into_owned(),
                    value: v,
                })
            }),
            map(expr_parser, ParsedArg::Positional),
        ))
        .parse(input)
    }
}

/// Splits parsed args into positionals (all leading Positional) and keyword_args (the rest).
fn collect_args(items: Vec<ParsedArg>) -> Result<(Vec<Expr>, Vec<KeyValue>), &'static str> {
    let mut positional_args = Vec::new();
    let mut keyword_args = Vec::new();
    let mut seen_kv = false;

    for item in items {
        match item {
            ParsedArg::Positional(expr) => {
                if seen_kv {
                    return Err("positional argument after key-value pair");
                }
                positional_args.push(expr);
            }
            ParsedArg::KeyValue(kv) => {
                seen_kv = true;
                keyword_args.push(kv);
            }
        }
    }
    Ok((positional_args, keyword_args))
}

/// Parses a function call: `Identifier` `(` args `)` or `Identifier` args.
/// Parentheses are optional; without them, at least one argument is required.
/// Arguments are comma-separated expressions, with optional trailing key-value
/// pairs that are collected into a `MapLiteral` as the final argument.
pub fn parse_function_call(
    parse_expr: fn(&str) -> IResult<&str, Expr>,
) -> impl FnMut(&str) -> IResult<&str, FunctionCall> {
    move |input: &str| {
        let (input, name) =
            map(parse_identifier, |s: &str| Identifier(s.to_string())).parse(input)?;
        let (input, (positional_args, keyword_args)) = alt((
            preceded(
                multispace0_or_comment,
                delimited(
                    tag("("),
                    delimited(
                        multispace0_or_comment,
                        map_res(
                            separated_list0(
                                (multispace0_or_comment, tag(","), multispace0_or_comment),
                                parse_arg(parse_expr),
                            ),
                            collect_args,
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
                        parse_arg(parse_expr),
                    ),
                    collect_args,
                ),
            ),
        ))
        .parse(input)?;
        Ok((
            input,
            FunctionCall {
                name,
                positional_args,
                keyword_args,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use bigdecimal::BigDecimal;

    use super::parse_function_call;
    use crate::ast::Expr;
    use crate::ast::FunctionCall;
    use crate::ast::Identifier;
    use crate::ast::KeyValue;
    use crate::parser::expr::parse_expr;

    #[test]
    fn test_parse_trim() {
        assert_eq!(
            parse_function_call(parse_expr)(r#"trim("string")"#),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("trim").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_string("string")],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_floor() {
        assert_eq!(
            parse_function_call(parse_expr)("floor(3.14)"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("floor").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(
                        BigDecimal::from_str("3.14").expect("unable to parse 3.14 into BigDecimal")
                    )],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_ceil() {
        assert_eq!(
            parse_function_call(parse_expr)("ceil(3.14)"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("ceil").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(
                        BigDecimal::from_str("3.14").expect("unable to parse 3.14 into BigDecimal")
                    )],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_round() {
        assert_eq!(
            parse_function_call(parse_expr)("round(3.14, 2)"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("round").expect("invalid identifier"),
                    positional_args: vec![
                        Expr::literal_number(
                            BigDecimal::from_str("3.14")
                                .expect("unable to parse 3.14 into BigDecimal")
                        ),
                        Expr::literal_number(2),
                    ],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_single_arg() {
        assert_eq!(
            parse_function_call(parse_expr)("add 1"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("add").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(1)],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_multiple_args() {
        assert_eq!(
            parse_function_call(parse_expr)("add 1, 2"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("add").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(1), Expr::literal_number(2),],
                    keyword_args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_parse_with_trailing_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)("foo(1, optional: true)"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("foo").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(1)],
                    keyword_args: vec![KeyValue {
                        key: "optional".into(),
                        value: Expr::literal_boolean(true),
                    }],
                }
            ))
        );
    }

    #[test]
    fn test_parse_with_only_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)(r#"foo(name: "Alice", age: 30)"#),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("foo").expect("invalid identifier"),
                    positional_args: vec![],
                    keyword_args: vec![
                        KeyValue {
                            key: "name".into(),
                            value: Expr::literal_string("Alice"),
                        },
                        KeyValue {
                            key: "age".into(),
                            value: Expr::literal_number(30),
                        },
                    ],
                }
            ))
        );
    }

    #[test]
    fn test_parse_without_parens_trailing_kv_pairs() {
        assert_eq!(
            parse_function_call(parse_expr)("foo 1, optional: true"),
            Ok((
                "",
                FunctionCall {
                    name: Identifier::try_from("foo").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(1)],
                    keyword_args: vec![KeyValue {
                        key: "optional".into(),
                        value: Expr::literal_boolean(true),
                    }],
                }
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
                FunctionCall {
                    name: Identifier::try_from("foo").expect("invalid identifier"),
                    positional_args: vec![Expr::literal_number(1)],
                    keyword_args: vec![KeyValue {
                        key: "output file".into(),
                        value: Expr::literal_string("out.csv"),
                    }],
                }
            ))
        );
    }
}
