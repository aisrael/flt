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

use crate::ast::Expr;
use crate::ast::FunctionDeclaration;
use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Parameter;
use crate::ast::ParameterKind;
use crate::ast::Statement;
use crate::ast::TypeExpr;
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
            |input| parse_declaration(Keyword::Let, input),
            |(name, r#type, expr)| Statement::Let {
                name: Identifier(name.to_string()),
                r#type,
                expr,
            },
        ),
        map(
            |input| parse_declaration(Keyword::Const, input),
            |(name, r#type, expr)| Statement::Const {
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

    let (input, stmt) = if starts_with_keyword(input, Keyword::Def) || starts_with_const_def(input)
    {
        map(parse_def, Statement::Def).parse(input)?
    } else if starts_with_keyword(input, Keyword::Return) {
        parse_return(input)?
    } else if input.starts_with("let") || input.starts_with("const") {
        parse_let_or_assign.parse(input)?
    } else {
        alt((parse_let_or_assign, map(parse_expr, Statement::Expr))).parse(input)?
    };
    let (input, _) = multispace0_or_comment(input)?;
    let (input, _) = opt(tag(";")).parse(input)?;
    Ok((input, stmt))
}

/// Parses a declaration after `keyword`, `let` or `const`: a name, an optional type after a `:`,
/// and an expression.
fn parse_declaration(
    keyword: Keyword,
    input: &str,
) -> IResult<&str, (&str, Option<TypeExpr>, Expr)> {
    map(
        (
            verify(parse_keyword, |k: &Keyword| *k == keyword),
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
        |(_, _, name, _, r#type, _, _, expr)| (name, r#type, expr),
    )
    .parse(input)
}

/// Parses `return` and its expression, if any. The expression is on the same line as `return`:
/// a line break, a `;`, a comment, or a `}`, `else`, `elsif` or `end` ends a bare `return`.
fn parse_return(input: &str) -> IResult<&str, Statement> {
    let (input, _) = verify(parse_keyword, |k: &Keyword| *k == Keyword::Return).parse(input)?;
    let after_blanks = input.trim_start_matches([' ', '\t']);
    let is_bare = after_blanks.is_empty()
        || after_blanks.starts_with(['\n', '\r', ';', '#', '}'])
        || starts_with_keyword(after_blanks, Keyword::End)
        || starts_with_keyword(after_blanks, Keyword::Else)
        || starts_with_keyword(after_blanks, Keyword::Elsif);
    if is_bare {
        return Ok((input, Statement::Return(None)));
    }
    // After `return` and its blanks, an expression must follow
    let (input, expr) = cut(parse_expr).parse(after_blanks)?;
    Ok((input, Statement::return_value(expr)))
}

/// Whether `input` starts with the reserved word `keyword`, as a whole word.
fn starts_with_keyword(input: &str, keyword: Keyword) -> bool {
    matches!(parse_keyword(input), Ok((_, found)) if found == keyword)
}

/// Whether `input` starts with `const` and then `def`, which is a function declaration. `const`
/// and a name is a constant declaration.
fn starts_with_const_def(input: &str) -> bool {
    match parse_keyword(input) {
        Ok((rest, Keyword::Const)) => multispace0_or_comment(rest)
            .is_ok_and(|(rest, _)| starts_with_keyword(rest, Keyword::Def)),
        _ => false,
    }
}

/// A failure, which stops the parser from trying anything else, e.g. a `def` with its
/// parameters in the wrong order.
fn failure(input: &str) -> nom::Err<ParseError<&str>> {
    nom::Err::Failure(ParseError::new(input, nom::error::ErrorKind::Verify))
}

/// Parses a parameter: `name: Type`, `*name: Type`, `name: Type = default` or `**name: Type`.
fn parse_parameter(input: &str) -> IResult<&str, Parameter> {
    let (input, stars) = alt((tag("**"), tag("*"), tag(""))).parse(input)?;
    let (input, name) = parse_binding_name(input)?;
    let (input, _) = (multispace0_or_comment, tag(":"), multispace0_or_comment).parse(input)?;
    let (input, r#type) = parse_type(input)?;
    match stars {
        "**" => Ok((input, Parameter::rest_keywords(name, r#type))),
        "*" => Ok((input, Parameter::variadic(name, r#type))),
        _ => {
            let (input, default) = opt(preceded(
                (multispace0_or_comment, tag("="), multispace0_or_comment),
                parse_expr,
            ))
            .parse(input)?;
            Ok(match default {
                Some(default) => (input, Parameter::keyword(name, r#type, default)),
                None => (input, Parameter::positional(name, r#type)),
            })
        }
    }
}

/// Checks the parameters of a declaration: they come in the order positional, variadic,
/// keyword, rest keywords, with at most one variadic and one rest keywords parameter, and no
/// name occurs twice.
fn params_are_valid(params: &[Parameter]) -> bool {
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

/// Whether the default of a keyword parameter holds a `return`, which has no function to leave.
fn default_contains_return(param: &Parameter) -> bool {
    matches!(&param.kind, ParameterKind::Keyword(default) if default.contains_return())
}

/// Parses a function declaration, `def name(params) -> type`, a body and `end`, with an optional
/// `const` before `def`. The `-> type` is optional. The body is statements, separated by newlines
/// or by `;`.
fn parse_def(input: &str) -> IResult<&str, FunctionDeclaration> {
    let (input, is_const) = opt(terminated(
        verify(parse_keyword, |k: &Keyword| *k == Keyword::Const),
        multispace0_or_comment,
    ))
    .parse(input)?;
    let (input, _) = verify(parse_keyword, |k: &Keyword| *k == Keyword::Def).parse(input)?;
    // After `def`, nothing else can parse, so a mistake is a failure
    let (input, declaration) = cut(parse_def_rest).parse(input)?;
    Ok((
        input,
        FunctionDeclaration {
            is_const: is_const.is_some(),
            ..declaration
        },
    ))
}

fn parse_def_rest(input: &str) -> IResult<&str, FunctionDeclaration> {
    let (input, _) = multispace0_or_comment(input)?;
    let (input, name) = parse_binding_name(input)?;
    let (input, _) = (multispace0_or_comment, tag("("), multispace0_or_comment).parse(input)?;
    let (after_params, params) = separated_list0(
        (multispace0_or_comment, tag(","), multispace0_or_comment),
        parse_parameter,
    )
    .parse(input)?;
    if !params_are_valid(&params) || params.iter().any(default_contains_return) {
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
        FunctionDeclaration {
            is_const: false,
            name: Identifier(name.to_string()),
            params,
            return_type,
            body,
        },
    ))
}

/// What ends a list of statements.
#[derive(Clone, Copy)]
enum Closer {
    /// The keyword `end`, which closes the body of a `def`
    End,
    /// A `}`, which closes a brace block
    Brace,
    /// The keyword `else`, `elsif` or `end`, which ends an arm of an expanded `if`. It is not
    /// part of the arm, so the parser leaves it in the input.
    Arm,
}

impl Closer {
    /// The input after the closer, if `input` starts with it.
    fn strip(self, input: &str) -> Option<&str> {
        match self {
            Closer::End if starts_with_keyword(input, Keyword::End) => Some(&input["end".len()..]),
            Closer::Brace => input.strip_prefix('}'),
            Closer::Arm
                if [Keyword::Else, Keyword::Elsif, Keyword::End]
                    .into_iter()
                    .any(|keyword| starts_with_keyword(input, keyword)) =>
            {
                Some(input)
            }
            Closer::End | Closer::Arm => None,
        }
    }
}

/// Parses the statements of a body, up to and including the `end` that closes it. As in a
/// program, two statements on one line need a `;` between them.
fn parse_body(input: &str) -> IResult<&str, Vec<Statement>> {
    parse_statements_until(input, Closer::End)
}

/// Parses a brace block, `{`, statements and `}`, and gives its statements. As in a body, two
/// statements on one line need a `;` between them. The block can be empty. Unlike a body, a
/// mistake in a block is an error, not a failure, so the parser can try a map instead: `{ a: 1 }`.
pub fn parse_block(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, _) = tag("{").parse(input)?;
    parse_statements_until(input, Closer::Brace)
}

/// Parses the statements of an arm of an expanded `if`, up to `else`, `elsif` or `end`. The
/// keyword stays in the input. The arm can be empty. As in a body, two statements on one line
/// need a `;` between them.
pub fn parse_arm(input: &str) -> IResult<&str, Vec<Statement>> {
    parse_statements_until(input, Closer::Arm)
}

/// Parses statements up to and including `closer`.
fn parse_statements_until(input: &str, closer: Closer) -> IResult<&str, Vec<Statement>> {
    let mut statements = Vec::new();
    let mut rest = input;
    loop {
        let (before_statement, _) = multispace0_or_comment(rest)?;
        if let Some(after) = closer.strip(before_statement) {
            return Ok((after, statements));
        }
        let (remainder, statement) = parse_statement(before_statement)?;
        let consumed = &before_statement[..before_statement.len() - remainder.len()];
        let statement_end = consumed
            .trim_end_matches(|c: char| c.is_whitespace() || c == ';')
            .len();
        let separator = &consumed[statement_end..];
        let (remainder_start, _) = multispace0_or_comment(remainder)?;
        if closer.strip(remainder_start).is_none() && !separator.contains([';', '\n']) {
            return Err(match closer {
                Closer::End | Closer::Arm => failure(remainder),
                Closer::Brace => {
                    nom::Err::Error(ParseError::new(remainder, nom::error::ErrorKind::Verify))
                }
            });
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
    // At the top level, no function is around a `return`. The body of a `def` is not searched:
    // its `return` leaves that function.
    if statements.iter().any(Statement::contains_return) {
        return Err(Error::ParserError(
            "`return` is only allowed in the body of a function".to_string(),
        ));
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
            Ok((rest, Statement::Let { .. } | Statement::Const { .. } | Statement::Assign(_, _)))
                if rest.trim().is_empty()
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
                !parses_as_binding(&format!("const {keyword} = 1")),
                "const {keyword} = 1 should not parse as a binding"
            );
            assert!(
                !parses_as_binding(&format!("{keyword} = 1")),
                "{keyword} = 1 should not parse as a binding"
            );
        }
    }

    #[test]
    fn test_parse_const_statement() {
        let (rest, stmt) = parse_statement("const A = 1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::constant("A", Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_const_statement_with_expr() {
        let (rest, stmt) = parse_statement("const A = 1 + 2").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt.to_string(), "const A = 1 + 2");
    }

    #[test]
    fn test_parse_typed_const_statement() {
        let expected =
            Statement::constant_typed("A", TypeExpr::name("Number"), Expr::literal_integer(1));
        for input in [
            "const A: Number = 1",
            "const A:Number=1",
            "const A : Number = 1",
        ] {
            assert_eq!(
                parse_statement(input),
                Ok(("", expected.clone())),
                "{input:?}"
            );
        }
    }

    #[test]
    fn test_parse_typed_const_statement_with_union() {
        let (rest, stmt) = parse_statement("const A: Number | None = None").unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            stmt,
            Statement::constant_typed(
                "A",
                TypeExpr::Union(vec![TypeExpr::name("Number"), TypeExpr::name("None")]),
                Expr::Literal(crate::ast::Literal::None)
            )
        );
    }

    #[test]
    fn test_parse_const_needs_a_name_a_value_and_a_binding_name() {
        for input in [
            "const A",
            "const = 1",
            "const 1 = 2",
            "const A: = 1",
            "const A: Number",
            "const: Integer = 1",
            "let const = 1",
            "const = 1",
        ] {
            assert!(!parses_as_binding(input), "{input:?} should not parse");
        }
    }

    #[test]
    fn test_parse_const_with_comments_and_separators() {
        assert_eq!(
            parse_statement("const A # the name\n  = 1"),
            Ok(("", Statement::constant("A", Expr::literal_integer(1))))
        );
        assert_eq!(
            parse_statement("const A = 1;"),
            Ok(("", Statement::constant("A", Expr::literal_integer(1))))
        );
        assert_eq!(
            parse_statements("const A = 1; const B = 2").unwrap(),
            vec![
                Statement::constant("A", Expr::literal_integer(1)),
                Statement::constant("B", Expr::literal_integer(2)),
            ]
        );
    }

    #[test]
    fn test_a_name_that_starts_with_const_is_a_name() {
        assert_eq!(
            parse_statement("constant = 1"),
            Ok(("", Statement::assign("constant", Expr::literal_integer(1))))
        );
        assert_eq!(
            parse_statement("const-a = 1"),
            Ok(("", Statement::assign("const-a", Expr::literal_integer(1))))
        );
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
    fn test_parse_return() {
        let (rest, stmt) = parse_statement("return 1").unwrap();
        assert!(rest.is_empty());
        assert_eq!(stmt, Statement::return_value(Expr::literal_integer(1)));
    }

    #[test]
    fn test_parse_bare_return() {
        for source in [
            "return",
            "return\n1",
            "return; 1",
            "return # done",
            "return }",
            "return end",
            "return else",
            "return elsif",
        ] {
            let (_, stmt) = parse_statement(source).unwrap();
            assert_eq!(stmt, Statement::Return(None), "{source:?}");
        }
    }

    #[test]
    fn test_return_expression_is_on_the_same_line() {
        let (rest, stmt) = parse_statement("return\n1").unwrap();
        assert_eq!(stmt, Statement::Return(None));
        assert_eq!(rest.trim(), "1");
    }

    #[test]
    fn test_return_of_a_pipe() {
        let (_, stmt) = parse_statement("return 3 |> double").unwrap();
        assert_eq!(stmt.to_string(), "return 3 |> double");
    }

    #[test]
    fn test_return_in_a_function_body() {
        let statements = parse_statements("def f() -> Integer\n  return 1\nend").unwrap();
        let [Statement::Def(declaration)] = statements.as_slice() else {
            panic!("expected one def, got {statements:?}");
        };
        assert_eq!(
            declaration.body,
            vec![Statement::return_value(Expr::literal_integer(1))]
        );
    }

    #[test]
    fn test_return_in_a_block_in_a_function_body() {
        let source = "def f()\n  if true\n    return\n  end\nend";
        assert!(parse_statements(source).is_ok());
    }

    #[test]
    fn test_return_outside_a_function_is_a_parse_error() {
        for source in [
            "return",
            "return 1",
            "{ return 1 }",
            "if true\n  return 1\nend",
            "if true { return 1 }",
            "1 + { return 2 }",
            "f({ return 1 })",
            "[{ return 1 }]",
            "{ a: { return 1 } }",
            "let x = { return 1 }",
            "x = { return 1 }",
            "def f()\n  1\nend\nreturn",
        ] {
            let result = parse_statements(source);
            assert!(
                matches!(result, Err(Error::ParserError(_))),
                "{source:?}: {result:?}"
            );
        }
    }

    #[test]
    fn test_return_in_a_parameter_default_is_a_parse_error() {
        assert!(parse_statement("def f(k: Integer = { return 1 }) -> Integer k end").is_err());
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

    fn parse_const_def(source: &str) -> Result<Statement, String> {
        match parse_statement(source) {
            Ok((rest, stmt))
                if rest.trim().is_empty() && stmt.to_string().starts_with("const def ") =>
            {
                Ok(stmt)
            }
            Ok((rest, stmt)) => Err(format!("not a const def, or left {rest:?}: {stmt:?}")),
            Err(e) => Err(e.to_string()),
        }
    }

    #[test]
    fn test_parse_const_def_accepts_every_kind_of_parameter() {
        for source in [
            "const def flip_a_coin() -> Boolean\n  true\nend",
            "const def mod(n: Integer, d: Integer) -> Integer\n  n - (n // d) * d\nend",
            "const def greet(name: String, greeting: String = \"Hello\") -> String\n  greeting\nend",
            "const def log(label: String, *values: Any, level: Symbol = :info)\n  None\nend",
            "const def options(**opts: Any) -> Map\n  opts\nend",
            "const def one() 1 end",
            "const   def   spaced() 1 end",
            "const def nothing()\nend",
        ] {
            assert!(parse_const_def(source).is_ok(), "{source:?}");
        }
    }

    #[test]
    fn test_parse_const_def_display_parses_back_to_the_same_statement() {
        for source in [
            "const def flip_a_coin() -> Boolean\n  true\nend",
            "const def mod(n: Integer, d: Integer) -> Integer\n  n - (n // d) * d\nend",
            "const def log(label: String, *values: Any, level: Symbol = :info)\n  None\nend",
            "const def two() -> Integer\n  1\n  2\nend",
        ] {
            let stmt = parse_const_def(source).unwrap();
            let printed = stmt.to_string();
            assert_eq!(parse_const_def(&printed), Ok(stmt), "{printed:?}");
        }
    }

    #[test]
    fn test_a_const_def_is_not_the_same_statement_as_a_def() {
        let plain = parse_def("def one() 1 end").unwrap();
        let constant = parse_const_def("const def one() 1 end").unwrap();
        assert_ne!(plain, constant);
        assert_eq!(
            plain.to_string().replacen("def ", "const def ", 1),
            constant.to_string()
        );
    }

    #[test]
    fn test_const_def_checks_the_same_rules_as_def() {
        for source in [
            "const def f(a: Integer = 1, b: Integer) 1 end",
            "const def f(a: Integer, a: Integer) 1 end",
            "const def end() 1 end",
            "const def f(",
            "const def",
            "const def f() 1",
        ] {
            assert!(
                parse_statement(source).map_or(true, |(rest, _)| !rest.trim().is_empty()),
                "{source:?}"
            );
        }
    }

    #[test]
    fn test_const_followed_by_a_name_that_starts_with_def_is_a_constant() {
        for source in ["const defer = 1", "const default: Integer = 1"] {
            let (rest, stmt) = parse_statement(source).unwrap();
            assert!(rest.is_empty(), "{source:?}");
            assert!(matches!(stmt, Statement::Const { .. }), "{stmt:?}");
        }
    }

    #[test]
    fn test_const_def_in_a_statement_list() {
        let statements = parse_statements("const def one() 1 end\none()").unwrap();
        assert_eq!(statements.len(), 2);
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

    #[test]
    fn test_parse_arm_stops_at_else_elsif_and_end() {
        for (input, rest) in [
            ("1\nelse\n2", "else\n2"),
            ("1; 2; elsif x", "elsif x"),
            ("let a = 1\na\nend\nfoo", "end\nfoo"),
            ("1 # one\n# two\nend", "end"),
        ] {
            let (remainder, statements) = parse_arm(input).unwrap();
            assert_eq!(remainder, rest, "{input:?}");
            assert!(!statements.is_empty(), "{input:?}");
        }
    }

    #[test]
    fn test_parse_arm_can_be_empty() {
        assert_eq!(parse_arm("else"), Ok(("else", vec![])));
        assert_eq!(parse_arm("\n  # nothing\nend"), Ok(("end", vec![])));
    }

    #[test]
    fn test_parse_arm_gives_its_statements() {
        let (rest, statements) = parse_arm("let a = 1; a * 2\nelse").unwrap();
        assert_eq!(rest, "else");
        assert_eq!(statements.len(), 2);
        assert_eq!(
            statements[0],
            Statement::declare("a", Expr::literal_integer(1))
        );
    }

    #[test]
    fn test_parse_arm_needs_a_separator_between_statements() {
        assert!(matches!(parse_arm("1 2\nend"), Err(nom::Err::Failure(_))));
    }

    #[test]
    fn test_parse_arm_does_not_stop_at_a_longer_name() {
        let (rest, statements) = parse_arm("ends\nelse").unwrap();
        assert_eq!(rest, "else");
        assert_eq!(statements, vec![Statement::Expr(Expr::ident("ends"))]);
    }

    #[test]
    fn test_parse_arm_without_a_closer_fails() {
        assert!(parse_arm("1\n2").is_err());
    }

    #[test]
    fn test_parse_statements_with_an_expanded_if() {
        let statements = parse_statements("let x = true\nif x\n  1\nelsif x\n  2\nend\nx").unwrap();
        assert_eq!(statements.len(), 3);
    }
}
