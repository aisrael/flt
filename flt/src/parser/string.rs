use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::combinator::success;
use nom::combinator::value;
use nom::sequence::delimited;
use nom::IResult;

/// Parses a Rust-like string: `"..."` with support for escaped `\"` (yielding `"`) and `\\` (yielding `\`).
pub fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        alt((
            escaped_transform(
                is_not("\\\""),
                '\\',
                alt((value("\"", tag("\"")), value("\\", tag("\\")))),
            ),
            value(String::new(), success(())),
        )),
        tag("\""),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(parse_string(r#""hello""#), Ok(("", "hello".to_string())));
    }

    #[test]
    fn test_parse_string_with_escaped_quote() {
        assert_eq!(
            parse_string(r#""say \"hello\"""#),
            Ok(("", r#"say "hello""#.to_string()))
        );
    }

    #[test]
    fn test_parse_string_with_escaped_backslash() {
        assert_eq!(
            parse_string(r#""path\\to\\file""#),
            Ok(("", r#"path\to\file"#.to_string()))
        );
    }

    #[test]
    fn test_parse_empty_string() {
        assert_eq!(parse_string(r#""""#), Ok(("", "".to_string())));
    }
}
