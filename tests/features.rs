use std::fmt;
use std::path::Path;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use chrono::NaiveTime;
use cucumber::gherkin::Step;
use cucumber::given;
use cucumber::then;
use cucumber::when;
use cucumber::World;

use flt::ast::BinaryOp;
use flt::ast::Expr;
use flt::ast::Identifier;
use flt::ast::Literal;
use flt::ast::Statement;
use flt::parser::parse_expr;
use flt::parser::parse_statement;
use flt::runtime::functions::Argument;
use flt::runtime::functions::FunctionDefinition;
use flt::runtime::functions::NativeFunction;
use flt::runtime::native::NativeType;
use flt::runtime::types::Type;
use flt::runtime::Map;
use flt::runtime::MapKey;
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;
use flt::runtime::Value;
use flt::Error;

#[derive(Debug, Default, World)]
pub struct AstWorld {
    pub input: Option<String>,
    pub output: Option<Result<Expr, String>>,
    /// Parsed as a statement when input is e.g. "let x = 1" or "x = 1".
    pub output_statement: Option<Result<Statement, String>>,
    /// Set by array step so "first/second/third element" steps can inspect it.
    pub last_parsed_expr: Option<Expr>,
    /// The displayed value of the last statement evaluated by "I evaluate the input".
    pub result: Option<Result<String, String>>,
    /// Fixture native functions registered by "the native function" before evaluating.
    pub natives: Vec<String>,
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
    if let Ok((remainder, stmt)) = parse_statement(&input) {
        if remainder.trim().is_empty() {
            if let Statement::Expr(expr) = &stmt {
                world.output = Some(Ok(expr.clone()));
            }
            world.output_statement = Some(Ok(stmt));
            return;
        }
    }
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

#[given(expr = r"the native function {string}")]
fn given_the_native_function(world: &mut AstWorld, name: String) {
    assert!(
        native_function(&name).is_some(),
        "no fixture native function named {name:?}"
    );
    world.natives.push(name);
}

/// The fixture native functions that "the native function" can register.
fn native_function(name: &str) -> Option<NativeFunction> {
    match name {
        "identity" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::any(),
                vec![Argument::new("v", Type::any())],
            ),
            |args| Ok(args[0].clone()),
        )),
        "greet" => Some(NativeFunction::with_keywords(
            FunctionDefinition::new("greet", Type::string(), vec![Argument::string("name")])
                .keyword("greeting", Type::string(), Value::String("Hello".into())),
            |args: &[Value], keywords: &Map| match (
                &args[0],
                &keywords[&MapKey::symbol("greeting")],
            ) {
                (Value::String(name), Value::String(greeting)) => {
                    Ok(Value::String(format!("{greeting}, {name}")))
                }
                _ => Ok(Value::None),
            },
        )),
        "options" => Some(NativeFunction::with_keywords(
            FunctionDefinition::new("options", Type::map(), vec![]).rest_keywords(Type::any()),
            |_args: &[Value], keywords: &Map| Ok(Value::Map(keywords.clone())),
        )),
        "count_args" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "count_args",
                Type::number(),
                vec![Argument::variadic("args", Type::any())],
            ),
            |args| Ok(Value::from(args.len() as u64)),
        )),
        // A stub, not a real sprintf: it returns the variadic values it was given, so
        // scenarios can see which arguments went to `format` and which to `args`.
        "sprintf" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "sprintf",
                Type::string(),
                vec![
                    Argument::string("format"),
                    Argument::variadic("args", Type::any()),
                ],
            ),
            |args| Ok(Value::Array(args[1..].to_vec())),
        )),
        "keys" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "keys",
                Type::array(),
                vec![Argument::new("map", Type::map())],
            ),
            |args| match &args[0] {
                Value::Map(map) => Ok(Value::Array(map.keys().cloned().map(Value::from).collect())),
                _ => Ok(Value::Array(vec![])),
            },
        )),
        "date" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "date",
                Type::native::<Date>(),
                vec![
                    Argument::number("yyyy"),
                    Argument::number("mm"),
                    Argument::number("dd"),
                ],
            ),
            |args| {
                let year = i32::try_from(&args[0])?;
                let month = u32::try_from(&args[1])?;
                let day = u32::try_from(&args[2])?;
                match NaiveDate::from_ymd_opt(year, month, day) {
                    Some(date) => Ok(Value::native(Date(date))),
                    None => Err(Error::native(std::io::Error::other(format!(
                        "{year:04}-{month:02}-{day:02} is not a valid date"
                    )))),
                }
            },
        )),
        "days_between" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "days_between",
                Type::number(),
                vec![
                    Argument::new("from", Type::native::<Date>()),
                    Argument::new("to", Type::native::<Date>()),
                ],
            ),
            |args| {
                let from = args[0].downcast_native::<Date>()?;
                let to = args[1].downcast_native::<Date>()?;
                Ok(Value::from((to.0 - from.0).num_days()))
            },
        )),
        "time" => Some(NativeFunction::new(
            FunctionDefinition::new(
                "time",
                Type::native::<Time>(),
                vec![
                    Argument::number("hh"),
                    Argument::number("mm"),
                    Argument::number("ss"),
                ],
            ),
            |args| {
                let hour = u32::try_from(&args[0])?;
                let min = u32::try_from(&args[1])?;
                let sec = u32::try_from(&args[2])?;
                match NaiveTime::from_hms_opt(hour, min, sec) {
                    Some(time) => Ok(Value::native(Time(time))),
                    None => Err(Error::native(std::io::Error::other(format!(
                        "{hour:02}:{min:02}:{sec:02} is not a valid time"
                    )))),
                }
            },
        )),
        _ => None,
    }
}

