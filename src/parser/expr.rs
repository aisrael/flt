use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::cut;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::verify;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;
use nom::Parser;

use super::array::parse_array_literal;
use super::comment::multispace0_or_comment;
use super::function::parse_function_call;
use super::function::parse_function_call_parens_only;
use super::identifier::parse_identifier;
use super::keyword::parse_keyword;
use super::literal::parse_literal;
use super::map::parse_map_literal;
use super::operands::parse_binary_op;
use super::operands::parse_unary_op;
use super::string::parse_interpolated_string;
use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::FunctionCall;
use crate::ast::Keyword;

/// Parses a `{ expr }` block used by control-flow expressions.
fn parse_block_expr(input: &str) -> IResult<&str, Expr> {
    delimited(
        (multispace0_or_comment, tag("{"), multispace0_or_comment),
        parse_or,
        (multispace0_or_comment, tag("}"), multispace0_or_comment),
    )
    .parse(input)
}

fn parse_if_branch(input: &str) -> IResult<&str, Expr> {
    alt((parse_block_expr, parse_or)).parse(input)
}

fn parse_if_then_branch(input: &str) -> IResult<&str, Expr> {
    verify(parse_if_branch, |e: &Expr| match e {
        Expr::Keyword(Keyword::Else) => false,
        Expr::FunctionCall(name, _) if *name == "else" => false,
        _ => true,
    })
    .parse(input)
}

fn parse_if_else_clause(input: &str) -> IResult<&str, Expr> {
    let (input, _) = verify(parse_keyword, |k: &Keyword| *k == Keyword::Else).parse(input)?;
    let (input, _) = multispace0_or_comment(input)?;
    parse_if_branch(input)
}

fn parse_if_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = verify(parse_keyword, |k: &Keyword| *k == Keyword::If).parse(input)?;
    let (input, _) = multispace0_or_comment(input)?;

    let (input, (condition, then_branch, else_branch)) = alt((
        |input| {
            let (input, condition) = cut(parse_pipe).parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, then_branch) = parse_if_then_branch.parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, else_branch) = opt(parse_if_else_clause).parse(input)?;
            Ok((input, (condition, then_branch, else_branch)))
        },
        |input| {
            let (input, condition) = cut(parse_if_condition_pipe).parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, then_branch) = parse_if_then_branch.parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, else_branch) = opt(parse_if_else_clause).parse(input)?;
            Ok((input, (condition, then_branch, else_branch)))
        },
    ))
    .parse(input)?;

    Ok((input, Expr::if_expr(condition, then_branch, else_branch)))
}

/// Parses a primary expression: literal, identifier, function call, or parenthesized expression.
fn parse_primary(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_interpolated_string(parse_or),
        map(parse_literal, Expr::Literal),
        parse_if_expr,
        map(parse_function_call(parse_or), |fc: FunctionCall| {
            let args = fc.args_as_exprs();
            Expr::FunctionCall(fc.name, args)
        }),
        map(parse_keyword, Expr::keyword),
        map(parse_identifier, Expr::ident),
        parse_array_literal(parse_or),
        parse_map_literal(parse_or),
        map(
            delimited(
                (multispace0_or_comment, tag("("), multispace0_or_comment),
                parse_or,
                (multispace0_or_comment, tag(")"), multispace0_or_comment),
            ),
            Expr::parenthesized,
        ),
    ))
    .parse(input)
}

/// Parses a primary expression used specifically for `if` conditions.
///
/// The main difference from `parse_primary` is that it disallows *parenless*
/// function calls (`Identifier <whitespace> args`), which would otherwise make
/// `if success "Ok" else ...` ambiguous.
fn parse_if_condition_primary(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_interpolated_string(parse_if_condition_or),
        map(parse_literal, Expr::Literal),
        map(
            parse_function_call_parens_only(parse_if_condition_or),
            |fc: FunctionCall| {
                let args = fc.args_as_exprs();
                Expr::FunctionCall(fc.name, args)
            },
        ),
        parse_if_expr,
        map(parse_keyword, Expr::keyword),
        map(parse_identifier, Expr::ident),
        parse_array_literal(parse_if_condition_or),
        parse_map_literal(parse_if_condition_or),
        map(
            delimited(
                (multispace0_or_comment, tag("("), multispace0_or_comment),
                parse_if_condition_or,
                (multispace0_or_comment, tag(")"), multispace0_or_comment),
            ),
            Expr::parenthesized,
        ),
    ))
    .parse(input)
}

