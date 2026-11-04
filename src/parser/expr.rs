use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::cut;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::verify;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
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
use super::statement::parse_block;
use super::string::parse_interpolated_string;
use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Keyword;
use crate::parser::IResult;

/// Parses a brace block: `{`, statements and `}`. See [`parse_block`].
fn parse_block_expr(input: &str) -> IResult<&str, Expr> {
    map(parse_block, Expr::block).parse(input)
}

/// Parses an arm of an `if`: a brace block, or an expression. A pair of braces is an empty
/// block here, but an empty map anywhere else. A `{` and a key and a `:` is a map: the block
/// fails on it, so the expression parser gets it.
fn parse_if_branch(input: &str) -> IResult<&str, Expr> {
    let (input, branch) = alt((parse_block_expr, parse_or)).parse(input)?;
    let (input, _) = multispace0_or_comment(input)?;
    Ok((input, branch))
}

fn parse_if_then_branch(input: &str) -> IResult<&str, Expr> {
    verify(parse_if_branch, |e: &Expr| match e {
        Expr::Keyword(Keyword::Else) => false,
        Expr::FunctionCall(call) if call.name == "else" => false,
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
            let (input, condition) = cut(parse_or).parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, then_branch) = parse_if_then_branch.parse(input)?;
            let (input, _) = multispace0_or_comment(input)?;
            let (input, else_branch) = opt(parse_if_else_clause).parse(input)?;
            Ok((input, (condition, then_branch, else_branch)))
        },
        |input| {
            let (input, condition) = cut(parse_if_condition_or).parse(input)?;
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
        map(parse_function_call(parse_or), Expr::FunctionCall),
        map(parse_keyword, Expr::keyword),
        map(parse_identifier, Expr::ident),
        parse_array_literal(parse_or),
        parse_map_literal(parse_or),
        parse_block_expr,
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

/// Parses zero or more `.field` postfix suffixes onto a primary expression,
/// left-folding them into nested `Expr::FieldAccess` nodes (e.g. `u.foo.bar`
/// becomes `FieldAccess(FieldAccess(u, "foo"), "bar")`).
///
/// Note: `.` binds tightly — no whitespace/comments are allowed between the
/// primary expression, the `.`, and the field name (mirrors the unary-operator
/// "tight" convention; see `parse_unary_tight`).
fn parse_postfix(input: &str) -> IResult<&str, Expr> {
    let (input, base) = parse_primary(input)?;
    let (input, fields) = many0(preceded(tag("."), parse_identifier)).parse(input)?;
    Ok((input, fields.into_iter().fold(base, Expr::field_access)))
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
            Expr::FunctionCall,
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

/// Parses zero or more `.field` postfix suffixes for `if`-condition expressions.
/// See `parse_postfix` for the general-grammar equivalent.
fn parse_if_condition_postfix(input: &str) -> IResult<&str, Expr> {
    let (input, base) = parse_if_condition_primary(input)?;
    let (input, fields) = many0(preceded(tag("."), parse_identifier)).parse(input)?;
    Ok((input, fields.into_iter().fold(base, Expr::field_access)))
}

/// Parses a unary expression for `if` conditions.
fn parse_if_condition_unary(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0_or_comment(input)?;
    alt((
        map(
            (parse_unary_op, parse_if_condition_unary_tight),
            |(op, e)| Expr::unary_expr(op, e),
        ),
        parse_if_condition_postfix,
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
        parse_if_condition_postfix,
    ))
    .parse(input)
}

fn parse_if_condition_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_and, &[BinaryOp::Or])
}

fn parse_if_condition_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_xor, &[BinaryOp::And])
}

fn parse_if_condition_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_equality, &[BinaryOp::Xor])
}

fn parse_if_condition_equality(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_if_condition_relational,
        &[BinaryOp::Eq, BinaryOp::Ne],
    )
}

fn parse_if_condition_relational(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_if_condition_bit_or,
        &[BinaryOp::Lt, BinaryOp::Gt, BinaryOp::Lte, BinaryOp::Gte],
    )
}