/// Registers the fixture native types that the named fixture functions use. `Time` is
/// never registered, so it also stands for a type a host forgot to register.
fn register_native_types(runtime: &mut impl Runtime, natives: &[String]) -> Result<(), Error> {
    if natives
        .iter()
        .any(|name| name == "date" || name == "days_between")
    {
        runtime.register_native_type::<Date>()?;
    }
    Ok(())
}

/// chrono's `NaiveDate` as the native type `Date`. `NaiveDate` is from another crate, so
/// this crate can't implement `NativeType` for it directly.
#[derive(PartialEq)]
struct Date(NaiveDate);

/// A date prints as `yyyy-mm-dd`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NativeType for Date {
    const NAME: &'static str = "Date";
}

/// chrono's `NaiveTime` as the native type `Time`, which the fixtures never register.
#[derive(PartialEq)]
struct Time(NaiveTime);

/// A time prints as `hh:mm:ss`.
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NativeType for Time {
    const NAME: &'static str = "Time";
}

#[when(expr = "I evaluate the input")]
fn when_i_evaluate_the_input(world: &mut AstWorld) {
    let input = world.input.take().expect("input should be set");
    world.result = Some(evaluate(&input, &world.natives));
}

/// Registers the named fixture native functions, and the native types they use, through
/// the `Runtime` trait alone, as a host does.
fn install(runtime: &mut impl Runtime, natives: &[String]) -> Result<(), Error> {
    register_native_types(runtime, natives)?;
    for name in natives {
        let function = native_function(name).expect("fixture native function should exist");
        runtime.register_function(function)?;
    }
    Ok(())
}

/// Evaluates `input` in a fresh runtime with the named fixture native functions installed,
/// returning the displayed value of its last statement.
fn evaluate(input: &str, natives: &[String]) -> Result<String, String> {
    let mut runtime = SimpleRuntime::default();
    install(&mut runtime, natives).expect("fixtures should register");
    runtime
        .eval(input)
        .map(|value| value.to_string())
        .map_err(|e| e.to_string())
}

#[then(expr = r#"the result should be {string}"#)]
fn then_the_result_should_be(world: &mut AstWorld, expected: String) {
    let result = world.result.take().expect("result should be set");
    let actual = result.expect("evaluation should succeed");
    assert_eq!(actual, expected);
}

#[then(expr = r#"the evaluation should fail with exactly {string}"#)]
fn then_the_evaluation_should_fail_with_exactly(world: &mut AstWorld, expected: String) {
    let result = world.result.take().expect("result should be set");
    let error = result.expect_err("evaluation should fail");
    assert_eq!(error, expected);
}

