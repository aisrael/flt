use nom::bytes::complete::take_while1;
use nom::combinator::map_opt;
use nom::Parser;

use crate::ast::Keyword;
use crate::parser::IResult;

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Parses a reserved keyword as an expression (word-boundary aware). It reads a whole word, so
/// e.g. "if" is a keyword but "iffy" is not.
pub fn parse_keyword(input: &str) -> IResult<&str, Keyword> {
    map_opt(take_while1(is_identifier_continue), Keyword::from_name).parse(input)
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
