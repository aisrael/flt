//! Comment parsing: `#` to end of line

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::character::complete::multispace1;
use nom::character::complete::not_line_ending;
use nom::combinator::opt;
use nom::combinator::value;
use nom::multi::many0;
use nom::multi::many1;
use nom::IResult;
use nom::Parser;

/// Parses a single comment: `#` followed by any characters until end of line.
fn parse_comment(input: &str) -> IResult<&str, ()> {
    value((), (tag("#"), not_line_ending, opt(line_ending))).parse(input)
}

/// Parses zero or more whitespace or comments.
pub fn multispace0_or_comment(input: &str) -> IResult<&str, ()> {
    value((), many0(alt((value((), multispace1), parse_comment)))).parse(input)
}

/// Parses one or more whitespace or comments.
pub fn multispace1_or_comment(input: &str) -> IResult<&str, ()> {
    value((), many1(alt((value((), multispace1), parse_comment)))).parse(input)
}
