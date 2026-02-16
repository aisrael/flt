use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;

use super::boolean::parse_boolean;
use super::number::parse_number;
use super::string::parse_string;
use super::symbol::parse_symbol;
use crate::ast::Literal;

/// Parses a literal: number, string, boolean, or symbol.
pub fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        map(parse_boolean, Literal::Boolean),
        map(parse_symbol, Literal::symbol),
        map(parse_string, Literal::string),
        map(parse_number, Literal::Number),
    ))(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use crate::ast::Literal;
    use crate::ast::Numeric;

    use super::parse_literal;

    fn n(s: &str) -> Numeric {
        Numeric::new(BigDecimal::from_str(s).unwrap())
    }

    #[test]
    fn test_parse_number_literal() {
        assert_eq!(parse_literal("42"), Ok(("", Literal::Number(n("42")))));
        assert_eq!(parse_literal("3.14"), Ok(("", Literal::Number(n("3.14")))));
    }

    #[test]
    fn test_parse_string_literal() {
        assert_eq!(
            parse_literal(r#""hello""#),
            Ok(("", Literal::String("hello".into())))
        );
    }

    #[test]
    fn test_parse_boolean_literal() {
        assert_eq!(parse_literal("true"), Ok(("", Literal::Boolean(true))));
        assert_eq!(parse_literal("false"), Ok(("", Literal::Boolean(false))));
    }

    #[test]
    fn test_parse_symbol_literal() {
        assert_eq!(
            parse_literal(":foo"),
            Ok(("", Literal::Symbol("foo".into())))
        );
        assert_eq!(
            parse_literal(r#":"hello world""#),
            Ok(("", Literal::Symbol("hello world".into())))
        );
    }

    #[test]
    fn test_parse_literal_with_remainder() {
        assert_eq!(parse_literal("42 "), Ok((" ", Literal::Number(n("42")))));
        assert_eq!(
            parse_literal(r#""hi" "#),
            Ok((" ", Literal::String("hi".into())))
        );
        assert_eq!(parse_literal("true)"), Ok((")", Literal::Boolean(true))));
    }
}