fn parse_if_condition_bit_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_bit_xor, &[BinaryOp::BitOr])
}

fn parse_if_condition_bit_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_bit_and, &[BinaryOp::BitXor])
}

fn parse_if_condition_bit_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_pipe, &[BinaryOp::BitAnd])
}

fn parse_if_condition_pipe(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_if_condition_add_sub_concat, &[BinaryOp::Pipe])
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
        &[BinaryOp::Mul, BinaryOp::Div, BinaryOp::FloorDiv],
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
        parse_postfix,
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
        parse_postfix,
    ))
    .parse(input)
}

/// Parses binary expressions: `Expr` then `BinaryOp` then `Expr`, with left-associative folding.
/// Precedence (lowest to highest): ||, &&, ^^, ==/!=, </>/<=/>=, |, ^, &, |>, +/-/<> (add/sub/concat), *, /
/// `next` parses the higher-precedence operand; `allowed` restricts which operators this level accepts.
fn parse_binary_level<'a>(
    input: &'a str,
    next: fn(&str) -> IResult<&str, Expr>,
    allowed: &[BinaryOp],
) -> IResult<&'a str, Expr> {
    parse_binary_level_split(input, next, next, allowed)
}

/// Like `parse_binary_level`, but parses the first operand with `first` and every operand
/// after an operator with `rest`.
fn parse_binary_level_split<'a>(
    input: &'a str,
    first: fn(&str) -> IResult<&str, Expr>,
    rest: fn(&str) -> IResult<&str, Expr>,
    allowed: &[BinaryOp],
) -> IResult<&'a str, Expr> {
    let parse_expr_binary_op_expr = (
        first,
        many0((
            multispace0_or_comment,
            verify(parse_binary_op, |o: &BinaryOp| allowed.contains(o)),
            multispace0_or_comment,
            rest,
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

fn parse_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_and, &[BinaryOp::Or])
}

fn parse_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_xor, &[BinaryOp::And])
}

fn parse_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_equality, &[BinaryOp::Xor])
}

fn parse_equality(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_relational, &[BinaryOp::Eq, BinaryOp::Ne])
}

fn parse_relational(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(
        input,
        parse_bit_or,
        &[BinaryOp::Lt, BinaryOp::Gt, BinaryOp::Lte, BinaryOp::Gte],
    )
}

fn parse_bit_or(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_xor, &[BinaryOp::BitOr])
}

fn parse_bit_xor(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_bit_and, &[BinaryOp::BitXor])
}

fn parse_bit_and(input: &str) -> IResult<&str, Expr> {
    parse_binary_level(input, parse_pipe, &[BinaryOp::BitAnd])
}

/// `|>` sits directly below `+`, `-` and `<>`, as in Elixir, so `x |> f == 3` is
/// `(x |> f) == 3` and `1 + 1 |> f` is `(1 + 1) |> f`.
///
/// The right side of `|>` is parsed like an `if` condition, so a call there needs
/// parentheses: `x |> f 2` parses as `x |> f`, leaving `2` unparsed.
fn parse_pipe(input: &str) -> IResult<&str, Expr> {
    parse_binary_level_split(
        input,
        parse_add_sub_concat,
        parse_if_condition_add_sub_concat,
        &[BinaryOp::Pipe],
    )
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
        &[BinaryOp::Mul, BinaryOp::Div, BinaryOp::FloorDiv],
    )
}

