use std::path::Path;

use bigdecimal::BigDecimal;
use cucumber::gherkin::Step;
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
    /// Set by array step so "first/second/third element" steps can inspect it.
    pub last_parsed_expr: Option<Expr>,
}

#[given(expr = r"the input {string}")]
fn given_the_input(world: &mut AstWorld, input: String) {
    world.input = Some(input);
}

#[given(regex = r"^the multiline input$")]
fn given_the_multiline_input(world: &mut AstWorld, step: &Step) {
    let input = step
        .docstring
        .as_ref()
        .expect("step requires a docstring")
        .trim()
        .to_string();
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

#[then(expr = r#"the output should be {string}"#)]
fn then_output_should_be_string(world: &mut AstWorld, expected: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    assert_eq!(
        format!("{expr:?}"),
        expected,
        "expected expression string {:?}",
        expected
    );
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
    assert!(
        output.is_err(),
        "expected parsing to fail, got {:?}",
        output
    );
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

#[then(expr = "the output should be an empty map")]
fn then_output_should_be_empty_map(world: &mut AstWorld) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::MapLiteral(entries) => {
            assert!(entries.is_empty(), "expected empty map, got {entries:?}");
        }
        _ => panic!("expected map literal, got {expr:?}"),
    }
}

#[then(regex = r#"^the output should be a map with (\d+) entr(?:y|ies)$"#)]
fn then_output_should_be_map_with_n_entries(world: &mut AstWorld, count: usize) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::MapLiteral(entries) => {
            assert_eq!(
                entries.len(),
                count,
                "expected {count} entries, got {}",
                entries.len()
            );
        }
        _ => panic!("expected map literal, got {expr:?}"),
    }
}

#[then(expr = r#"the output should be a map with key {string} and string value {string}"#)]
fn then_output_should_be_map_with_key_string_value(
    world: &mut AstWorld,
    key: String,
    value: String,
) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::MapLiteral(entries) => {
            assert_eq!(entries.len(), 1, "expected single-entry map");
            let kv = &entries[0];
            assert_eq!(kv.key, key, "expected key {key:?}");
            assert_eq!(
                kv.value,
                Expr::literal_string(&value),
                "expected string value {value:?}"
            );
        }
        _ => panic!("expected map literal, got {expr:?}"),
    }
}

#[then(regex = r#"^the output should be a map with key "([^"]*)" and number value (\d+)$"#)]
fn then_output_should_be_map_with_key_number_value(world: &mut AstWorld, key: String, value: i64) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::MapLiteral(entries) => {
            assert_eq!(entries.len(), 1, "expected single-entry map");
            let kv = &entries[0];
            assert_eq!(kv.key, key, "expected key {key:?}");
            assert_eq!(
                kv.value,
                Expr::literal_number(value),
                "expected number value {value}"
            );
        }
        _ => panic!("expected map literal, got {expr:?}"),
    }
}

#[then(regex = r#"^the output should be a function call "([^"]*)" with (\d+) args$"#)]
fn then_output_should_be_function_call(world: &mut AstWorld, name: String, arg_count: usize) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::FunctionCall(ident, args) => {
            assert_eq!(ident.0, name, "expected function name {name:?}");
            assert_eq!(
                args.len(),
                arg_count,
                "expected {arg_count} args, got {}",
                args.len()
            );
        }
        _ => panic!("expected function call, got {expr:?}"),
    }
}

#[then(expr = "the output should be an empty array")]
fn then_output_should_be_empty_array(world: &mut AstWorld) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            assert!(elems.is_empty(), "expected empty array, got {elems:?}");
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(regex = r"^the output should be an array with (\d+) elements?$")]
fn then_output_should_be_array_with_n_elements(world: &mut AstWorld, count: usize) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            assert_eq!(
                elems.len(),
                count,
                "expected {count} elements, got {}",
                elems.len()
            );
            world.last_parsed_expr = Some(expr);
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(regex = r"^the first element should be the number (\d+)$")]
fn then_first_element_should_be_number(world: &mut AstWorld, expected: i64) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set (use 'array with N elements' step first)");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let first = elems.first().expect("array should have at least one element");
            match first {
                Expr::Literal(Literal::Number(n)) => {
                    assert_eq!(n.as_ref(), &BigDecimal::from(expected));
                }
                _ => panic!("expected first element to be number, got {first:?}"),
            }
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(regex = r"^the second element should be the number (\d+)$")]
fn then_second_element_should_be_number(world: &mut AstWorld, expected: i64) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let second = elems.get(1).expect("array should have at least two elements");
            match second {
                Expr::Literal(Literal::Number(n)) => {
                    assert_eq!(n.as_ref(), &BigDecimal::from(expected));
                }
                _ => panic!("expected second element to be number, got {second:?}"),
            }
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(regex = r"^the third element should be the number (\d+)$")]
fn then_third_element_should_be_number(world: &mut AstWorld, expected: i64) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems.get(2).expect("array should have at least three elements");
            match third {
                Expr::Literal(Literal::Number(n)) => {
                    assert_eq!(n.as_ref(), &BigDecimal::from(expected));
                }
                _ => panic!("expected third element to be number, got {third:?}"),
            }
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(expr = r#"the first element should be the string {string}"#)]
fn then_first_element_should_be_string(world: &mut AstWorld, expected: String) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let first = elems.first().expect("array should have at least one element");
            assert_eq!(first, &Expr::literal_string(&expected));
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(expr = r#"the second element should be the string {string}"#)]
fn then_second_element_should_be_string(world: &mut AstWorld, expected: String) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let second = elems.get(1).expect("array should have at least two elements");
            assert_eq!(second, &Expr::literal_string(&expected));
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(expr = r#"the third element should be the string {string}"#)]
fn then_third_element_should_be_string(world: &mut AstWorld, expected: String) {
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems.get(2).expect("array should have at least three elements");
            assert_eq!(third, &Expr::literal_string(&expected));
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(regex = r"^the third element should be the boolean (true|false)$")]
fn then_third_element_should_be_boolean(world: &mut AstWorld, expected: String) {
    let expected: bool = expected.parse().expect("true or false");
    let expr = world
        .last_parsed_expr
        .take()
        .expect("last_parsed_expr should be set");
    match &expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems.get(2).expect("array should have at least three elements");
            match third {
                Expr::Literal(Literal::Boolean(b)) => assert_eq!(*b, expected),
                _ => panic!("expected third element to be boolean, got {third:?}"),
            }
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
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
