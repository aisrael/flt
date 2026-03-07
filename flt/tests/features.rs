use std::path::Path;

use bigdecimal::BigDecimal;
use cucumber::given;
use cucumber::then;
use cucumber::when;
use cucumber::World;

use flt::ast::BinaryOp;
use flt::ast::Expr;
use flt::ast::Literal;
use flt::parser::parse_expr;

#[derive(Debug, Default, World)]
pub struct AstWorld {
    pub input: Option<String>,
    pub output: Option<Result<Expr, String>>,
}

#[given(expr = r"the input {string}")]
fn given_the_input(world: &mut AstWorld, input: String) {
    world.input = Some(input);
}

#[when(expr = "I parse the input")]
fn when_i_parse_the_input(world: &mut AstWorld) {
    let input = world.input.take().expect("input should be set");
    world.output = Some(match parse_expr(&input) {
        Ok((remainder, expr)) => {
            if remainder.is_empty() {
                Ok(expr)
            } else {
                Err(format!("unconsumed input: {:?}", remainder))
            }
        }
        Err(e) => Err(format!("parse error: {}", e)),
    });
}

#[then(expr = r"the output should be a `Literal::Number\({int}\)`")]
fn then_output_should_be_number(world: &mut AstWorld, expected: i64) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Literal(Literal::Number(n)) => {
            let expected_bd = BigDecimal::from(expected);
            assert_eq!(n.as_ref(), &expected_bd, "expected number {}", expected);
        }
        _ => panic!("expected number literal, got {:?}", expr),
    }
}

#[then(expr = r"the output should be a `Literal::String\({string}\)`")]
fn then_output_should_be_string(world: &mut AstWorld, expected: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Literal(Literal::String(s)) => {
            assert_eq!(s.as_str(), expected, "expected string {:?}", expected);
        }
        _ => panic!("expected string literal, got {:?}", expr),
    }
}

#[then(expr = r"the output should be a `Literal::Boolean\({word}\)`")]
fn then_output_should_be_boolean(world: &mut AstWorld, expected: String) {
    let expected: bool = expected.parse().expect("expected 'true' or 'false'");
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Literal(Literal::Boolean(b)) => {
            assert_eq!(*b, expected, "expected boolean {}", expected);
        }
        _ => panic!("expected boolean literal, got {:?}", expr),
    }
}

#[then(regex = r#"^the output should be a `Literal::Symbol\("([^"]*)"\)`$"#)]
fn then_output_should_be_symbol(world: &mut AstWorld, expected: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Literal(Literal::Symbol(s)) => {
            assert_eq!(s.as_str(), expected, "expected symbol {:?}", expected);
        }
        _ => panic!("expected symbol literal, got {:?}", expr),
    }
}

#[then(expr = r#"the output should be an identifier {string}"#)]
fn then_output_should_be_identifier(world: &mut AstWorld, expected: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Ident(s) => {
            assert_eq!(s.as_str(), expected, "expected identifier {:?}", expected);
        }
        _ => panic!("expected identifier, got {:?}", expr),
    }
}

#[then(expr = "parsing should fail")]
fn then_parsing_should_fail(world: &mut AstWorld) {
    let output = world.output.take().expect("output should be set");
    assert!(output.is_err(), "expected parsing to fail, got {:?}", output);
}

#[then(expr = r"the output should parse to interpolated string {string} {word} {string}")]
fn then_output_should_be_interpolated_string(
    world: &mut AstWorld,
    before: String,
    ident: String,
    after: String,
) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    let expected = Expr::binary_expr(
        Expr::binary_expr(
            Expr::literal_string(before),
            BinaryOp::Concat,
            Expr::ident(ident),
        ),
        BinaryOp::Concat,
        Expr::literal_string(after),
    );
    assert_eq!(expr, expected, "expected interpolated string expr");
}

#[then(expr = r"the output should parse to string concat {string} and {string}")]
fn then_output_should_be_string_concat(world: &mut AstWorld, left: String, right: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::BinaryExpr(l, op, r) if *op == BinaryOp::Concat => match (l.as_ref(), r.as_ref()) {
            (Expr::Literal(Literal::String(a)), Expr::Literal(Literal::String(b))) => {
                assert_eq!(a.as_str(), left, "expected left operand {:?}", left);
                assert_eq!(b.as_str(), right, "expected right operand {:?}", right);
            }
            _ => panic!("expected string literals in concat, got {:?} <> {:?}", l, r),
        },
        _ => panic!("expected string concatenation expression, got {:?}", expr),
    }
}

#[tokio::main]
async fn main() {
    let features = Path::new(env!("CARGO_MANIFEST_DIR")).join("features");
    AstWorld::run(features).await;
}
