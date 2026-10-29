use crate::ast::Expr;
use crate::ast::Statement;
use crate::errors::Error;
use crate::runtime::Runtime;
use crate::runtime::SimpleRuntime;

pub fn eval(expr: &Expr) -> Result<String, Error> {
    let mut rt = SimpleRuntime::default();
    let value = rt.eval_statement(&Statement::Expr(expr.clone()))?;
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::ast::UnaryOp;
    use crate::errors::Error;
    use crate::errors::RuntimeError;

    use super::eval;

    fn n(s: &str) -> BigDecimal {
        BigDecimal::from_str(s).unwrap()
    }

    /// `left op right`, evaluated
    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Result<String, Error> {
        eval(&Expr::binary_expr(left, op, right))
    }

    fn int(i: i64) -> Expr {
        Expr::literal_integer(i)
    }

    fn num(s: &str) -> Expr {
        Expr::literal_number(n(s))
    }

    #[test]
    fn test_eval_literal_number() {
        let expr = Expr::literal_integer(42);
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_literal_string() {
        let expr = Expr::literal_string("hello");
        assert_eq!(eval(&expr).unwrap(), "\"hello\"");
    }

    #[test]
    fn test_eval_literal_boolean() {
        let expr = Expr::literal_boolean(true);
        assert_eq!(eval(&expr).unwrap(), "true");
    }

    #[test]
    fn test_eval_literal_symbol() {
        let expr = Expr::literal_symbol("foo");
        assert_eq!(eval(&expr).unwrap(), ":foo");
    }

    #[test]
    fn test_eval_ident_unbound() {
        let expr = Expr::ident("x");
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::UnboundIdentifier(s)) if s == "x"
        ));
    }

    #[test]
    fn test_eval_parenthesized() {
        let expr = Expr::parenthesized(Expr::literal_integer(99));
        assert_eq!(eval(&expr).unwrap(), "99");
    }

    #[test]
    fn test_eval_undefined_function() {
        let expr = Expr::function_call("foo", vec![]);
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::UndefinedFunction(name)) if name == "foo"
        ));
    }

    #[test]
    fn test_eval_typeof_integer() {
        let expr = Expr::function_call("typeof", vec![Expr::literal_integer(42)]);
        assert_eq!(eval(&expr).unwrap(), "Integer");
    }

    #[test]
    fn test_eval_typeof_number() {
        let expr = Expr::function_call("typeof", vec![Expr::literal_number(n("4.2"))]);
        assert_eq!(eval(&expr).unwrap(), "Number");
    }

    #[test]
    fn test_eval_typeof_string() {
        let expr = Expr::function_call("typeof", vec![Expr::literal_string("hi")]);
        assert_eq!(eval(&expr).unwrap(), "String");
    }

    #[test]
    fn test_eval_typeof_arity_mismatch() {
        let expr = Expr::function_call("typeof", vec![]);
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::ArgumentCountMismatch { found: 0, .. })
        ));
    }

    #[test]
    fn test_eval_unary_not_true() {
        let expr = Expr::unary_expr(UnaryOp::Not, Expr::literal_boolean(true));
        assert_eq!(eval(&expr).unwrap(), "false");
    }

    #[test]
    fn test_eval_unary_not_false() {
        let expr = Expr::unary_expr(UnaryOp::Not, Expr::literal_boolean(false));
        assert_eq!(eval(&expr).unwrap(), "true");
    }

    #[test]
    fn test_eval_unary_not_invalid_type() {
        let expr = Expr::unary_expr(UnaryOp::Not, Expr::literal_integer(42));
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_unary_plus() {
        let expr = Expr::unary_expr(UnaryOp::Plus, Expr::literal_integer(42));
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_unary_minus() {
        let expr = Expr::unary_expr(UnaryOp::Minus, Expr::literal_integer(42));
        assert_eq!(eval(&expr).unwrap(), "-42");
    }

    #[test]
    fn test_eval_unary_minus_invalid_type() {
        let expr = Expr::unary_expr(UnaryOp::Minus, Expr::literal_boolean(true));
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_one_plus_one_returns_two() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(1),
            BinaryOp::Add,
            Expr::literal_integer(1),
        );
        assert_eq!(eval(&expr).unwrap(), "2");
    }

    #[test]
    fn test_eval_binary_add() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(10),
            BinaryOp::Add,
            Expr::literal_integer(32),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_sub() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(50),
            BinaryOp::Sub,
            Expr::literal_integer(8),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_mul() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(6),
            BinaryOp::Mul,
            Expr::literal_integer(7),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_div() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(84),
            BinaryOp::Div,
            Expr::literal_integer(2),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_div_by_zero() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(1),
            BinaryOp::Div,
            Expr::literal_integer(0),
        );
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::DivisionByZero)
        ));
    }

    #[test]
    fn test_eval_binary_and() {
        let expr = Expr::binary_expr(
            Expr::literal_boolean(true),
            BinaryOp::And,
            Expr::literal_boolean(false),
        );
        assert_eq!(eval(&expr).unwrap(), "false");
    }

    #[test]
    fn test_eval_binary_or() {
        let expr = Expr::binary_expr(
            Expr::literal_boolean(true),
            BinaryOp::Or,
            Expr::literal_boolean(false),
        );
        assert_eq!(eval(&expr).unwrap(), "true");
    }

    #[test]
    fn test_eval_binary_xor() {
        let expr = Expr::binary_expr(
            Expr::literal_boolean(true),
            BinaryOp::Xor,
            Expr::literal_boolean(false),
        );
        assert_eq!(eval(&expr).unwrap(), "true");
    }

    #[test]
    fn test_eval_binary_concat() {
        let expr = Expr::binary_expr(
            Expr::literal_string("foo"),
            BinaryOp::Concat,
            Expr::literal_string("bar"),
        );
        assert_eq!(eval(&expr).unwrap(), "\"foobar\"");
    }

    #[test]
    fn test_eval_binary_eq() {
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Eq,
                Expr::literal_integer(1),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Eq,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "false"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_string("a"),
                BinaryOp::Eq,
                Expr::literal_string("a"),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_boolean(true),
                BinaryOp::Eq,
                Expr::literal_boolean(false),
            ))
            .unwrap(),
            "false"
        );
    }

    #[test]
    fn test_eval_binary_ne() {
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Ne,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Ne,
                Expr::literal_integer(1),
            ))
            .unwrap(),
            "false"
        );
    }

    #[test]
    fn test_eval_binary_gt_lt_gte_lte() {
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(3),
                BinaryOp::Gt,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Gt,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "false"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Lt,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(2),
                BinaryOp::Lte,
                Expr::literal_integer(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_integer(3),
                BinaryOp::Gte,
                Expr::literal_integer(3),
            ))
            .unwrap(),
            "true"
        );
    }

    #[test]
    fn test_eval_string_interpolation() {
        let expr = Expr::binary_expr(
            Expr::binary_expr(
                Expr::literal_string("Answer: "),
                BinaryOp::Concat,
                Expr::binary_expr(
                    Expr::literal_integer(1),
                    BinaryOp::Add,
                    Expr::literal_integer(2),
                ),
            ),
            BinaryOp::Concat,
            Expr::literal_string("!"),
        );
        assert_eq!(eval(&expr).unwrap(), "\"Answer: 3!\"");
    }

    #[test]
    fn test_eval_binary_concat_chain() {
        let expr = Expr::binary_expr(
            Expr::binary_expr(
                Expr::literal_string("hello"),
                BinaryOp::Concat,
                Expr::literal_string(" "),
            ),
            BinaryOp::Concat,
            Expr::literal_string("world"),
        );
        assert_eq!(eval(&expr).unwrap(), "\"hello world\"");
    }

    #[test]
    fn test_eval_binary_concat_coerces_to_string() {
        let expr = Expr::binary_expr(
            Expr::literal_string("foo"),
            BinaryOp::Concat,
            Expr::literal_integer(42),
        );
        assert_eq!(eval(&expr).unwrap(), "\"foo42\"");
    }

    #[test]
    fn test_eval_binary_pipe_into_bare_name() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(1),
            BinaryOp::Pipe,
            Expr::ident("typeof"),
        );
        assert_eq!(eval(&expr).unwrap(), "Integer");
    }

    #[test]
    fn test_eval_binary_pipe_into_call() {
        let expr = Expr::binary_expr(
            Expr::literal_string("a"),
            BinaryOp::Pipe,
            Expr::function_call("typeof", vec![]),
        );
        assert_eq!(eval(&expr).unwrap(), "String");
    }

    #[test]
    fn test_eval_binary_pipe_chain() {
        let expr = Expr::binary_expr(
            Expr::binary_expr(
                Expr::literal_integer(1),
                BinaryOp::Pipe,
                Expr::ident("typeof"),
            ),
            BinaryOp::Pipe,
            Expr::ident("typeof"),
        );
        assert_eq!(eval(&expr).unwrap(), "Type");
    }

    #[test]
    fn test_eval_binary_pipe_into_non_call() {
        let expr = Expr::binary_expr(
            Expr::literal_integer(1),
            BinaryOp::Pipe,
            Expr::literal_integer(2),
        );
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidPipeTarget(target)) if target == "2"
        ));
    }

    #[test]
    fn test_eval_if_expr_with_else() {
        let expr = Expr::if_expr(
            Expr::literal_boolean(true),
            Expr::literal_integer(1),
            Some(Expr::literal_integer(2)),
        );
        assert_eq!(eval(&expr).unwrap(), "1");

        let expr = Expr::if_expr(
            Expr::literal_boolean(false),
            Expr::literal_integer(1),
            Some(Expr::literal_integer(2)),
        );
        assert_eq!(eval(&expr).unwrap(), "2");
    }

    #[test]
    fn test_eval_if_expr_without_else_returns_none() {
        let expr = Expr::if_expr(Expr::literal_boolean(true), Expr::literal_integer(1), None);
        assert_eq!(eval(&expr).unwrap(), "1");

        let expr = Expr::if_expr(Expr::literal_boolean(false), Expr::literal_integer(1), None);
        assert_eq!(eval(&expr).unwrap(), "None");
    }

    #[test]
    fn test_eval_field_access_on_map_literal() {
        let expr = Expr::field_access(
            Expr::map_literal(vec![("foo", Expr::literal_string("bar"))]),
            "foo",
        );
        assert_eq!(eval(&expr).unwrap(), "\"bar\"");
    }

    #[test]
    fn test_eval_chained_field_access() {
        let inner = Expr::map_literal(vec![("bar", Expr::literal_integer(42))]);
        let outer = Expr::map_literal(vec![("baz", inner)]);
        let expr = Expr::field_access(Expr::field_access(outer, "baz"), "bar");
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_field_access_not_found() {
        let expr = Expr::field_access(
            Expr::map_literal(vec![("foo", Expr::literal_string("bar"))]),
            "missing",
        );
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::NoSuchField(f)) if f == "missing"
        ));
    }

    #[test]
    fn test_eval_field_access_on_non_map() {
        let expr = Expr::field_access(Expr::literal_integer(42), "foo");
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_if_expr_condition_must_be_boolean() {
        let expr = Expr::if_expr(
            Expr::literal_integer(1),
            Expr::literal_integer(10),
            Some(Expr::literal_integer(20)),
        );
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_empty_array() {
        let expr = Expr::array_literal(vec![]);
        assert_eq!(eval(&expr).unwrap(), "[]");
    }

    #[test]
    fn test_eval_array_literal() {
        let expr = Expr::array_literal(vec![
            Expr::literal_integer(1),
            Expr::literal_integer(2),
            Expr::literal_integer(3),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[1, 2, 3]");
    }

    #[test]
    fn test_eval_array_mixed_types() {
        let expr = Expr::array_literal(vec![
            Expr::literal_integer(1),
            Expr::literal_string("foo"),
            Expr::literal_boolean(true),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[1, \"foo\", true]");
    }

    #[test]
    fn test_eval_nested_array() {
        let expr = Expr::array_literal(vec![
            Expr::array_literal(vec![Expr::literal_integer(1), Expr::literal_integer(2)]),
            Expr::literal_integer(3),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[[1, 2], 3]");
    }

    #[test]
    fn test_eval_array_propagates_element_error() {
        let expr = Expr::array_literal(vec![Expr::literal_integer(1), Expr::ident("x")]);
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::UnboundIdentifier(s)) if s == "x"
        ));
    }

    #[test]
    fn test_eval_typeof_array() {
        let expr = Expr::function_call(
            "typeof",
            vec![Expr::array_literal(vec![
                Expr::literal_integer(1),
                Expr::literal_integer(2),
            ])],
        );
        assert_eq!(eval(&expr).unwrap(), "Array");
    }

    #[test]
    fn test_eval_literal_numbers_print_with_their_decimal_point() {
        assert_eq!(eval(&num("4.2")).unwrap(), "4.2");
        assert_eq!(eval(&num("42.0")).unwrap(), "42.0");
        assert_eq!(eval(&num("42")).unwrap(), "42.0");
        assert_eq!(eval(&num("2.50")).unwrap(), "2.50");
        assert_eq!(eval(&num("-0.0")).unwrap(), "0.0");
    }

    #[test]
    fn test_eval_integer_arithmetic_stays_integer() {
        assert_eq!(binary(int(1), BinaryOp::Add, int(2)).unwrap(), "3");
        assert_eq!(binary(int(5), BinaryOp::Sub, int(7)).unwrap(), "-2");
        assert_eq!(binary(int(2), BinaryOp::Mul, int(3)).unwrap(), "6");
    }

    #[test]
    fn test_eval_mixed_arithmetic_gives_a_number() {
        assert_eq!(binary(int(1), BinaryOp::Add, num("2.0")).unwrap(), "3.0");
        assert_eq!(binary(num("1.0"), BinaryOp::Add, int(2)).unwrap(), "3.0");
        assert_eq!(binary(int(5), BinaryOp::Sub, num("7.0")).unwrap(), "-2.0");
        assert_eq!(binary(int(2), BinaryOp::Mul, num("1.5")).unwrap(), "3.0");
        assert_eq!(binary(num("2.50"), BinaryOp::Mul, int(2)).unwrap(), "5.00");
    }

    #[test]
    fn test_eval_integer_division_floors() {
        assert_eq!(binary(int(7), BinaryOp::Div, int(2)).unwrap(), "3");
        assert_eq!(binary(int(6), BinaryOp::Div, int(2)).unwrap(), "3");
        assert_eq!(binary(int(-7), BinaryOp::Div, int(2)).unwrap(), "-4");
        assert_eq!(binary(int(7), BinaryOp::Div, int(-2)).unwrap(), "-4");
        assert_eq!(binary(int(-7), BinaryOp::Div, int(-2)).unwrap(), "3");
        assert_eq!(binary(int(-1), BinaryOp::Div, int(3)).unwrap(), "-1");
    }

    #[test]
    fn test_eval_division_with_a_number_gives_a_number() {
        assert_eq!(binary(num("7.0"), BinaryOp::Div, int(2)).unwrap(), "3.5");
        assert_eq!(binary(int(7), BinaryOp::Div, num("2.0")).unwrap(), "3.5");
        assert_eq!(binary(int(-7), BinaryOp::Div, num("2.0")).unwrap(), "-3.5");
        assert_eq!(binary(num("4.0"), BinaryOp::Div, int(2)).unwrap(), "2.0");
    }

    #[test]
    fn test_eval_number_division_keeps_100_significant_digits() {
        let third = binary(num("1.0"), BinaryOp::Div, int(3)).unwrap();
        assert_eq!(third, format!("0.{}", "3".repeat(100)));
    }

    #[test]
    fn test_eval_floor_division() {
        assert_eq!(binary(int(7), BinaryOp::FloorDiv, int(2)).unwrap(), "3");
        assert_eq!(binary(int(-7), BinaryOp::FloorDiv, int(2)).unwrap(), "-4");
        assert_eq!(binary(int(7), BinaryOp::FloorDiv, int(-2)).unwrap(), "-4");
        assert_eq!(binary(int(-7), BinaryOp::FloorDiv, int(-2)).unwrap(), "3");
    }

    #[test]
    fn test_eval_floor_division_takes_only_integers() {
        for (left, right) in [
            (num("7.5"), int(2)),
            (int(7), num("2.0")),
            (num("7.0"), num("2.0")),
            (num("1.0"), int(0)),
            (Expr::literal_string("7"), int(2)),
            (int(7), Expr::literal_boolean(true)),
        ] {
            let err = binary(left, BinaryOp::FloorDiv, right).unwrap_err();
            assert!(matches!(
                err,
                Error::RuntimeError(RuntimeError::InvalidOperandType)
            ));
        }
    }

    #[test]
    fn test_eval_dividing_by_zero() {
        for (left, op, right) in [
            (int(1), BinaryOp::Div, num("0.0")),
            (num("1.0"), BinaryOp::Div, int(0)),
            (num("1.0"), BinaryOp::Div, num("0.0")),
            (int(0), BinaryOp::Div, int(0)),
            (int(1), BinaryOp::FloorDiv, int(0)),
        ] {
            let err = binary(left, op, right).unwrap_err();
            assert!(matches!(
                err,
                Error::RuntimeError(RuntimeError::DivisionByZero)
            ));
        }
    }

    #[test]
    fn test_eval_integers_dont_overflow() {
        let max = Expr::literal_integer(i64::MAX);
        assert_eq!(
            binary(max, BinaryOp::Add, int(1)).unwrap(),
            "9223372036854775808"
        );
        let min = Expr::literal_integer(i64::MIN);
        assert_eq!(
            binary(min, BinaryOp::Sub, int(1)).unwrap(),
            "-9223372036854775809"
        );
    }

    #[test]
    fn test_eval_unary_on_numbers() {
        let minus = Expr::unary_expr(UnaryOp::Minus, num("1.5"));
        assert_eq!(eval(&minus).unwrap(), "-1.5");
        let plus = Expr::unary_expr(UnaryOp::Plus, num("2.0"));
        assert_eq!(eval(&plus).unwrap(), "2.0");
    }

    #[test]
    fn test_eval_integers_and_numbers_compare_by_value() {
        assert_eq!(binary(int(1), BinaryOp::Eq, num("1.0")).unwrap(), "true");
        assert_eq!(binary(num("1.0"), BinaryOp::Eq, int(1)).unwrap(), "true");
        assert_eq!(binary(int(1), BinaryOp::Ne, num("1.0")).unwrap(), "false");
        assert_eq!(binary(int(1), BinaryOp::Eq, num("1.5")).unwrap(), "false");
        assert_eq!(binary(int(1), BinaryOp::Lt, num("1.5")).unwrap(), "true");
        assert_eq!(binary(int(2), BinaryOp::Gt, num("1.5")).unwrap(), "true");
        assert_eq!(binary(int(1), BinaryOp::Lte, num("1.0")).unwrap(), "true");
        assert_eq!(binary(num("1.0"), BinaryOp::Gte, int(1)).unwrap(), "true");
    }

    #[test]
    fn test_eval_collections_compare_integers_and_numbers_by_value() {
        let ints = Expr::array_literal(vec![int(1), int(2)]);
        let nums = Expr::array_literal(vec![num("1.0"), int(2)]);
        assert_eq!(binary(ints, BinaryOp::Eq, nums).unwrap(), "true");
        let ints = Expr::map_literal(vec![("a", int(1))]);
        let nums = Expr::map_literal(vec![("a", num("1.0"))]);
        assert_eq!(binary(ints, BinaryOp::Eq, nums).unwrap(), "true");
    }

    #[test]
    fn test_eval_typeof_tells_integers_and_numbers_apart() {
        let typeof_int = Expr::function_call("typeof", vec![int(1)]);
        let typeof_num = Expr::function_call("typeof", vec![num("1.0")]);
        assert_eq!(
            binary(typeof_int, BinaryOp::Eq, typeof_num).unwrap(),
            "false"
        );
    }

    #[test]
    fn test_eval_concat_converts_numbers_as_they_print() {
        let expr = Expr::binary_expr(Expr::literal_string("x"), BinaryOp::Concat, num("2.0"));
        assert_eq!(eval(&expr).unwrap(), "\"x2.0\"");
        let expr = Expr::binary_expr(Expr::literal_string("x"), BinaryOp::Concat, int(2));
        assert_eq!(eval(&expr).unwrap(), "\"x2\"");
    }
}
