use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::success;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;
use nom::Parser;

use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Literal;

/// Parses an interpolated string: `"..."` with `{expr}` for interpolation.
/// Escapes: `\"`, `\\`, `\{` (literal `{`).
/// Returns `Expr` (either a literal string or a concat chain).
pub fn parse_interpolated_string<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Expr> {
    move |input: &'a str| {
        let (input, segments) = delimited(
            tag("\""),
            many0(alt((
                map(
                    escaped_transform(
                        is_not("\\\"{"),
                        '\\',
                        alt((
                            value("\"", tag("\"")),
                            value("\\", tag("\\")),
                            value("{", tag("{")),
                        )),
                    ),
                    |s: String| Either::Str(s),
                ),
                map(delimited(tag("{"), expr_parser, tag("}")), Either::Expr),
            ))),
            tag("\""),
        )
        .parse(input)?;

        let expr = segments_to_expr(&segments);
        Ok((input, expr))
    }
}

enum Either<L, R> {
    Str(L),
    Expr(R),
}

fn segments_to_expr(segments: &[Either<String, Expr>]) -> Expr {
    let parts: Vec<Expr> = segments
        .iter()
        .map(|s| match s {
            Either::Str(st) => Expr::Literal(Literal::string(st.clone())),
            Either::Expr(e) => e.clone(),
        })
        .collect();

    match parts.as_slice() {
        [] => Expr::Literal(Literal::string("")),
        [single] => single.clone(),
        _ => parts
            .into_iter()
            .reduce(|acc, right| Expr::binary_expr(acc, BinaryOp::Concat, right))
            .expect("reduce on 2+ elements always yields Some"),
    }
}

/// Parses a Rust-like string: `"..."` with support for escaped `\"` (yielding `"`) and `\\` (yielding `\`).
pub fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        alt((
            escaped_transform(
                is_not("\\\""),
                '\\',
                alt((value("\"", tag("\"")), value("\\", tag("\\")))),
            ),
            value(String::new(), success(())),
        )),
        tag("\""),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::parser::parse_expr;

    use super::*;

    #[test]
    fn test_parse_interpolated_string_simple() {
        let (_, expr) = parse_interpolated_string(parse_expr)(r#""Hello, {who}!""#).unwrap();
        let expected = Expr::binary_expr(
            Expr::binary_expr(
                Expr::literal_string("Hello, "),
                BinaryOp::Concat,
                Expr::ident("who"),
            ),
            BinaryOp::Concat,
            Expr::literal_string("!"),
        );
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_interpolated_string_consecutive() {
        let (_, expr) = parse_interpolated_string(parse_expr)(r#""{a}{b}""#).unwrap();
        let expected = Expr::binary_expr(Expr::ident("a"), BinaryOp::Concat, Expr::ident("b"));
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_interpolated_string_escaped_brace() {
        let (_, expr) = parse_interpolated_string(parse_expr)(r#""a \{ b""#).unwrap();
        assert_eq!(expr, Expr::literal_string("a { b"));
    }

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(parse_string(r#""hello""#), Ok(("", "hello".to_string())));
    }

    #[test]
    fn test_parse_string_with_escaped_quote() {
        assert_eq!(
            parse_string(r#""say \"hello\"""#),
            Ok(("", r#"say "hello""#.to_string()))
        );
    }

    #[test]
    fn test_parse_string_with_escaped_backslash() {
        assert_eq!(
            parse_string(r#""path\\to\\file""#),
            Ok(("", r#"path\to\file"#.to_string()))
        );
    }

    #[test]
    fn test_parse_empty_string() {
        assert_eq!(parse_string(r#""""#), Ok(("", "".to_string())));
    }
}
