use std::str::FromStr;

use bigdecimal::BigDecimal;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit0;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::sequence::tuple;
use nom::IResult;

use crate::ast::Numeric;

/// Parses a numeric: optional `+` or `-`, digits, then optionally `.` followed by any number of decimal digits.
pub fn parse_number(input: &str) -> IResult<&str, Numeric> {
    map_res(
        recognize(tuple((
            opt(alt((tag("-"), tag("+")))),
            digit1,
            opt(tuple((tag("."), digit0))),
        ))),
        |s: &str| {
            BigDecimal::from_str(s)
                .map(Numeric::new)
                .map_err(|e| format!("invalid number '{}': {}", s, e))
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use crate::ast::Numeric;

    use super::parse_number;

    fn n(s: &str) -> Numeric {
        Numeric::new(BigDecimal::from_str(s).unwrap())
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_number("42"), Ok(("", n("42"))));
        assert_eq!(parse_number("0"), Ok(("", n("0"))));
        assert_eq!(parse_number("123"), Ok(("", n("123"))));
    }

    #[test]
    fn test_parse_positive_integer() {
        assert_eq!(parse_number("+0"), Ok(("", n("0"))));
        assert_eq!(parse_number("+7"), Ok(("", n("7"))));
        assert_eq!(parse_number("+123"), Ok(("", n("123"))));
    }

    #[test]
    fn test_parse_negative_integer() {
        assert_eq!(parse_number("-7"), Ok(("", n("-7"))));
        assert_eq!(parse_number("-123"), Ok(("", n("-123"))));
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_number("3.14"), Ok(("", n("3.14"))));
        assert_eq!(parse_number("0.5"), Ok(("", n("0.5"))));
        assert_eq!(parse_number("42.0"), Ok(("", n("42.0"))));
    }

    #[test]
    fn test_parse_decimal_with_zero_fractional_digits() {
        assert_eq!(parse_number("42."), Ok(("", n("42"))));
        assert_eq!(parse_number("+3."), Ok(("", n("3"))));
        assert_eq!(parse_number("-7."), Ok(("", n("-7"))));
    }

    #[test]
    fn test_parse_positive_decimal() {
        assert_eq!(parse_number("+3.14"), Ok(("", n("3.14"))));
        assert_eq!(parse_number("+0.5"), Ok(("", n("0.5"))));
    }

    #[test]
    fn test_parse_negative_decimal() {
        assert_eq!(parse_number("-3.14"), Ok(("", n("-3.14"))));
        assert_eq!(parse_number("-0.5"), Ok(("", n("-0.5"))));
    }

    #[test]
    fn test_parse_number_with_remainder() {
        assert_eq!(parse_number("42 "), Ok((" ", n("42"))));
        assert_eq!(parse_number("3.14,"), Ok((",", n("3.14"))));
        assert_eq!(parse_number("+42 "), Ok((" ", n("42"))));
        assert_eq!(parse_number("+3.14,"), Ok((",", n("3.14"))));
    }
}