/// Parses an expression: unary and binary with proper precedence.
pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0_or_comment(input)?;
    let (input, expr) = parse_or(input)?;
    let (input, _) = multispace0_or_comment(input)?;
    Ok((input, expr))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::ast::Numeric;
    use crate::ast::Statement;
    use crate::ast::UnaryOp;

    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_expr("42"), Ok(("", Expr::literal_integer(42))));
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
        assert_eq!(
            parse_expr("123abc"),
            Ok(("abc", Expr::literal_integer(123)))
        );
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
                    Expr::block(vec![Statement::Expr(Expr::literal_integer(1))]),
                    Some(Expr::block(vec![Statement::Expr(Expr::literal_integer(2))]))
                )
            ))
        );

        assert_eq!(
            parse_expr("if false { do() }"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(false),
                    Expr::block(vec![Statement::Expr(Expr::function_call("do", vec![]))]),
                    None
                )
            ))
        );
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(
            parse_expr("{ 1 }"),
            Ok((
                "",
                Expr::block(vec![Statement::Expr(Expr::literal_integer(1))])
            ))
        );
        assert_eq!(
            parse_expr("{ a }"),
            Ok(("", Expr::block(vec![Statement::Expr(Expr::ident("a"))])))
        );
    }

    #[test]
    fn test_parse_block_with_statements_on_one_line() {
        let (rest, block) = parse_expr("{ let a = 1; a * 2 }").unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.to_string(), "{ let a = 1; a * 2 }");
        let Expr::Block(statements) = block else {
            panic!("expected a block");
        };
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn test_parse_block_with_statements_on_separate_lines() {
        let (rest, block) = parse_expr("{\n  let a = 1\n  a + 1\n}").unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.to_string(), "{ let a = 1; a + 1 }");
    }

    #[test]
    fn test_parse_block_with_comments() {
        let (rest, block) = parse_expr("{ # first\n  1 # one\n}").unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.to_string(), "{ 1 }");
    }

    #[test]
    fn test_parse_block_needs_a_separator_between_statements_on_one_line() {
        assert!(parse_expr("{ let a = 1 a }").is_err());
    }

    #[test]
    fn test_parse_nested_blocks() {
        let (rest, block) = parse_expr("{ { 1 } }").unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.to_string(), "{ { 1 } }");
    }

    #[test]
    fn test_parse_block_as_an_operand() {
        let (rest, expr) = parse_expr("{ 1 } + { 2 }").unwrap();
        assert_eq!(rest, "");
        assert_eq!(expr.to_string(), "{ 1 } + { 2 }");
    }

    #[test]
    fn test_parse_block_as_an_argument() {
        let (rest, expr) = parse_expr("typeof({ 1 })").unwrap();
        assert_eq!(rest, "");
        assert_eq!(expr.to_string(), "typeof({ 1 })");
    }

    #[test]
    fn test_parse_unclosed_block_fails() {
        assert!(parse_expr("{ 1").is_err());
        assert!(parse_expr("{ let a = 1;").is_err());
        assert!(parse_expr("{ { 1 }").is_err());
    }

    #[test]
    fn test_parse_key_and_colon_starts_a_map() {
        assert_eq!(
            parse_expr("{ a: 1 }"),
            Ok(("", Expr::map_literal(vec![("a", Expr::literal_integer(1))])))
        );
        assert_eq!(
            parse_expr("{ \"a b\": 1 }"),
            Ok((
                "",
                Expr::map_literal(vec![("a b", Expr::literal_integer(1))])
            ))
        );
    }

    #[test]
    fn test_parse_empty_braces_is_an_empty_map() {
        assert_eq!(
            parse_expr("{}"),
            Ok(("", Expr::map_literal(Vec::<(String, Expr)>::new())))
        );
        assert_eq!(
            parse_expr("{ }"),
            Ok(("", Expr::map_literal(Vec::<(String, Expr)>::new())))
        );
    }

    #[test]
    fn test_parse_empty_braces_as_an_if_arm_is_an_empty_block() {
        assert_eq!(
            parse_expr("if true {}"),
            Ok((
                "",
                Expr::if_expr(Expr::literal_boolean(true), Expr::block(vec![]), None)
            ))
        );
        assert_eq!(
            parse_expr("if true 1 else {}"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(true),
                    Expr::literal_integer(1),
                    Some(Expr::block(vec![]))
                )
            ))
        );
        assert_eq!(
            parse_expr("if true { } else { }"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(true),
                    Expr::block(vec![]),
                    Some(Expr::block(vec![]))
                )
            ))
        );
    }

    #[test]
    fn test_parse_parenthesized_braces_as_an_if_arm_is_an_empty_map() {
        assert_eq!(
            parse_expr("if true ({})"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(true),
                    Expr::parenthesized(Expr::map_literal(Vec::<(String, Expr)>::new())),
                    None
                )
            ))
        );
    }

    #[test]
    fn test_parse_map_as_an_if_arm_is_a_map() {
        assert_eq!(
            parse_expr("if true { a: 1 }"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::literal_boolean(true),
                    Expr::map_literal(vec![("a", Expr::literal_integer(1))]),
                    None
                )
            ))
        );
    }

    #[test]
    fn test_parse_if_arms_with_statements() {
        let (rest, expr) =
            parse_expr("if x {let a = 1; a * 2} else {0}").expect("the arms should parse");
        assert_eq!(rest, "");
        assert_eq!(expr.to_string(), "if x { let a = 1; a * 2 } else { 0 }");
    }

    #[test]
    fn test_parse_if_arms_on_separate_lines() {
        let (rest, expr) = parse_expr("if x {\n  let a = 1\n  a * 2\n} else {\n  0\n}").unwrap();
        assert_eq!(rest, "");
        assert_eq!(expr.to_string(), "if x { let a = 1; a * 2 } else { 0 }");
    }

    #[test]
    fn test_parse_if_with_a_block_arm_does_not_take_the_next_line_as_its_arm() {
        let (rest, expr) = parse_expr("if x { 1 }\nfoo").unwrap();
        assert_eq!(rest.trim(), "foo");
        assert_eq!(expr.to_string(), "if x { 1 }");
    }

    #[test]
    fn test_parse_block_is_not_an_argument_without_parentheses() {
        let (rest, expr) = parse_expr("f { 1 }").unwrap();
        assert_eq!(expr, Expr::ident("f"));
        assert_eq!(rest.trim(), "{ 1 }");
    }

    #[test]
    fn test_parse_map_is_an_argument_without_parentheses() {
        let (rest, expr) = parse_expr("f { a: 1 }").unwrap();
        assert_eq!(rest, "");
        assert_eq!(expr.to_string(), "f({ a: 1 })");
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
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Plus, Expr::literal_integer(1))
            ))
        );
        assert!(parse_expr("! x").is_err());
        assert!(parse_expr("+ 1").is_err());
        assert!(parse_expr("- 42").is_err());
        assert_eq!(
            parse_expr("-42"),
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Minus, Expr::literal_integer(42))
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
                    Expr::literal_integer(1),
                    BinaryOp::Add,
                    Expr::literal_integer(2)
                )
            ))
        );
        assert_eq!(
            parse_expr("10 * 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_integer(10),
                    BinaryOp::Mul,
                    Expr::literal_integer(2)
                )
            ))
        );
    }

    #[test]
    fn test_parse_binary_add_spacing_variants() {
        let expected = Expr::binary_expr(Expr::ident("x"), BinaryOp::Add, Expr::literal_integer(1));
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
                    Expr::literal_integer(1),
                    BinaryOp::Add,
                    Expr::binary_expr(
                        Expr::literal_integer(2),
                        BinaryOp::Mul,
                        Expr::literal_integer(3)
                    )
                )
            ))
        );
    }

    #[test]
    fn test_parse_function_call() {
        assert_eq!(
            parse_expr("foo()"),
            Ok(("", Expr::function_call("foo", vec![])))
        );
        assert_eq!(
            parse_expr("bar(1)"),
            Ok((
                "",
                Expr::function_call("bar", vec![Expr::literal_integer(1)])
            ))
        );
        assert_eq!(
            parse_expr("add(1, 2)"),
            Ok((
                "",
                Expr::function_call(
                    "add",
                    vec![Expr::literal_integer(1), Expr::literal_integer(2)]
                )
            ))
        );
        assert_eq!(
            parse_expr("add 1, 2"),
            Ok((
                "",
                Expr::function_call(
                    "add",
                    vec![Expr::literal_integer(1), Expr::literal_integer(2)]
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
                    Expr::literal_integer(1),
                    BinaryOp::Pipe,
                    Expr::function_call("add", vec![Expr::literal_integer(2)])
                )
            ))
        );
        assert_eq!(
            parse_expr("1 |> add"),
            Ok((
                "",
                Expr::binary_expr(Expr::literal_integer(1), BinaryOp::Pipe, Expr::ident("add"))
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
                        Expr::literal_integer(1),
                        BinaryOp::Add,
                        Expr::literal_integer(2)
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
            Ok(("", Expr::literal_integer(42)))
        );
        assert_eq!(
            parse_expr("# leading comment\n42"),
            Ok(("", Expr::literal_integer(42)))
        );
        assert_eq!(
            parse_expr("1 # add these\n+ 2"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::literal_integer(1),
                    BinaryOp::Add,
                    Expr::literal_integer(2)
                )
            ))
        );
        assert_eq!(
            parse_expr("add(1, # first\n 2)"),
            Ok((
                "",
                Expr::function_call(
                    "add",
                    vec![Expr::literal_integer(1), Expr::literal_integer(2)]
                )
            ))
        );
    }

    #[test]
    fn test_parse_field_access() {
        assert_eq!(
            parse_expr("u.foo"),
            Ok(("", Expr::field_access(Expr::ident("u"), "foo")))
        );
    }

    #[test]
    fn test_parse_chained_field_access() {
        assert_eq!(
            parse_expr("a.b.c"),
            Ok((
                "",
                Expr::field_access(Expr::field_access(Expr::ident("a"), "b"), "c")
            ))
        );
    }

    #[test]
    fn test_parse_field_access_on_map_literal() {
        assert_eq!(
            parse_expr(r#"{ foo: "bar" }.foo"#),
            Ok((
                "",
                Expr::field_access(
                    Expr::map_literal(vec![("foo", Expr::literal_string("bar"))]),
                    "foo"
                )
            ))
        );
    }

    #[test]
    fn test_parse_unary_binds_looser_than_dot() {
        assert_eq!(
            parse_expr("-u.foo"),
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Minus, Expr::field_access(Expr::ident("u"), "foo"))
            ))
        );
        assert_eq!(
            parse_expr("!x.y"),
            Ok((
                "",
                Expr::unary_expr(UnaryOp::Not, Expr::field_access(Expr::ident("x"), "y"))
            ))
        );
    }

    #[test]
    fn test_parse_if_condition_with_field_access() {
        assert_eq!(
            parse_expr("if u.active 1 else 0"),
            Ok((
                "",
                Expr::if_expr(
                    Expr::field_access(Expr::ident("u"), "active"),
                    Expr::literal_integer(1),
                    Some(Expr::literal_integer(0))
                )
            ))
        );
    }

    #[test]
    fn test_parse_field_access_no_whitespace_allowed() {
        let (remainder, expr) = parse_expr("u . foo").unwrap();
        assert_eq!(expr, Expr::ident("u"));
        assert!(remainder.contains("foo"));
    }

    #[test]
    fn test_parse_parenthesized() {
        assert_eq!(
            parse_expr("(1 + 2) * 3"),
            Ok((
                "",
                Expr::binary_expr(
                    Expr::parenthesized(Expr::binary_expr(
                        Expr::literal_integer(1),
                        BinaryOp::Add,
                        Expr::literal_integer(2)
                    )),
                    BinaryOp::Mul,
                    Expr::literal_integer(3)
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
                Expr::function_call_with_keywords(
                    "foo",
                    vec![Expr::literal_integer(1)],
                    vec![("optional", Expr::literal_boolean(true))],
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
                Expr::function_call_with_keywords(
                    "config",
                    vec![],
                    vec![
                        ("format", Expr::literal_string("csv")),
                        ("header", Expr::literal_boolean(true)),
                    ],
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
                    Expr::function_call_with_keywords(
                        "WRITE",
                        vec![Expr::literal_string("output")],
                        vec![("format", Expr::literal_string("csv"))],
                    )
                )
            ))
        );
    }

    /// Asserts that `input` parses completely to `expected`.
    fn assert_parses_to(input: &str, expected: Expr) {
        assert_eq!(parse_expr(input), Ok(("", expected)), "parsing {input:?}");
    }

    fn bin(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr::binary_expr(left, op, right)
    }

    fn int(n: i64) -> Expr {
        Expr::literal_integer(n)
    }

    fn id(name: &str) -> Expr {
        Expr::ident(name)
    }

    #[test]
    fn test_parse_comparisons_bind_looser_than_arithmetic() {
        assert_parses_to(
            "1 + 2 == 3",
            bin(bin(int(1), BinaryOp::Add, int(2)), BinaryOp::Eq, int(3)),
        );
        assert_parses_to(
            "x == y - 1",
            bin(id("x"), BinaryOp::Eq, bin(id("y"), BinaryOp::Sub, int(1))),
        );
        assert_parses_to(
            "x * 2 < y",
            bin(bin(id("x"), BinaryOp::Mul, int(2)), BinaryOp::Lt, id("y")),
        );
        assert_parses_to(
            r#""a" <> "b" == "ab""#,
            bin(
                bin(
                    Expr::literal_string("a"),
                    BinaryOp::Concat,
                    Expr::literal_string("b"),
                ),
                BinaryOp::Eq,
                Expr::literal_string("ab"),
            ),
        );
    }

    #[test]
    fn test_parse_floor_division() {
        assert_parses_to("7 // 2", bin(int(7), BinaryOp::FloorDiv, int(2)));
        assert_parses_to("a//b", bin(id("a"), BinaryOp::FloorDiv, id("b")));
    }

    #[test]
    fn test_parse_floor_division_binds_like_multiplication() {
        assert_parses_to(
            "7 // 2 * 2",
            bin(
                bin(int(7), BinaryOp::FloorDiv, int(2)),
                BinaryOp::Mul,
                int(2),
            ),
        );
        assert_parses_to(
            "1 + 7 // 2",
            bin(
                int(1),
                BinaryOp::Add,
                bin(int(7), BinaryOp::FloorDiv, int(2)),
            ),
        );
        assert_parses_to(
            "8 / 2 // 3",
            bin(
                bin(int(8), BinaryOp::Div, int(2)),
                BinaryOp::FloorDiv,
                int(3),
            ),
        );
        assert_parses_to(
            "-7 // 2",
            bin(
                Expr::unary_expr(UnaryOp::Minus, int(7)),
                BinaryOp::FloorDiv,
                int(2),
            ),
        );
    }

    #[test]
    fn test_parse_equality_binds_looser_than_ordering() {
        assert_parses_to(
            "a < b == c",
            bin(bin(id("a"), BinaryOp::Lt, id("b")), BinaryOp::Eq, id("c")),
        );
        assert_parses_to(
            "a != b >= c",
            bin(id("a"), BinaryOp::Ne, bin(id("b"), BinaryOp::Gte, id("c"))),
        );
    }

    #[test]
    fn test_parse_comparisons_bind_looser_than_bitwise() {
        assert_parses_to(
            "x & 1 == 0",
            bin(bin(id("x"), BinaryOp::BitAnd, int(1)), BinaryOp::Eq, int(0)),
        );
        assert_parses_to(
            "x | y < z",
            bin(
                bin(id("x"), BinaryOp::BitOr, id("y")),
                BinaryOp::Lt,
                id("z"),
            ),
        );
    }

    #[test]
    fn test_parse_logical_binds_looser_than_comparisons() {
        assert_parses_to(
            "a == b && c",
            bin(bin(id("a"), BinaryOp::Eq, id("b")), BinaryOp::And, id("c")),
        );
        assert_parses_to(
            "a || b < c",
            bin(id("a"), BinaryOp::Or, bin(id("b"), BinaryOp::Lt, id("c"))),
        );
    }

    #[test]
    fn test_parse_pipe_binds_tighter_than_comparisons_and_logical() {
        assert_parses_to(
            "x |> f == 3",
            bin(bin(id("x"), BinaryOp::Pipe, id("f")), BinaryOp::Eq, int(3)),
        );
        assert_parses_to(
            "x |> f < 3",
            bin(bin(id("x"), BinaryOp::Pipe, id("f")), BinaryOp::Lt, int(3)),
        );
        assert_parses_to(
            "a || b |> f",
            bin(id("a"), BinaryOp::Or, bin(id("b"), BinaryOp::Pipe, id("f"))),
        );
        assert_parses_to(
            "a && b |> f",
            bin(
                id("a"),
                BinaryOp::And,
                bin(id("b"), BinaryOp::Pipe, id("f")),
            ),
        );
    }

    #[test]
    fn test_parse_pipe_binds_tighter_than_bitwise() {
        assert_parses_to(
            "x |> f & 1",
            bin(
                bin(id("x"), BinaryOp::Pipe, id("f")),
                BinaryOp::BitAnd,
                int(1),
            ),
        );
        assert_parses_to(
            "x | y |> f",
            bin(
                id("x"),
                BinaryOp::BitOr,
                bin(id("y"), BinaryOp::Pipe, id("f")),
            ),
        );
    }

    #[test]
    fn test_parse_arithmetic_binds_tighter_than_pipe() {
        assert_parses_to(
            "1 + 1 |> f",
            bin(bin(int(1), BinaryOp::Add, int(1)), BinaryOp::Pipe, id("f")),
        );
        assert_parses_to(
            r#""a" <> "b" |> f"#,
            bin(
                bin(
                    Expr::literal_string("a"),
                    BinaryOp::Concat,
                    Expr::literal_string("b"),
                ),
                BinaryOp::Pipe,
                id("f"),
            ),
        );
        assert_parses_to(
            "x |> f(2) * 3",
            bin(
                id("x"),
                BinaryOp::Pipe,
                bin(
                    Expr::function_call("f", vec![int(2)]),
                    BinaryOp::Mul,
                    int(3),
                ),
            ),
        );
    }

    /// `if success "Ok" else ...` forces the `if`-condition grammar (no parenless calls),
    /// so these exercise the `parse_if_condition_*` chain rather than the main one.
    #[test]
    fn test_parse_if_condition_precedence() {
        let if_ok = |condition: Expr| {
            Expr::if_expr(
                condition,
                Expr::literal_string("Ok"),
                Some(Expr::literal_string("No")),
            )
        };
        assert_parses_to(
            r#"if 1 + 2 == n "Ok" else "No""#,
            if_ok(bin(
                bin(int(1), BinaryOp::Add, int(2)),
                BinaryOp::Eq,
                id("n"),
            )),
        );
        assert_parses_to(
            r#"if x |> f == n "Ok" else "No""#,
            if_ok(bin(
                bin(id("x"), BinaryOp::Pipe, id("f")),
                BinaryOp::Eq,
                id("n"),
            )),
        );
        assert_parses_to(
            r#"if x & 1 == n "Ok" else "No""#,
            if_ok(bin(
                bin(id("x"), BinaryOp::BitAnd, int(1)),
                BinaryOp::Eq,
                id("n"),
            )),
        );
        assert_parses_to(
            r#"if a || b |> f "Ok" else "No""#,
            if_ok(bin(
                id("a"),
                BinaryOp::Or,
                bin(id("b"), BinaryOp::Pipe, id("f")),
            )),
        );
        assert_parses_to(
            r#"if a < b == c "Ok" else "No""#,
            if_ok(bin(
                bin(id("a"), BinaryOp::Lt, id("b")),
                BinaryOp::Eq,
                id("c"),
            )),
        );
    }

    #[test]
    fn test_parse_call_forms_next_to_pipe() {
        // `identity 1` and `identity(1)` parse the same; the pipe forms parse differently
        let call = Expr::function_call("identity", vec![int(1)]);
        assert_parses_to("identity 1", call.clone());
        assert_parses_to("identity(1)", call);
        assert_parses_to("1 |> identity", bin(int(1), BinaryOp::Pipe, id("identity")));
        assert_parses_to(
            "1 |> identity()",
            bin(
                int(1),
                BinaryOp::Pipe,
                Expr::function_call("identity", vec![]),
            ),
        );
    }

    #[test]
    fn test_parse_pipe_into_call_without_parens_leaves_arguments_unparsed() {
        assert_eq!(
            parse_expr("1 |> f 2"),
            Ok(("2", bin(int(1), BinaryOp::Pipe, id("f"))))
        );
        assert_eq!(
            parse_expr("1 |> f 2, 3"),
            Ok(("2, 3", bin(int(1), BinaryOp::Pipe, id("f"))))
        );
        assert_eq!(
            parse_expr("a |> f |> g 1"),
            Ok((
                "1",
                bin(
                    bin(id("a"), BinaryOp::Pipe, id("f")),
                    BinaryOp::Pipe,
                    id("g"),
                )
            ))
        );
    }

    #[test]
    fn test_parse_call_without_parens_rejects_piped_argument() {
        assert_eq!(parse_expr("f 1 |> g()"), Ok(("1 |> g()", id("f"))));
        assert_eq!(parse_expr("f 1, 2 |> g()"), Ok(("1, 2 |> g()", id("f"))));
        assert_eq!(
            parse_expr("f 1 == 2 |> g()"),
            Ok(("1 == 2 |> g()", id("f")))
        );
        assert_eq!(parse_expr("f k: 1 |> g()"), Ok(("k: 1 |> g()", id("f"))));
    }

    #[test]
    fn test_parse_pipe_with_parenthesized_calls() {
        assert_parses_to(
            "f(1) |> g()",
            bin(
                Expr::function_call("f", vec![int(1)]),
                BinaryOp::Pipe,
                Expr::function_call("g", vec![]),
            ),
        );
        // the call's own parentheses, with whitespace before them
        assert_parses_to(
            "f (1 |> g())",
            Expr::function_call(
                "f",
                vec![bin(
                    int(1),
                    BinaryOp::Pipe,
                    Expr::function_call("g", vec![]),
                )],
            ),
        );
        // a parenthesized argument
        assert_parses_to(
            "f 2, (1 |> g())",
            Expr::function_call(
                "f",
                vec![
                    int(2),
                    Expr::parenthesized(bin(
                        int(1),
                        BinaryOp::Pipe,
                        Expr::function_call("g", vec![]),
                    )),
                ],
            ),
        );
        // a whitespace-form call on the left, without a pipe in its arguments
        assert_parses_to(
            "f 1 + 2",
            Expr::function_call("f", vec![bin(int(1), BinaryOp::Add, int(2))]),
        );
    }

    #[test]
    fn test_parse_call_without_parens_allows_pipe_in_interpolation() {
        assert_parses_to(
            r#"f "a{x |> g()}""#,
            Expr::function_call(
                "f",
                vec![bin(
                    Expr::literal_string("a"),
                    BinaryOp::Concat,
                    bin(id("x"), BinaryOp::Pipe, Expr::function_call("g", vec![])),
                )],
            ),
        );
    }

    #[test]
    fn test_parse_if_condition_ending_in_pipe_into_bare_name() {
        assert_parses_to(
            r#"if x |> f "Ok" else "No""#,
            Expr::if_expr(
                bin(id("x"), BinaryOp::Pipe, id("f")),
                Expr::literal_string("Ok"),
                Some(Expr::literal_string("No")),
            ),
        );
    }
}
