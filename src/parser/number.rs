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
use nom::Parser;

use bigdecimal::num_bigint::BigInt;

use crate::ast::Literal;
use crate::ast::Numeric;
use crate::parser::IResult;
use crate::parser::ParseError;

/// Parses a numeric literal: optional `+` or `-`, digits (with optional `_`), then optionally `.` followed by any number of decimal digits (with optional `_`). Without a `.`, it's an [`Literal::Integer`], and with one, a [`Literal::Number`], even if no digits follow the `.`.
pub fn parse_number(input: &str) -> IResult<&str, Literal> {
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

    let literal = match fractional {
        None => BigInt::from_str(&number).map(Literal::Integer).ok(),
        Some(_) => Numeric::from_str(&number).map(Literal::Number).ok(),
    };
    match literal {
        Some(literal) => Ok((input, literal)),
        None => Err(nom::Err::Failure(ParseError::new(
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

    use bigdecimal::num_bigint::BigInt;

    use crate::ast::Literal;
    use crate::ast::Numeric;

    use super::parse_number;

    /// An integer literal
    fn i(s: &str) -> Literal {
        Literal::Integer(BigInt::from_str(s).unwrap())
    }

    /// A number literal
    fn n(s: &str) -> Literal {
        Literal::Number(Numeric::from_str(s).unwrap())
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_number("42"), Ok(("", i("42"))));
        assert_eq!(parse_number("0"), Ok(("", i("0"))));
        assert_eq!(parse_number("123"), Ok(("", i("123"))));
    }

    #[test]
    fn test_parse_positive_integer() {
        assert_eq!(parse_number("+0"), Ok(("", i("0"))));
        assert_eq!(parse_number("+7"), Ok(("", i("7"))));
        assert_eq!(parse_number("+123"), Ok(("", i("123"))));
    }

    #[test]
    fn test_parse_negative_integer() {
        assert_eq!(parse_number("-7"), Ok(("", i("-7"))));
        assert_eq!(parse_number("-123"), Ok(("", i("-123"))));
    }

    #[test]
    fn test_parse_integer_with_arbitrary_precision() {
        assert_eq!(
            parse_number("340282366920938463463374607431768211456"),
            Ok(("", i("340282366920938463463374607431768211456")))
        );
    }

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_number("3.14"), Ok(("", n("3.14"))));
        assert_eq!(parse_number("0.5"), Ok(("", n("0.5"))));
        assert_eq!(parse_number("42.0"), Ok(("", n("42.0"))));
    }

    #[test]
    fn test_parse_decimal_with_zero_fractional_digits_is_a_number() {
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
        assert_eq!(parse_number("42 "), Ok((" ", i("42"))));
        assert_eq!(parse_number("3.14,"), Ok((",", n("3.14"))));
        assert_eq!(parse_number("+42 "), Ok((" ", i("42"))));
        assert_eq!(parse_number("+3.14,"), Ok((",", n("3.14"))));
    }

    #[test]
    fn test_parse_number_with_underscores() {
        assert_eq!(parse_number("1_000"), Ok(("", i("1000"))));
        assert_eq!(parse_number("3.14_15"), Ok(("", n("3.1415"))));
        assert_eq!(parse_number("+7_000"), Ok(("", i("7000"))));
        assert_eq!(parse_number("-1_234.56"), Ok(("", n("-1234.56"))));
    }

    #[test]
    fn test_parse_integer_then_floor_division() {
        assert_eq!(parse_number("7//2"), Ok(("//2", i("7"))));
    }
}
