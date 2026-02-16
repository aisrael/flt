use nom::bytes::complete::take_while1;
use nom::IResult;

/// Parses an identifier: one or more alphanumeric, hyphen, or underscore characters.
pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
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
    }
}
