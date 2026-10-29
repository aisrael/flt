use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::verify;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::Parser;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Statement;
use crate::parser::IResult;
use crate::Error;

use super::comment::multispace0_or_comment;
use super::expr::parse_expr;
use super::identifier::parse_identifier;
use super::keyword::parse_keyword;
use super::type_expr::parse_type;

/// Parses the name in a binding: an identifier that isn't a reserved keyword, so `let if = 1`
/// doesn't parse.
fn parse_binding_name(input: &str) -> IResult<&str, &str> {
    verify(parse_identifier, |name: &str| {
        Keyword::from_name(name).is_none()
    })
    .parse(input)
}

/// Parses a statement: a declaration, `let x = 1` or `let x: Number = 1`; an assignment,
/// `x = 1`; or an expression. Whitespace (or comments) may separate each part.
/// A statement may be followed by an optional `;`. If it ends on a newline,
/// the `;` is not required. Two statements on the same line require `;` after the first.
pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = multispace0_or_comment(input)?;
    let mut parse_let_or_assign = alt((
        map(
            (
                verify(parse_keyword, |k: &Keyword| *k == Keyword::Let),
                multispace0_or_comment,
                parse_binding_name,
                multispace0_or_comment,
                opt(terminated(
                    preceded((tag(":"), multispace0_or_comment), parse_type),
                    multispace0_or_comment,
                )),
                tag("="),
                multispace0_or_comment,
                parse_expr,
            ),
            |(_, _, name, _, r#type, _, _, expr)| Statement::Let {
                name: Identifier(name.to_string()),
                r#type,
                expr,
            },
        ),
        map(
            (
                parse_binding_name,
                multispace0_or_comment,
                tag("="),
                multispace0_or_comment,
                parse_expr,
            ),
            |(name, _, _, _, expr)| Statement::assign(name, expr),
        ),
    ));

    let (input, stmt) = if input.starts_with("let") {
        parse_let_or_assign.parse(input)?
    } else {
        alt((parse_let_or_assign, map(parse_expr, Statement::Expr))).parse(input)?
    };
    let (input, _) = multispace0_or_comment(input)?;
    let (input, _) = opt(tag(";")).parse(input)?;
    Ok((input, stmt))
}

/// Parses all of `source` into its statements, which are separated by newlines or by `;`.
///
/// Unlike [`parse_statement`], it doesn't return leftover input: anything that isn't a
/// statement, whitespace or a comment is an [`Error::ParserError`], as are two statements on
/// one line with no `;` between them. Empty source, or source with only comments, has no
/// statements.
pub fn parse_statements(source: &str) -> Result<Vec<Statement>, Error> {
    let parse_error = Error::from;
    let mut statements = Vec::new();
    let (mut rest, _) = multispace0_or_comment(source).map_err(parse_error)?;
    while !rest.is_empty() {
        let (remainder, statement) = parse_statement(rest).map_err(parse_error)?;
        // What follows the statement, up to the next one: whitespace, comments and any `;`
        let consumed = &rest[..rest.len() - remainder.len()];
        let statement_end = consumed
            .trim_end_matches(|c: char| c.is_whitespace() || c == ';')
            .len();
        let separator = &consumed[statement_end..];
        if !remainder.is_empty() && !separator.contains([';', '\n']) {
            let next = remainder.lines().next().unwrap_or_default();
            return Err(Error::ParserError(format!(
                "expected `;` or a newline before {next:?}"
            )));
        }
        statements.push(statement);
        (rest, _) = multispace0_or_comment(remainder).map_err(parse_error)?;
    }
    Ok(statements)
}

#[cfg(test)]
mod tests {
    use crate::ast::Expr;
    use crate::ast::Statement;
    use crate::ast::TypeExpr;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let (rest, stmt) = parse_statement("let x = 1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_number(1)));
    }

