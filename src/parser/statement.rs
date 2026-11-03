use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::cut;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::verify;
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::Parser;

use crate::ast::FunctionDecl;
use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Param;
use crate::ast::Statement;
use crate::parser::IResult;
use crate::parser::ParseError;
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

    let (input, stmt) = if starts_with_keyword(input, Keyword::Def) {
        map(parse_def, Statement::Def).parse(input)?
    } else if input.starts_with("let") {
        parse_let_or_assign.parse(input)?
    } else {
        alt((parse_let_or_assign, map(parse_expr, Statement::Expr))).parse(input)?
    };
    let (input, _) = multispace0_or_comment(input)?;
    let (input, _) = opt(tag(";")).parse(input)?;
    Ok((input, stmt))
}

/// Whether `input` starts with the reserved word `keyword`, as a whole word.
fn starts_with_keyword(input: &str, keyword: Keyword) -> bool {
    matches!(parse_keyword(input), Ok((_, found)) if found == keyword)
}

/// A failure, which stops the parser from trying anything else, e.g. a `def` with its
/// parameters in the wrong order.
fn failure(input: &str) -> nom::Err<ParseError<&str>> {
    nom::Err::Failure(ParseError::new(input, nom::error::ErrorKind::Verify))
}

/// Parses a parameter: `name: Type`, `*name: Type`, `name: Type = default` or `**name: Type`.
fn parse_param(input: &str) -> IResult<&str, Param> {
    let (input, stars) = alt((tag("**"), tag("*"), tag(""))).parse(input)?;
    let (input, name) = parse_binding_name(input)?;
    let (input, _) = (multispace0_or_comment, tag(":"), multispace0_or_comment).parse(input)?;
    let (input, r#type) = parse_type(input)?;
    match stars {
        "**" => Ok((input, Param::rest_keywords(name, r#type))),
        "*" => Ok((input, Param::variadic(name, r#type))),
        _ => {
            let (input, default) = opt(preceded(
                (multispace0_or_comment, tag("="), multispace0_or_comment),
                parse_expr,
            ))
            .parse(input)?;
            Ok(match default {
                Some(default) => (input, Param::keyword(name, r#type, default)),
                None => (input, Param::positional(name, r#type)),
            })
        }
    }
}

/// Checks the parameters of a declaration: they come in the order positional, variadic,
/// keyword, rest keywords, with at most one variadic and one rest keywords parameter, and no
/// name occurs twice.
fn params_are_valid(params: &[Param]) -> bool {
    let in_order = params.windows(2).all(|pair| {
        let (a, b) = (pair[0].order(), pair[1].order());
        // Positional and keyword parameters can repeat; the other kinds occur once
        a < b || (a == b && (a == 0 || a == 2))
    });
    let unique_names = params
        .iter()
        .enumerate()
        .all(|(i, param)| params[..i].iter().all(|other| other.name != param.name));
    in_order && unique_names
}

/// Parses a function declaration, `def name(params) -> type`, a body and `end`. The `-> type` is
/// optional. The body is statements, separated by newlines or by `;`.
fn parse_def(input: &str) -> IResult<&str, FunctionDecl> {
    let (input, _) = verify(parse_keyword, |k: &Keyword| *k == Keyword::Def).parse(input)?;
    // After `def`, nothing else can parse, so a mistake is a failure
    cut(parse_def_rest).parse(input)
}

fn parse_def_rest(input: &str) -> IResult<&str, FunctionDecl> {
    let (input, _) = multispace0_or_comment(input)?;
    let (input, name) = parse_binding_name(input)?;
    let (input, _) = (multispace0_or_comment, tag("("), multispace0_or_comment).parse(input)?;
    let (after_params, params) = separated_list0(
        (multispace0_or_comment, tag(","), multispace0_or_comment),
        parse_param,
    )
    .parse(input)?;
    if !params_are_valid(&params) {
        return Err(failure(input));
    }
    let (input, _) = (multispace0_or_comment, tag(")")).parse(after_params)?;
    let (input, return_type) = opt(preceded(
        (multispace0_or_comment, tag("->"), multispace0_or_comment),
        parse_type,
    ))
    .parse(input)?;
    let (input, body) = parse_body(input)?;
    Ok((
        input,
        FunctionDecl {
            name: Identifier(name.to_string()),
            params,
            return_type,
            body,
        },
    ))
}

/// Parses the statements of a body, up to and including the `end` that closes it. As in a
/// program, two statements on one line need a `;` between them.
fn parse_body(input: &str) -> IResult<&str, Vec<Statement>> {
    let mut statements = Vec::new();
    let mut rest = input;
    loop {
        let (before_statement, _) = multispace0_or_comment(rest)?;
        if starts_with_keyword(before_statement, Keyword::End) {
            return Ok((&before_statement["end".len()..], statements));
        }
        let (remainder, statement) = parse_statement(before_statement)?;
        let consumed = &before_statement[..before_statement.len() - remainder.len()];
        let statement_end = consumed
            .trim_end_matches(|c: char| c.is_whitespace() || c == ';')
            .len();
        let separator = &consumed[statement_end..];
        if !starts_with_keyword(remainder, Keyword::End) && !separator.contains([';', '\n']) {
            return Err(failure(remainder));
        }
        statements.push(statement);
        rest = remainder;
    }
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
        assert_eq!(stmt, Statement::declare("x", Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_let_statement_no_spaces() {
        let (rest, stmt) = parse_statement("let x=1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_integer(1)));
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
        assert_eq!(stmt, Statement::assign("x", Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_typed_let_statement() {
        let expected =
            Statement::declare_typed("x", TypeExpr::name("Number"), Expr::literal_integer(1));
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
        assert_eq!(stmt, Statement::declare("x", Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_two_statements_same_line() {
        let (rest, stmt1) = parse_statement("let x = 1; let y = 2").unwrap();
        assert_eq!(stmt1, Statement::declare("x", Expr::literal_integer(1)));
        let (rest, stmt2) = parse_statement(rest.trim()).unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt2, Statement::declare("y", Expr::literal_integer(2)));
    }

    #[test]
    fn test_parse_let_statement_newline_no_semicolon_required() {
        let (rest, stmt) = parse_statement("let x = 1\n").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::declare("x", Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_expr_statement_number() {
        let (rest, stmt) = parse_statement("42").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::Expr(Expr::literal_integer(42)));
    }

    #[test]
    fn test_parse_expr_statement_binary_with_semicolon() {
        let (rest, stmt) = parse_statement("1 + 1;").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::Expr(Expr::binary_expr(
                Expr::literal_integer(1),
                crate::ast::BinaryOp::Add,
                Expr::literal_integer(1)
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
        Statement::declare("x", Expr::literal_integer(n))
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
            assert_eq!(stmt, Statement::declare(name, Expr::literal_integer(1)));
            assert!(parses_as_binding(&format!("{name} = 1")));
        }
    }

    /// Parses `source` as one `def` statement, which must use all of it.
    fn parse_def(source: &str) -> Result<Statement, String> {
        match parse_statement(source) {
            Ok((rest, stmt)) if rest.trim().is_empty() && stmt.to_string().starts_with("def ") => {
                Ok(stmt)
            }
            Ok((rest, stmt)) => Err(format!("not a def, or left {rest:?}: {stmt:?}")),
            Err(e) => Err(e.to_string()),
        }
    }

    #[test]
    fn test_parse_def_accepts_every_kind_of_parameter() {
        for source in [
            "def flip_a_coin() -> Boolean\n  true\nend",
            "def mod(n: Integer, d: Integer) -> Integer\n  n - (n // d) * d\nend",
            "def greet(name: String, greeting: String = \"Hello\") -> String\n  greeting\nend",
            "def log(label: String, *values: Any, level: Symbol = :info)\n  None\nend",
            "def options(**opts: Any) -> Map\n  opts\nend",
            "def all(a: Integer, *b: Any, c: Integer = 1, **d: Any) -> None\n  None\nend",
            "def union(x: Number | None) -> (Number | None) x end",
            "def one() 1 end",
            "def nothing()\nend",
            "def spaced ( n : Integer ) -> Integer n end",
        ] {
            assert!(
                parse_def(source).is_ok(),
                "{source:?}: {:?}",
                parse_def(source)
            );
        }
    }

    #[test]
    fn test_parse_def_display_parses_back_to_the_same_statement() {
        for source in [
            "def flip_a_coin() -> Boolean\n  true\nend",
            "def mod(n: Integer, d: Integer) -> Integer\n  n - (n // d) * d\nend",
            "def log(label: String, *values: Any, level: Symbol = :info)\n  None\nend",
            "def options(**opts: Any) -> Map\n  opts\nend",
            "def two() -> Integer\n  1\n  2\nend",
        ] {
            let stmt = parse_def(source).unwrap();
            let printed = stmt.to_string();
            assert_eq!(parse_def(&printed), Ok(stmt), "{printed:?}");
        }
    }

    #[test]
    fn test_parse_def_rejects_parameters_in_the_wrong_order() {
        for source in [
            // A positional parameter after a keyword parameter
            "def f(k: Integer = 1, n: Integer) -> Integer n end",
            // A variadic parameter after a keyword parameter
            "def f(k: Integer = 1, *rest: Any) -> Integer k end",
            // A positional parameter after a variadic parameter
            "def f(*rest: Any, n: Integer) -> Integer n end",
            // Two variadic parameters
            "def f(*a: Any, *b: Any) -> Integer 1 end",
            // Two rest keywords parameters
            "def f(**a: Any, **b: Any) -> Integer 1 end",
            // Anything after the rest keywords parameter
            "def f(**a: Any, k: Integer = 1) -> Integer 1 end",
            "def f(n: Integer, **k: Any, j: Integer) -> Integer n end",
        ] {
            assert!(parse_def(source).is_err(), "{source:?} should not parse");
        }
    }

    #[test]
    fn test_parse_def_rejects_a_repeated_parameter_name() {
        for source in [
            "def f(n: Integer, n: Integer) -> Integer n end",
            "def f(n: Integer, *n: Any) -> Integer 1 end",
            "def f(n: Integer, n: Integer = 1) -> Integer 1 end",
            "def f(n: Integer, **n: Any) -> Integer 1 end",
        ] {
            assert!(parse_def(source).is_err(), "{source:?} should not parse");
        }
    }

    #[test]
    fn test_parse_def_rejects_malformed_declarations() {
        for source in [
            // A parameter without a type
            "def f(n) -> Integer n end",
            "def f(*rest) 1 end",
            // A variadic or rest keywords parameter with a default
            "def f(*rest: Any = 1) 1 end",
            "def f(**rest: Any = 1) 1 end",
            // No body, or no `end`
            "def f(n: Integer) -> Integer",
            "def f(n: Integer) -> Integer n",
            // No parentheses
            "def f -> Integer 1 end",
            // An arrow with no type
            "def f() -> 1 end",
            // No name
            "def (n: Integer) 1 end",
            // A trailing comma is not allowed
            "def f(n: Integer,) 1 end",
        ] {
            assert!(parse_def(source).is_err(), "{source:?} should not parse");
        }
    }

    #[test]
    fn test_parse_def_rejects_a_keyword_as_the_function_name() {
        for keyword in Keyword::iter() {
            let source = format!("def {keyword}() 1 end");
            assert!(parse_def(&source).is_err(), "{source:?} should not parse");
        }
    }

    #[test]
    fn test_parse_def_accepts_a_name_starting_with_a_keyword() {
        for name in ["iffy", "ending", "define", "default", "do-it"] {
            let source = format!("def {name}() 1 end");
            assert!(parse_def(&source).is_ok(), "{source:?}");
        }
    }

    #[test]
    fn test_def_is_not_an_identifier_or_binding_name() {
        assert!(!parses_as_binding("let def = 1"));
        assert!(!parses_as_binding("def = 1"));
        assert!(!parses_as_binding("let end = 1"));
        assert!(!parses_as_binding("end = 1"));
        // A word that only starts with `def` or `end` is still a name
        assert!(parses_as_binding("define = 1"));
        assert!(parses_as_binding("ending = 1"));
    }

    #[test]
    fn test_parse_statements_reads_a_def_then_the_next_statement() {
        let source = "def one() -> Integer\n  1\nend\none()";
        let statements = parse_statements(source).unwrap();
        assert_eq!(statements.len(), 2);
        assert!(statements[0].to_string().starts_with("def one"));
        assert_eq!(statements[1].to_string(), "one()");
    }

    #[test]
    fn test_parse_statements_reads_a_def_with_comments_and_semicolons() {
        let source = "# a function\ndef f() -> Integer # no parameters\n  1; 2 # last\nend\n";
        let statements = parse_statements(source).unwrap();
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_parse_statements_reads_two_defs() {
        let source = "def a() 1 end\ndef b() 2 end";
        assert_eq!(parse_statements(source).unwrap().len(), 2);
    }

    #[test]
    fn test_parse_statements_needs_a_separator_after_end() {
        assert!(parse_statements("def a() 1 end def b() 2 end").is_err());
    }

    #[test]
    fn test_parse_statements_fails_on_a_def_without_end() {
        for source in ["def a() 1", "def a()\n  1\n", "def a() def b() 1 end"] {
            assert!(
                matches!(parse_statements(source), Err(Error::ParserError(_))),
                "{source:?}"
            );
        }
    }
}
