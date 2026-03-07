use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::recognize;
use nom::sequence::pair;
use nom::IResult;
use nom::Parser;

/// Parses an identifier: starts with a letter, followed by zero or more
/// alphanumeric, hyphen, or underscore characters.
pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while_m_n(1, 1, |c: char| c.is_alphabetic()),
        take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_identifier("foo"), Ok(("", "foo")));
        assert_eq!(parse_identifier("abc123"), Ok(("", "abc123")));
        assert_eq!(parse_identifier("abc_123"), Ok(("", "abc_123")));
        assert_eq!(parse_identifier("abc-123"), Ok(("", "abc-123")));
        assert_eq!(parse_identifier("xyz"), Ok(("", "xyz")));
        assert_eq!(parse_identifier("foo bar"), Ok((" bar", "foo")));
        assert!(parse_identifier("123abc").is_err());
        assert!(parse_identifier("_abc").is_err());
        assert!(parse_identifier("-abc").is_err());
    }
}
