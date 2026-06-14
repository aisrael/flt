use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::digit0;
use nom::character::complete::digit1;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

use crate::ast::Numeric;

/// Parses a numeric: optional `+` or `-`, digits (with optional `_`), then optionally `.` followed by any number of decimal digits (with optional `_`).
pub fn parse_number(input: &str) -> IResult<&str, Numeric> {
    let (input, (sign, integer, fractional)) = (
        opt(alt((tag("-"), tag("+")))),
        digits_with_underscores,
        opt(preceded(char('.'), fractional_digits)),
    )
        .parse(input)?;

    let mut number = String::new();
    if let Some(sign) = sign {
        number.push_str(sign);
    }
    number.push_str(&integer.replace('_', ""));
    if let Some(fractional) = fractional {
        number.push('.');
        number.push_str(&fractional.replace('_', ""));
    }

    match Numeric::from_str(&number) {
        Ok(value) => Ok((input, value)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Float,
        ))),
    }
}

/// Digits with optional `_` separators between digit groups (e.g. `1_000`, `14_15`).
fn digits_with_underscores(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, many0(preceded(char('_'), digit1)))).parse(input)
}

/// Fraction after `.`: either empty (`digit0`) or digits with optional `_` (e.g. `14_15`).
fn fractional_digits(input: &str) -> IResult<&str, &str> {
    alt((digits_with_underscores, digit0)).parse(input)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ast::Numeric;

    use super::parse_number;

    fn n(s: &str) -> Numeric {
        Numeric::from_str(s).unwrap()
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

    #[test]
    fn test_parse_number_with_underscores() {
        assert_eq!(parse_number("1_000"), Ok(("", n("1000"))));
        assert_eq!(parse_number("3.14_15"), Ok(("", n("3.1415"))));
        assert_eq!(parse_number("+7_000"), Ok(("", n("7000"))));
        assert_eq!(parse_number("-1_234.56"), Ok(("", n("-1234.56"))));
    }
}