#[then(expr = r#"the evaluation should fail with {string}"#)]
fn then_the_evaluation_should_fail_with(world: &mut AstWorld, expected: String) {
    let result = world.result.take().expect("result should be set");
    let error = result.expect_err("evaluation should fail");
    assert!(
        error.contains(&expected),
        "expected error containing {expected:?}, got {error:?}"
    );
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

#[then(expr = r#"the output should print as {string}"#)]
fn then_output_should_print_as(world: &mut AstWorld, expected: String) {
    let output = world.output.as_ref().expect("output should be set");
    let expr = output.as_ref().expect("parse should succeed");
    assert_eq!(expr.to_string(), expected);
}

#[then(expr = "the printed output should parse back to the same expression")]
fn then_printed_output_should_parse_back(world: &mut AstWorld) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    let printed = expr.to_string();
    let (remainder, reparsed) = parse_expr(&printed).expect("printed output should parse");
    assert!(remainder.is_empty(), "unconsumed input: {remainder:?}");
    assert_eq!(
        reparsed, expr,
        "{printed} should parse back to the same expression"
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

#[then(expr = r"the output should be a `Literal::None`")]
fn then_output_should_be_none(world: &mut AstWorld) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::Literal(Literal::None) => {}
        _ => panic!("expected None literal, got {:?}", expr),
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

#[then(
    regex = r#"^the output should be a `Statement::Let\(Identifier\("([^"]*)"\), Expr::Literal\(Literal::Number\((\d+)\)\)\)`$"#
)]
fn then_output_should_be_let_statement_number(
    world: &mut AstWorld,
    ident: String,
    expected_num: i64,
) {
    let output = world
        .output_statement
        .take()
        .expect("output_statement should be set (input was parsed as statement)");
    let stmt = output.expect("statement parse should succeed");
    let expected = Statement::Let(
        Identifier(ident.clone()),
        Expr::literal_number(expected_num),
    );
    assert_eq!(
        stmt, expected,
        "expected Statement::Let(Identifier({ident:?}), Literal::Number({expected_num}))"
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

#[then(expr = r#"the output should be a field access with field {string}"#)]
fn then_output_should_be_field_access(world: &mut AstWorld, field: String) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::FieldAccess(_, f) => assert_eq!(f, &field, "expected field {field:?}"),
        _ => panic!("expected field access, got {expr:?}"),
    }
}

#[then(regex = r#"^the output should be a function call "([^"]*)" with (\d+) args$"#)]
fn then_output_should_be_function_call(world: &mut AstWorld, name: String, arg_count: usize) {
    then_output_should_be_function_call_with_keywords(world, name, arg_count, 0);
}

#[then(
    regex = r#"^the output should be a function call "([^"]*)" with (\d+) args and (\d+) keyword args$"#
)]
fn then_output_should_be_function_call_with_keywords(
    world: &mut AstWorld,
    name: String,
    arg_count: usize,
    keyword_count: usize,
) {
    let output = world.output.take().expect("output should be set");
    let expr = output.expect("parse should succeed");
    match &expr {
        Expr::FunctionCall(call) => {
            assert_eq!(call.name.0, name, "expected function name {name:?}");
            assert_eq!(
                call.positional_args.len(),
                arg_count,
                "expected {arg_count} args, got {}",
                call.positional_args.len()
            );
            assert_eq!(
                call.keyword_args.len(),
                keyword_count,
                "expected {keyword_count} keyword args, got {}",
                call.keyword_args.len()
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
        .as_ref()
        .expect("last_parsed_expr should be set (use 'array with N elements' step first)");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let first = elems
                .first()
                .expect("array should have at least one element");
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
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let second = elems
                .get(1)
                .expect("array should have at least two elements");
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
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems
                .get(2)
                .expect("array should have at least three elements");
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
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let first = elems
                .first()
                .expect("array should have at least one element");
            assert_eq!(first, &Expr::literal_string(&expected));
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(expr = r#"the second element should be the string {string}"#)]
fn then_second_element_should_be_string(world: &mut AstWorld, expected: String) {
    let expr = world
        .last_parsed_expr
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let second = elems
                .get(1)
                .expect("array should have at least two elements");
            assert_eq!(second, &Expr::literal_string(&expected));
        }
        _ => panic!("expected array literal, got {expr:?}"),
    }
}

#[then(expr = r#"the third element should be the string {string}"#)]
fn then_third_element_should_be_string(world: &mut AstWorld, expected: String) {
    let expr = world
        .last_parsed_expr
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems
                .get(2)
                .expect("array should have at least three elements");
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
        .as_ref()
        .expect("last_parsed_expr should be set");
    match expr {
        Expr::ArrayLiteral(elems) => {
            let third = elems
                .get(2)
                .expect("array should have at least three elements");
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
