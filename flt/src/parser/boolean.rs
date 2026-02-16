use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::IResult;

/// Parses a boolean literal: `true` or `false`.
pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(false, tag("false")),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_true() {
        assert_eq!(parse_boolean("true"), Ok(("", true)));
    }

    #[test]
    fn test_parse_false() {
        assert_eq!(parse_boolean("false"), Ok(("", false)));
    }

    #[test]
    fn test_parse_boolean_with_remainder() {
        assert_eq!(parse_boolean("true "), Ok((" ", true)));
        assert_eq!(parse_boolean("false)"), Ok((")", false)));
    }
}
