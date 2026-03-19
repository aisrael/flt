use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::peek;
use nom::combinator::value;
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;

use crate::ast::Keyword;

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Ensures the next character (if any) is not an identifier continuation,
/// so that e.g. "if" is recognized but "iffy" is not.
fn word_boundary(input: &str) -> IResult<&str, ()> {
    peek(alt((
        value((), nom::combinator::eof),
        value(
            (),
            take_while_m_n(1, 1, |c: char| !is_identifier_continue(c)),
        ),
    )))
    .parse(input)
}

/// Parses a reserved keyword as an expression (word-boundary aware).
pub fn parse_keyword(input: &str) -> IResult<&str, Keyword> {
    alt((
        value(Keyword::Return, terminated(tag("return"), word_boundary)),
        value(Keyword::While, terminated(tag("while"), word_boundary)),
        value(Keyword::Else, terminated(tag("else"), word_boundary)),
        value(Keyword::For, terminated(tag("for"), word_boundary)),
        value(Keyword::And, terminated(tag("and"), word_boundary)),
        value(Keyword::Not, terminated(tag("not"), word_boundary)),
        value(Keyword::If, terminated(tag("if"), word_boundary)),
        value(Keyword::In, terminated(tag("in"), word_boundary)),
        value(Keyword::Let, terminated(tag("let"), word_boundary)),
        value(Keyword::Or, terminated(tag("or"), word_boundary)),
        value(Keyword::Do, terminated(tag("do"), word_boundary)),
        value(Keyword::Fn, terminated(tag("fn"), word_boundary)),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::ast::Keyword;

    use super::*;

    #[test]
    fn test_parse_keywords() {
        assert_eq!(parse_keyword("if"), Ok(("", Keyword::If)));
        assert_eq!(parse_keyword("else"), Ok(("", Keyword::Else)));
        assert_eq!(parse_keyword("return"), Ok(("", Keyword::Return)));
        assert_eq!(parse_keyword("and"), Ok(("", Keyword::And)));
        assert_eq!(parse_keyword("or"), Ok(("", Keyword::Or)));
        assert_eq!(parse_keyword("not"), Ok(("", Keyword::Not)));
        assert_eq!(parse_keyword("for"), Ok(("", Keyword::For)));
        assert_eq!(parse_keyword("in"), Ok(("", Keyword::In)));
        assert_eq!(parse_keyword("while"), Ok(("", Keyword::While)));
        assert_eq!(parse_keyword("do"), Ok(("", Keyword::Do)));
        assert_eq!(parse_keyword("fn"), Ok(("", Keyword::Fn)));
        assert_eq!(parse_keyword("let"), Ok(("", Keyword::Let)));
    }

    #[test]
    fn test_parse_keyword_with_remainder() {
        assert_eq!(parse_keyword("if "), Ok((" ", Keyword::If)));
        assert_eq!(parse_keyword("return("), Ok(("(", Keyword::Return)));
    }

    #[test]
    fn test_keyword_word_boundary() {
        // "if" alone is keyword
        assert_eq!(parse_keyword("if"), Ok(("", Keyword::If)));
        // "iffy" should not match "if" as keyword (identifier wins later in alt)
        assert!(parse_keyword("iffy").is_err());
        // "in" alone is keyword
        assert_eq!(parse_keyword("in"), Ok(("", Keyword::In)));
        // "int" or "input" should not match "in"
        assert!(parse_keyword("int").is_err());
        assert!(parse_keyword("input").is_err());
    }
}
