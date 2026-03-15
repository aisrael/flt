use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::separated_list0;
use nom::sequence::pair;
use nom::sequence::separated_pair;
use nom::IResult;
use nom::Parser;

use super::comment::multispace0_or_comment;
use super::string::parse_string;
use crate::ast::Expr;

/// Parses a bare map key: starts with a letter, followed by alphanumeric or `_`.
fn parse_bare_key(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while_m_n(1, 1, |c: char| c.is_alphabetic()),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

/// Parses a map key: bare identifier or quoted string.
fn parse_map_key(input: &str) -> IResult<&str, Cow<'_, str>> {
    alt((
        map(parse_string, Cow::Owned),
        map(parse_bare_key, Cow::Borrowed),
    ))
    .parse(input)
}

/// Parses a map literal: `{ key: value, ... }`.
///
/// Takes an expression parser as parameter to break the circular dependency
/// between the map parser and expression parser.
pub fn parse_map_literal<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Expr> {
    move |input: &'a str| {
        let (input, _) = tag("{").parse(input)?;
        let (input, _) = multispace0_or_comment(input)?;

        let (input, entries) = separated_list0(
            (multispace0_or_comment, tag(","), multispace0_or_comment),
            separated_pair(
                parse_map_key,
                (multispace0_or_comment, tag(":"), multispace0_or_comment),
                expr_parser,
            ),
        )
        .parse(input)?;

        let (input, _) = multispace0_or_comment(input)?;
        let (input, _) = opt(tag(",")).parse(input)?;
        let (input, _) = multispace0_or_comment(input)?;
        let (input, _) = tag("}").parse(input)?;

        let entries: Vec<(String, Expr)> = entries
            .into_iter()
            .map(|(k, v)| (k.into_owned(), v))
            .collect();

        Ok((input, Expr::MapLiteral(entries)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::parser::expr::parse_expr;

    fn parse_map(input: &str) -> IResult<&str, Expr> {
        parse_map_literal(parse_expr)(input)
    }

    #[test]
    fn test_parse_empty_map() {
        assert_eq!(parse_map("{}"), Ok(("", Expr::MapLiteral(vec![]))));
        assert_eq!(parse_map("{  }"), Ok(("", Expr::MapLiteral(vec![]))));
    }

    #[test]
    fn test_parse_single_bare_key_string_value() {
        assert_eq!(
            parse_map(r#"{ foo: "bar" }"#),
            Ok((
                "",
                Expr::map_literal(vec![("foo", Expr::literal_string("bar"))])
            ))
        );
    }

    #[test]
    fn test_parse_single_bare_key_number_value() {
        assert_eq!(
            parse_map("{ abc123: 456 }"),
            Ok((
                "",
                Expr::map_literal(vec![("abc123", Expr::literal_number(456))])
            ))
        );
    }

    #[test]
    fn test_parse_quoted_key() {
        assert_eq!(
            parse_map(r#"{ "spaced out": (1 + 1) }"#),
            Ok((
                "",
                Expr::map_literal(vec![(
                    "spaced out",
                    Expr::parenthesized(Expr::binary_expr(
                        Expr::literal_number(1),
                        BinaryOp::Add,
                        Expr::literal_number(1)
                    ))
                )])
            ))
        );
    }

    #[test]
    fn test_parse_multiple_entries() {
        assert_eq!(
            parse_map(r#"{ name: "Alice", age: 30 }"#),
            Ok((
                "",
                Expr::map_literal(vec![
                    ("name", Expr::literal_string("Alice")),
                    ("age", Expr::literal_number(30)),
                ])
            ))
        );
    }

    #[test]
    fn test_parse_trailing_comma() {
        assert_eq!(
            parse_map(r#"{ foo: 1, }"#),
            Ok((
                "",
                Expr::map_literal(vec![("foo", Expr::literal_number(1))])
            ))
        );
    }
}
