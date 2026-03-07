use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::sequence::pair;
use nom::IResult;
use nom::Parser;

use super::string::parse_string;

/// Parses a Ruby-like symbol: `:identifier` or `:"string"`.
/// - Identifier form: `:` followed by an identifier that starts with a letter,
///   then contains alphanumeric or underscore characters.
/// - String form: `:` followed by a quoted string `"..."` with escape support.
///
/// Returns `Cow<str>`: borrowed slice for identifiers (no allocation), owned for quoted strings (handles escapes).
pub fn parse_symbol(input: &str) -> IResult<&str, Cow<'_, str>> {
    let (input, _) = tag(":").parse(input)?;
    alt((
        map(parse_string, Cow::Owned),
        map(
            recognize(pair(
                take_while_m_n(1, 1, |c: char| c.is_alphabetic()),
                take_while(|c: char| c.is_alphanumeric() || c == '_'),
            )),
            Cow::Borrowed,
        ),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_parse_symbol_identifier() {
        assert_eq!(parse_symbol(":foo"), Ok(("", Cow::Borrowed("foo"))));
        assert_eq!(parse_symbol(":foo_bar"), Ok(("", Cow::Borrowed("foo_bar"))));
        assert_eq!(parse_symbol(":abc123"), Ok(("", Cow::Borrowed("abc123"))));
        assert_eq!(parse_symbol(":foo-bar"), Ok(("-bar", Cow::Borrowed("foo"))));
        assert!(parse_symbol(":123").is_err());
        assert!(parse_symbol(":_tmp").is_err());
    }

    #[test]
    fn test_parse_symbol_string() {
        assert_eq!(
            parse_symbol(r#":"hello""#),
            Ok(("", Cow::Owned("hello".into())))
        );
        assert_eq!(
            parse_symbol(r#":"hello world""#),
            Ok(("", Cow::Owned("hello world".into())))
        );
        // Empty string symbol: :""
        assert_eq!(parse_symbol(":\"\""), Ok(("", Cow::Owned(String::new()))));
    }
}
