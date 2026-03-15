use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::multi::separated_list0;
use nom::IResult;
use nom::Parser;

use super::comment::multispace0_or_comment;
use crate::ast::Expr;

/// Parses an array literal: `[ expr, ... ]`.
///
/// Takes an expression parser as parameter to break the circular dependency
/// between the array parser and expression parser.
pub fn parse_array_literal<'a>(
    expr_parser: fn(&'a str) -> IResult<&'a str, Expr>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Expr> {
    move |input: &'a str| {
        let (input, _) = tag("[").parse(input)?;
        let (input, _) = multispace0_or_comment(input)?;

        let (input, elems) = separated_list0(
            (multispace0_or_comment, tag(","), multispace0_or_comment),
            expr_parser,
        )
        .parse(input)?;

        let (input, _) = multispace0_or_comment(input)?;
        let (input, _) = opt(tag(",")).parse(input)?;
        let (input, _) = multispace0_or_comment(input)?;
        let (input, _) = tag("]").parse(input)?;

        Ok((input, Expr::ArrayLiteral(elems)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::parser::expr::parse_expr;

    fn parse_array(input: &str) -> IResult<&str, Expr> {
        parse_array_literal(parse_expr)(input)
    }

    #[test]
    fn test_parse_empty_array() {
        assert_eq!(parse_array("[]"), Ok(("", Expr::ArrayLiteral(vec![]))));
        assert_eq!(parse_array("[  ]"), Ok(("", Expr::ArrayLiteral(vec![]))));
    }

    #[test]
    fn test_parse_single_element() {
        assert_eq!(
            parse_array("[ 42 ]"),
            Ok(("", Expr::array_literal(vec![Expr::literal_number(42)])))
        );
        assert_eq!(
            parse_array(r#"[ "hello" ]"#),
            Ok(("", Expr::array_literal(vec![Expr::literal_string("hello")])))
        );
    }

    #[test]
    fn test_parse_multiple_elements() {
        assert_eq!(
            parse_array("[ 1, 2, 3 ]"),
            Ok((
                "",
                Expr::array_literal(vec![
                    Expr::literal_number(1),
                    Expr::literal_number(2),
                    Expr::literal_number(3),
                ])
            ))
        );
    }

    #[test]
    fn test_parse_trailing_comma() {
        assert_eq!(
            parse_array("[ 1, ]"),
            Ok(("", Expr::array_literal(vec![Expr::literal_number(1)])))
        );
    }

    #[test]
    fn test_parse_nested_expr() {
        assert_eq!(
            parse_array("[ 1 + 2, foo() ]"),
            Ok((
                "",
                Expr::array_literal(vec![
                    Expr::binary_expr(
                        Expr::literal_number(1),
                        crate::ast::BinaryOp::Add,
                        Expr::literal_number(2),
                    ),
                    Expr::function_call("foo", vec![]),
                ])
            ))
        );
    }
}
