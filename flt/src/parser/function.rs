use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

use crate::ast::Identifier;

use super::parse_identifier;

/// Parses a function call: `Identifier` `(` Expr* `)`.
/// Arguments are comma-separated. Returns `(name, args)`.
pub fn parse_function_call<F, O>(
    parse_expr: F,
) -> impl Fn(&str) -> IResult<&str, (Identifier, Vec<O>)>
where
    F: Fn(&str) -> IResult<&str, O>,
{
    move |input: &str| {
        let (input, name) = map(parse_identifier, |s: &str| Identifier(s.to_string()))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, args) = delimited(
            tag("("),
            delimited(
                multispace0,
                separated_list0(tuple((multispace0, tag(","), multispace0)), &parse_expr),
                multispace0,
            ),
            tag(")"),
        )(input)?;
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
}
