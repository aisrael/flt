use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::verify;
use nom::error::ErrorKind;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

use crate::ast::BinaryOp;
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

/// Reject arguments that start with unary `+` in parenless calls like `f + 1`.
///
/// Without this, expressions such as `x + 1` become ambiguous with the grammar
/// `Identifier <whitespace> args` and get parsed as a function call.
fn parse_arg_disallow_unary_plus<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, ParsedArg> {
    let mut inner = parse_arg(expr_parser);
    move |input: &'a str| {
        if input.starts_with('+') {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                ErrorKind::Tag,
            )));
        }
        inner(input)
    }
}

/// Splits parsed args into positionals (all leading Positional) and keyword_args (the rest).
/// A key holds its unquoted text, so `n:` and `"n":` are the same keyword and can't both
/// appear.
fn collect_args(items: Vec<ParsedArg>) -> Result<(Vec<Expr>, Vec<KeyValue>), &'static str> {
    let mut positional_args = Vec::new();
    let mut keyword_args: Vec<KeyValue> = Vec::new();

    for item in items {
        match item {
            ParsedArg::Positional(expr) => {
                if !keyword_args.is_empty() {
                    return Err("positional argument after key-value pair");
                }
                positional_args.push(expr);
            }
            ParsedArg::KeyValue(kv) => {
                if keyword_args.iter().any(|seen| seen.key == kv.key) {
                    return Err("duplicate keyword argument");
                }
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
                verify(
                    map_res(
                        separated_list1(
                            (multispace0_or_comment, tag(","), multispace0_or_comment),
                            parse_arg_disallow_unary_plus(parse_expr),
                        ),
                        collect_args,
                    ),
                    |(positional_args, keyword_args): &(Vec<Expr>, Vec<KeyValue>)| {
                        !positional_args
                            .iter()
                            .chain(keyword_args.iter().map(|kv| &kv.value))
                            .any(has_unparenthesized_pipe)
                    },
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

/// Whether `expr` has a pipe that isn't inside parentheses, brackets or a call.
///
/// A call without parentheses rejects such an argument, since `f 1 |> g()` would
/// otherwise mean `f(1 |> g())` rather than `f(1) |> g()`. Operators that bind tighter
/// than `|>` (`+ - <> * /`) can only hold a pipe through parentheses or string
/// interpolation, so only the looser operators are searched.
fn has_unparenthesized_pipe(expr: &Expr) -> bool {
    match expr {
        Expr::BinaryExpr(_, BinaryOp::Pipe, _) => true,
        Expr::BinaryExpr(_, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Concat, _)
        | Expr::BinaryExpr(_, BinaryOp::Mul | BinaryOp::Div, _) => false,
        Expr::BinaryExpr(left, _, right) => {
            has_unparenthesized_pipe(left) || has_unparenthesized_pipe(right)
        }
        _ => false,
    }
}

/// Parses a function call that *requires* parentheses: `Identifier` `(` args `)`.
///
/// This is useful in contexts like `if <cond> <then> else <else>`, where the
/// parenless call form `Identifier <whitespace> args` would otherwise absorb
/// the `<then>` expression and make parsing ambiguous.
pub fn parse_function_call_parens_only(
    parse_expr: fn(&str) -> IResult<&str, Expr>,
) -> impl FnMut(&str) -> IResult<&str, FunctionCall> {
    move |input: &str| {
        let (input, name) =
            map(parse_identifier, |s: &str| Identifier(s.to_string())).parse(input)?;

        let (input, (positional_args, keyword_args)) = preceded(
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
        )
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
    use std::str::FromStr;

    use super::parse_function_call;
    use crate::ast::Expr;
    use crate::ast::FunctionCall;
    use crate::ast::Identifier;
    use crate::ast::KeyValue;
    use crate::ast::Numeric;
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
                        Numeric::from_str("3.14").expect("unable to parse 3.14 into Numeric")
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
                        Numeric::from_str("3.14").expect("unable to parse 3.14 into Numeric")
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
                            Numeric::from_str("3.14").expect("unable to parse 3.14 into Numeric")
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
    fn test_parse_duplicate_keyword_fails() {
        for input in [
            "foo(a: 1, a: 2)",
            r#"foo(a: 1, "a": 2)"#,
            "foo(1, a: 1, b: 2, a: 3)",
            "foo a: 1, a: 2",
        ] {
            assert!(
                parse_function_call(parse_expr)(input).is_err(),
                "{input} should fail to parse"
            );
        }
    }

    #[test]
    fn test_parse_keywords_differ_from_a_map_argument() {
        let (_, keywords) = parse_function_call(parse_expr)("foo(k: 1)").unwrap();
        let (_, map) = parse_function_call(parse_expr)("foo({k: 1})").unwrap();
        assert_eq!(keywords.positional_args, vec![]);
        assert_eq!(keywords.keyword_args.len(), 1);
        assert_eq!(
            map.positional_args,
            vec![Expr::map_literal(vec![("k", Expr::literal_number(1))])]
        );
        assert_eq!(map.keyword_args, vec![]);
    }

    #[test]
    fn test_display_round_trips() {
        for input in ["foo(1, k: 2)", r#"foo(1, "c d": 2)"#, "foo(k: 2)", "foo()"] {
            let (_, call) = parse_function_call(parse_expr)(input).unwrap();
            assert_eq!(call.to_string(), input);
        }
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