/// Parses a unary expression for `if` conditions.
fn parse_if_condition_unary(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0_or_comment(input)?;
    alt((
        map(
            (parse_unary_op, parse_if_condition_unary_tight),
            |(op, e)| Expr::unary_expr(op, e),
        ),
        parse_if_condition_primary,
    ))
    .parse(input)
}

/// Parses a unary expression for `if` conditions without whitespace between unary
/// operators and the expression that follows.
fn parse_if_condition_unary_tight(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            (parse_unary_op, parse_if_condition_unary_tight),
            |(op, e)| Expr::unary_expr(op, e),
        ),
        parse_if_condition_primary,
    ))
    .parse(input)
}

fn parse_if_condition_pipe(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_or, &[BinaryOp::Pipe])
}

fn parse_if_condition_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_and, &[BinaryOp::Or])
}

fn parse_if_condition_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_xor, &[BinaryOp::And])
}

fn parse_if_condition_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_bit_or, &[BinaryOp::Xor])
}

fn parse_if_condition_bit_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_bit_xor, &[BinaryOp::BitOr])
}

fn parse_if_condition_bit_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_bit_and, &[BinaryOp::BitXor])
}

fn parse_if_condition_bit_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_if_condition_add_sub_concat,
        &[BinaryOp::BitAnd],
    )
}

fn parse_if_condition_add_sub_concat(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_if_condition_mul_div,
        &[BinaryOp::Add, BinaryOp::Sub, BinaryOp::Concat],
    )
}

fn parse_if_condition_mul_div(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_if_condition_unary,
        &[
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::Eq,
            BinaryOp::Ne,
            BinaryOp::Lt,
            BinaryOp::Gt,
            BinaryOp::Lte,
            BinaryOp::Gte,
        ],
    )
}

/// Parses a unary expression: optionally prefixed with `!`, `+`, or `-`.
///
/// Note: unary operators must be immediately adjacent to their operand.
/// e.g. `!x` and `+1` are valid, but `! x` / `+ 1` are not.
fn parse_unary(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0_or_comment(input)?;
    alt((
        map((parse_unary_op, parse_unary_tight), |(op, e)| {
            Expr::unary_expr(op, e)
        }),
        parse_primary,
    ))
    .parse(input)
}

/// Parses a unary expression without allowing whitespace/comments between a unary
/// operator and the expression that follows.
fn parse_unary_tight(input: &str) -> IResult<&str, Expr> {
    alt((
        map((parse_unary_op, parse_unary_tight), |(op, e)| {
            Expr::unary_expr(op, e)
        }),
        parse_primary,
    ))
    .parse(input)
}

/// Parses binary expressions: `Expr` then `BinaryOp` then `Expr`, with left-associative folding.
/// Precedence (lowest to highest): ||, &&, ^^, |, ^, &, +/-/<> (add/sub/concat), *, /, ==, !=, <, >, <=, >=
/// `next` parses the higher-precedence operand; `allowed` restricts which operators this level accepts.
fn parse_binary_level<'a>(
    input: &'a str,
    next: fn(&str) -> IResult<&str, Expr>,
    allowed: &[BinaryOp],
) -> IResult<&'a str, Expr> {
    let parse_expr_binary_op_expr = (
        next,
        many0((
            multispace0_or_comment,
            verify(parse_binary_op, |o: &BinaryOp| allowed.contains(o)),
            multispace0_or_comment,
            next,
        )),
    );
    map(
        parse_expr_binary_op_expr,
        |(left, pairs): (Expr, Vec<(_, BinaryOp, _, Expr)>)| {
            pairs.into_iter().fold(left, |acc, (_, op, _, right)| {
                Expr::binary_expr(acc, op, right)
            })
        },
    )
    .parse(input)
}

