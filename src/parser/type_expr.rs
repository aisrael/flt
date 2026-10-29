use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::not;
use nom::combinator::verify;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::Parser;

use crate::ast::Keyword;
use crate::ast::TypeExpr;
use crate::parser::IResult;

use super::comment::multispace0_or_comment;
use super::identifier::parse_identifier;

/// Parses a type, as written in a declaration: type names joined by `|`, with parentheses for
/// grouping, e.g. `Number | None` or `(Number | String) | None`. A single name is a
/// [`TypeExpr::Name`], and two or more joined by `|` are a [`TypeExpr::Union`].
pub fn parse_type(input: &str) -> IResult<&str, TypeExpr> {
    let (input, first) = parse_type_atom(input)?;
    let (input, rest) = many0(preceded(
        (multispace0_or_comment, union_bar, multispace0_or_comment),
        parse_type_atom,
    ))
    .parse(input)?;
    if rest.is_empty() {
        return Ok((input, first));
    }
    let mut members = vec![first];
    members.extend(rest);
    Ok((input, TypeExpr::Union(members)))
}

/// A `|` that isn't the start of `||` or `|>`.
fn union_bar(input: &str) -> IResult<&str, &str> {
    terminated(tag("|"), not(one_of("|>"))).parse(input)
}

/// A type name, or a type in parentheses.
fn parse_type_atom(input: &str) -> IResult<&str, TypeExpr> {
    let parenthesized = delimited(
        (tag("("), multispace0_or_comment),
        parse_type,
        (multispace0_or_comment, tag(")")),
    );
    let name = map(
        verify(parse_identifier, |name: &str| {
            Keyword::from_name(name).is_none()
        }),
        TypeExpr::name,
    );
    parenthesized.or(name).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(s: &str) -> TypeExpr {
        TypeExpr::name(s)
    }

    #[test]
    fn test_parse_type_name() {
        assert_eq!(parse_type("Number"), Ok(("", name("Number"))));
        assert_eq!(parse_type("None"), Ok(("", name("None"))));
        assert_eq!(parse_type("Date = 1"), Ok((" = 1", name("Date"))));
    }

    #[test]
    fn test_parse_union() {
        let expected = TypeExpr::Union(vec![name("Number"), name("String"), name("None")]);
        assert_eq!(
            parse_type("Number | String | None"),
            Ok(("", expected.clone()))
        );
        assert_eq!(parse_type("Number|String|None"), Ok(("", expected)));
    }

    #[test]
    fn test_parse_parenthesized_type() {
        assert_eq!(parse_type("(Number)"), Ok(("", name("Number"))));
        let expected = TypeExpr::Union(vec![
            TypeExpr::Union(vec![name("Number"), name("String")]),
            name("None"),
        ]);
        assert_eq!(parse_type("( Number | String ) | None"), Ok(("", expected)));
    }

    #[test]
    fn test_parse_type_stops_before_or_and_pipe() {
        assert_eq!(
            parse_type("Number || None"),
            Ok((" || None", name("Number")))
        );
        assert_eq!(parse_type("Number |> f"), Ok((" |> f", name("Number"))));
    }

    #[test]
    fn test_parse_type_rejects_non_names() {
        for input in ["1", "", "(Number", "| Number", "if", "fn(Any)"] {
            assert!(parse_type(input).is_err(), "{input:?} should not parse");
        }
    }
}