    #[test]
    fn test_parse_let_statement_no_spaces() {
        let (rest, stmt) = parse_statement("let x=1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_number(1)));
    }

    #[test]
    fn test_parse_let_statement_with_expr() {
        let (rest, stmt) = parse_statement("let foo = 2 + 3").unwrap();
        assert!(rest.is_empty());
        match &stmt {
            Statement::Let { name, expr, .. } => {
                assert!(*name == "foo");
                assert!(matches!(expr, Expr::BinaryExpr(_, _, _)));
            }
            _ => panic!("expected let statement"),
        }
    }

    #[test]
    fn test_parse_let_statement_fails_without_equals() {
        assert!(parse_statement("let x 1").is_err());
    }

    #[test]
    fn test_parse_assignment_without_let() {
        let (rest, stmt) = parse_statement("x = 1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::assign("x", Expr::literal_number(1)));
    }

    #[test]
    fn test_parse_typed_let_statement() {
        let expected =
            Statement::declare_typed("x", TypeExpr::name("Number"), Expr::literal_number(1));
        for input in ["let x: Number = 1", "let x:Number=1", "let x : Number = 1"] {
            assert_eq!(
                parse_statement(input),
                Ok(("", expected.clone())),
                "{input:?}"
            );
        }
    }

    #[test]
    fn test_parse_typed_let_statement_with_union() {
        let (rest, stmt) = parse_statement("let x: Number | None = None").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::declare_typed(
                "x",
                TypeExpr::Union(vec![TypeExpr::name("Number"), TypeExpr::name("None")]),
                Expr::Literal(crate::ast::Literal::None)
            )
        );
    }

    #[test]
    fn test_parse_type_only_after_let() {
        assert!(!parses_as_binding("x: Number = 1"));
        assert!(!parses_as_binding("let x: Number"));
        assert!(!parses_as_binding("let x: = 1"));
    }

    #[test]
    fn test_parse_let_statement_optional_semicolon() {
        let (rest, stmt) = parse_statement("let x = 1;").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_number(1)));
    }

    #[test]
    fn test_parse_two_statements_same_line() {
        let (rest, stmt1) = parse_statement("let x = 1; let y = 2").unwrap();
        assert_eq!(stmt1, Statement::declare("x", Expr::literal_number(1)));
        let (rest, stmt2) = parse_statement(rest.trim()).unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt2, Statement::declare("y", Expr::literal_number(2)));
    }

    #[test]
    fn test_parse_let_statement_newline_no_semicolon_required() {
        let (rest, stmt) = parse_statement("let x = 1\n").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_number(1)));
    }

    #[test]
    fn test_parse_expr_statement_number() {
        let (rest, stmt) = parse_statement("42").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::Expr(Expr::literal_number(42)));
    }

    #[test]
    fn test_parse_expr_statement_binary_with_semicolon() {
        let (rest, stmt) = parse_statement("1 + 1;").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Expr(Expr::binary_expr(
                Expr::literal_number(1),
                crate::ast::BinaryOp::Add,
                Expr::literal_number(1)
            ))
        );
    }

    /// Whether `input` parses completely as a binding.
    fn parses_as_binding(input: &str) -> bool {
        matches!(
            parse_statement(input),
            Ok((rest, Statement::Let { .. } | Statement::Assign(_, _))) if rest.trim().is_empty()
        )
    }

    #[test]
    fn test_keyword_is_not_a_binding_name() {
        for keyword in Keyword::iter() {
            assert!(
                !parses_as_binding(&format!("let {keyword} = 1")),
                "let {keyword} = 1 should not parse as a binding"
            );
            assert!(
                !parses_as_binding(&format!("{keyword} = 1")),
                "{keyword} = 1 should not parse as a binding"
            );
        }
    }

    fn let_x(n: i64) -> Statement {
        Statement::declare("x", Expr::literal_number(n))
    }

    #[test]
    fn test_parse_statements_separated_by_semicolons_or_newlines() {
        let expected = vec![let_x(1), let_x(2), Statement::Expr(Expr::ident("x"))];
        for source in [
            "let x = 1; let x = 2; x",
            "let x = 1\nlet x = 2\nx",
            "let x = 1;\n\nlet x = 2 # two\nx;",
            "  # leading comment\nlet x = 1; # one\n let x = 2\n  x\n",
        ] {
            assert_eq!(parse_statements(source).unwrap(), expected, "{source:?}");
        }
    }

    #[test]
    fn test_parse_statements_of_empty_source() {
        for source in ["", "  \n ", "# only a comment", "# one\n# two\n"] {
            assert_eq!(parse_statements(source).unwrap(), vec![], "{source:?}");
        }
    }

    #[test]
    fn test_parse_statements_needs_a_separator_on_one_line() {
        let error = parse_statements("let x = 1 let y = 2").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Parser Error: expected `;` or a newline before \"let y = 2\""
        );
    }

    #[test]
    fn test_parse_statements_fails_on_a_syntax_error() {
        for source in ["let x = 1; let = 2", "let x = 1\n)", "1 +"] {
            let result = parse_statements(source);
            assert!(
                matches!(result, Err(Error::ParserError(_))),
                "{source:?}: {result:?}"
            );
        }
    }

    #[test]
    fn test_name_starting_with_keyword_is_a_binding_name() {
        for name in ["iffy", "letter", "fnord", "do-it", "for_each"] {
            let input = format!("let {name} = 1");
            let (rest, stmt) = parse_statement(&input).unwrap();
            assert!(rest.is_empty());
            assert_eq!(stmt, Statement::declare(name, Expr::literal_number(1)));
            assert!(parses_as_binding(&format!("{name} = 1")));
        }
    }
}
