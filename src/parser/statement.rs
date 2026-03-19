use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::verify;
use nom::IResult;
use nom::Parser;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Statement;

use super::comment::multispace0_or_comment;
use super::expr::parse_expr;
use super::identifier::parse_identifier;
use super::keyword::parse_keyword;

/// Parses a let/assignment statement: optional `let` keyword, then identifier, `=`, expression,
/// with optional whitespace (or comments) between each part.
/// So both `let x = 1` and `x = 1` are valid and equivalent.
/// A statement may be followed by an optional `;`. If it ends on a newline,
/// the `;` is not required. Two statements on the same line require `;` after the first.
pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = multispace0_or_comment(input)?;
    let mut parse_let_or_assign = alt((
        map(
            (
                verify(parse_keyword, |k: &Keyword| *k == Keyword::Let),
                multispace0_or_comment,
                parse_identifier,
                multispace0_or_comment,
                tag("="),
                multispace0_or_comment,
                parse_expr,
            ),
            |(_, _, name, _, _, _, expr)| Statement::Let(Identifier(name.to_string()), expr),
        ),
        map(
            (
                parse_identifier,
                multispace0_or_comment,
                tag("="),
                multispace0_or_comment,
                parse_expr,
            ),
            |(name, _, _, _, expr)| Statement::Let(Identifier(name.to_string()), expr),
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

#[cfg(test)]
mod tests {
    use crate::ast::Expr;
    use crate::ast::Identifier;
    use crate::ast::Statement;

    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let (rest, stmt) = parse_statement("let x = 1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
    }

    #[test]
    fn test_parse_let_statement_no_spaces() {
        let (rest, stmt) = parse_statement("let x=1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
    }

    #[test]
    fn test_parse_let_statement_with_expr() {
        let (rest, stmt) = parse_statement("let foo = 2 + 3").unwrap();
        assert!(rest.is_empty());
        match &stmt {
            Statement::Let(ident, expr) => {
                assert!(*ident == "foo");
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
        assert_eq!(
            stmt,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
    }

    #[test]
    fn test_parse_let_statement_optional_semicolon() {
        let (rest, stmt) = parse_statement("let x = 1;").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
    }

    #[test]
    fn test_parse_two_statements_same_line() {
        let (rest, stmt1) = parse_statement("let x = 1; let y = 2").unwrap();
        assert_eq!(
            stmt1,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
        let (rest, stmt2) = parse_statement(rest.trim()).unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt2,
            Statement::Let(Identifier("y".to_string()), Expr::literal_number(2))
        );
    }

    #[test]
    fn test_parse_let_statement_newline_no_semicolon_required() {
        let (rest, stmt) = parse_statement("let x = 1\n").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Let(Identifier("x".to_string()), Expr::literal_number(1))
        );
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
}