fn parse_pipe(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_or, &[BinaryOp::Pipe])
}

fn parse_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_and, &[BinaryOp::Or])
}

fn parse_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_xor, &[BinaryOp::And])
}

fn parse_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_or, &[BinaryOp::Xor])
}

fn parse_bit_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_xor, &[BinaryOp::BitOr])
}

fn parse_bit_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_and, &[BinaryOp::BitXor])
}

fn parse_bit_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_add_sub_concat, &[BinaryOp::BitAnd])
}

fn parse_add_sub_concat(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_mul_div,
        &[BinaryOp::Add, BinaryOp::Sub, BinaryOp::Concat],
    )
}

fn parse_mul_div(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_unary,
        &[
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::Eq,
            BinaryOp::Ne,
            BinaryOp::Lt,
            BinaryOp::Gt,
            BinaryOp::Lte,
            BinaryOp::Gte,
        ],
    )
}

/// Parses an expression: unary and binary with proper precedence.
pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0_or_comment(input)?;
    let (input, expr) = parse_pipe(input)?;
    let (input, _) = multispace0_or_comment(input)?;
    Ok((input, expr))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::ast::Numeric;
    use crate::ast::UnaryOp;

    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_expr("42"), Ok(("", Expr::literal_number(42))));
        assert_eq!(
            parse_expr("3.14"),
            Ok((
                "",
                Expr::literal_number(
                    Numeric::from_str("3.14").expect("unable to parse 3.14 into Numeric")
                )
            ))
        );
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_expr("foo"), Ok(("", Expr::ident("foo"))));
        assert_eq!(parse_expr("foo-1"), Ok(("", Expr::ident("foo-1"))));
        assert_eq!(parse_expr("123abc"), Ok(("abc", Expr::literal_number(123))));
        assert!(parse_expr("_foo").is_err());
    }

    #[test]
    fn test_parse_keyword() {
        use crate::ast::Keyword;

        assert!(parse_expr("if").is_err());
        assert_eq!(parse_expr("else"), Ok(("", Expr::keyword(Keyword::Else))));
        assert_eq!(
            parse_expr("return"),
            Ok(("", Expr::keyword(Keyword::Return)))
        );
        assert_eq!(parse_expr("and"), Ok(("", Expr::keyword(Keyword::And))));
        assert_eq!(parse_expr("or"), Ok(("", Expr::keyword(Keyword::Or))));
        assert_eq!(parse_expr("not"), Ok(("", Expr::keyword(Keyword::Not))));
        assert_eq!(parse_expr("for"), Ok(("", Expr::keyword(Keyword::For))));
        assert_eq!(parse_expr("in"), Ok(("", Expr::keyword(Keyword::In))));
        assert_eq!(parse_expr("while"), Ok(("", Expr::keyword(Keyword::While))));
        assert_eq!(parse_expr("do"), Ok(("", Expr::keyword(Keyword::Do))));
        assert_eq!(parse_expr("fn"), Ok(("", Expr::keyword(Keyword::Fn))));
        assert_eq!(parse_expr("let"), Ok(("", Expr::keyword(Keyword::Let))));
        // Keywords are not identifiers: "iffy" parses as ident, not "if" + "fy"
        assert_eq!(parse_expr("iffy"), Ok(("", Expr::ident("iffy"))));
    }

    #[test]
    fn test_parse_if_expr_block_and_optional_else() {
        assert_eq!(
            parse_expr("if true { 1 } else { 2 }"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(true),
                    Expr::literal_number(1),
                    Some(Expr::literal_number(2))
                )
            ))
        );

        assert_eq!(
            parse_expr("if false { do() }"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(false),
                    Expr::function_call("do", vec![]),
                    None
                )
            ))
        );
    }

    #[test]
    fn test_parse_if_expr_expression_branches() {
        assert_eq!(
            parse_expr("if success \"Ok\" else \":(\""),
            Ok((
                "",
                Expr::if_expr(
                    Expr::ident("success"),
                    Expr::literal_string("Ok"),
                    Some(Expr::literal_string(":("))
                )
            ))
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_expr(r#""hello""#),
            Ok(("", Expr::literal_string("hello")))
        );
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_expr(":foo"), Ok(("", Expr::literal_symbol("foo"))));
        assert_eq!(
            parse_expr(":foo_bar"),
            Ok(("", Expr::literal_symbol("foo_bar")))
        );
        assert_eq!(
            parse_expr(r#":"hello world""#),
            Ok(("", Expr::literal_symbol("hello world")))
        );
        assert_eq!(
            parse_expr(":foo-bar"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_symbol("foo"),
                    BinaryOp::Sub,
                    Expr::ident("bar")
                )
            ))
        );
        assert!(parse_expr(":123").is_err());
        assert!(parse_expr(":_tmp").is_err());
    }

    #[test]
    fn test_parse_unary() {
        assert_eq!(
            parse_expr("!x"),
            Ok(("", Expr::unary_expr(UnaryOp::Not, Expr::ident("x"))))
        );
        assert_eq!(
            parse_expr("+1"),
            Ok(("", Expr::unary_expr(UnaryOp::Plus, Expr::literal_number(1))))
        );
        assert!(parse_expr("! x").is_err());
        assert!(parse_expr("+ 1").is_err());
        assert!(parse_expr("- 42").is_err());
        assert_eq!(
            parse_expr("-42"),
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Minus, Expr::literal_number(42))
            ))
        );
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(
            parse_expr("1 + 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::literal_number(2)
                )
            ))
        );
        assert_eq!(
            parse_expr("10 * 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(10),
                    BinaryOp::Mul,
                    Expr::literal_number(2)
                )
            ))
        );
    }

    #[test]
    fn test_parse_binary_add_spacing_variants() {
        let expected = Expr::binary_expr(Expr::ident("x"), BinaryOp::Add, Expr::literal_number(1));
        assert_eq!(parse_expr("x + 1"), Ok(("", expected.clone())));
        assert_eq!(parse_expr("x+1"), Ok(("", expected.clone())));
        assert_eq!(parse_expr("x +1"), Ok(("", expected.clone())));
        assert_eq!(parse_expr("x+ 1"), Ok(("", expected)));
    }

    #[test]
    fn test_parse_precedence() {
        // * has higher precedence than +
        assert_eq!(
            parse_expr("1 + 2 * 3"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::binary_expr(
                        Expr::literal_number(2),
                        BinaryOp::Mul,
                        Expr::literal_number(3)
                    )
                )
            ))
        );
    }

    use crate::ast::Identifier;

    #[test]
    fn test_parse_function_call() {
        assert_eq!(
            parse_expr("foo()"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![]
                )
            ))
        );
        assert_eq!(
            parse_expr("bar(1)"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("bar").expect("invalid identifier"),
                    vec![Expr::literal_number(1)]
                )
            ))
        );
        assert_eq!(
            parse_expr("add(1, 2)"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("add").expect("invalid identifier"),
                    vec![Expr::literal_number(1), Expr::literal_number(2)]
                )
            ))
        );
        assert_eq!(
            parse_expr("add 1, 2"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("add").expect("invalid identifier"),
                    vec![Expr::literal_number(1), Expr::literal_number(2)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe() {
        assert_eq!(
            parse_expr("1 |> add(2)"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Pipe,
                    Expr::function_call("add", vec![Expr::literal_number(2)])
                )
            ))
        );
        assert_eq!(
            parse_expr("1 |> add 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Pipe,
                    Expr::function_call("add", vec![Expr::literal_number(2)])
                )
            ))
        );
        assert_eq!(
            parse_expr("a |> b |> c"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(Expr::ident("a"), BinaryOp::Pipe, Expr::ident("b")),
                    BinaryOp::Pipe,
                    Expr::ident("c")
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe_with_function_calls() {
        assert_eq!(
            parse_expr(r#"READ("input") |> WRITE("output")"#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::function_call("READ", vec![Expr::literal_string("input")]),
                    BinaryOp::Pipe,
                    Expr::function_call("WRITE", vec![Expr::literal_string("output")])
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe_with_function_calls_and_symbols() {
        assert_eq!(
            parse_expr(r#"READ("input") |> SELECT(:id, :email, :name) |> WRITE("output")"#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(
                        Expr::function_call("READ", vec![Expr::literal_string("input")]),
                        BinaryOp::Pipe,
                        Expr::function_call(
                            "SELECT",
                            vec![
                                Expr::literal_symbol("id"),
                                Expr::literal_symbol("email"),
                                Expr::literal_symbol("name")
                            ]
                        ),
                    ),
                    BinaryOp::Pipe,
                    Expr::function_call("WRITE", vec![Expr::literal_string("output")])
                )
            ))
        );
    }

    #[test]
    fn test_parse_string_interpolation() {
        assert_eq!(
            parse_expr(r#""Hello, {who}!""#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(
                        Expr::literal_string("Hello, "),
                        BinaryOp::Concat,
                        Expr::ident("who")
                    ),
                    BinaryOp::Concat,
                    Expr::literal_string("!")
                )
            ))
        );
        assert_eq!(
            parse_expr(r#""Answer: {1 + 2}""#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_string("Answer: "),
                    BinaryOp::Concat,
                    Expr::binary_expr(
                        Expr::literal_number(1),
                        BinaryOp::Add,
                        Expr::literal_number(2)
                    )
                )
            ))
        );
    }

    #[test]
    fn test_parse_string_concat() {
        assert_eq!(
            parse_expr(r#""foo" <> "bar""#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_string("foo"),
                    BinaryOp::Concat,
                    Expr::literal_string("bar")
                )
            ))
        );
        assert_eq!(
            parse_expr(r#""hello" <> " " <> "world""#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::binary_expr(
                        Expr::literal_string("hello"),
                        BinaryOp::Concat,
                        Expr::literal_string(" ")
                    ),
                    BinaryOp::Concat,
                    Expr::literal_string("world")
                )
            ))
        );
    }

    #[test]
    fn test_parse_comments() {
        assert_eq!(
            parse_expr("42 # comment"),
            Ok(("", Expr::literal_number(42)))
        );
        assert_eq!(
            parse_expr("# leading comment\n42"),
            Ok(("", Expr::literal_number(42)))
        );
        assert_eq!(
            parse_expr("1 # add these\n+ 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::literal_number(2)
                )
            ))
        );
        assert_eq!(
            parse_expr("add(1, # first\n 2)"),
            Ok((
                "",
                Expr::function_call(
                    "add",
                    vec![Expr::literal_number(1), Expr::literal_number(2)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_parenthesized() {
        assert_eq!(
            parse_expr("(1 + 2) * 3"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::parenthesized(Expr::binary_expr(
                        Expr::literal_number(1),
                        BinaryOp::Add,
                        Expr::literal_number(2)
                    )),
                    BinaryOp::Mul,
                    Expr::literal_number(3)
                )
            ))
        );
    }

    #[test]
    fn test_parse_function_call_with_kv_pairs() {
        assert_eq!(
            parse_expr("foo(1, optional: true)"),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("foo").expect("invalid identifier"),
                    vec![
                        Expr::literal_number(1),
                        Expr::map_literal(vec![("optional", Expr::literal_boolean(true))]),
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_parse_function_call_kv_only() {
        assert_eq!(
            parse_expr(r#"config(format: "csv", header: true)"#),
            Ok((
                "",
                Expr::FunctionCall(
                    Identifier::try_from("config").expect("invalid identifier"),
                    vec![Expr::map_literal(vec![
                        ("format", Expr::literal_string("csv")),
                        ("header", Expr::literal_boolean(true)),
                    ])]
                )
            ))
        );
    }

    #[test]
    fn test_parse_pipe_with_kv_pairs() {
        assert_eq!(
            parse_expr(r#"READ("input") |> WRITE("output", format: "csv")"#),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::function_call("READ", vec![Expr::literal_string("input")]),
                    BinaryOp::Pipe,
                    Expr::FunctionCall(
                        Identifier::try_from("WRITE").expect("invalid identifier"),
                        vec![
                            Expr::literal_string("output"),
                            Expr::map_literal(vec![("format", Expr::literal_string("csv"))]),
                        ]
                    )
                )
            ))
        );
    }
}
