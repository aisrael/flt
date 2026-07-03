use crate::ast::Expr;
use crate::ast::Statement;
use crate::errors::Error;
use crate::runtime::Runtime;
use crate::runtime::SimpleRuntime;

pub fn eval(expr: &Expr) -> Result<String, Error> {
    let mut rt = SimpleRuntime::default();
    let value = rt.eval(&Statement::Expr(expr.clone()))?;
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use crate::ast::BinaryOp;
    use crate::ast::Expr;
    use crate::ast::UnaryOp;
    use crate::errors::Error;
    use crate::errors::RuntimeError;

    use super::eval;

    #[test]
    fn test_eval_literal_number() {
        let expr = Expr::literal_number(42);
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
        let expr = Expr::parenthesized(Expr::literal_number(99));
        assert_eq!(eval(&expr).unwrap(), "99");
    }

    #[test]
    fn test_eval_function_call_unsupported() {
        let expr = Expr::function_call("foo", vec![]);
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::UnsupportedFunctionCall)
        ));
    }

    #[test]
    fn test_eval_typeof_number() {
        let expr = Expr::function_call("typeof", vec![Expr::literal_number(42)]);
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
        let expr = Expr::unary_expr(UnaryOp::Not, Expr::literal_number(42));
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_unary_plus() {
        let expr = Expr::unary_expr(UnaryOp::Plus, Expr::literal_number(42));
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_unary_minus() {
        let expr = Expr::unary_expr(UnaryOp::Minus, Expr::literal_number(42));
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
            Expr::literal_number(1),
            BinaryOp::Add,
            Expr::literal_number(1),
        );
        assert_eq!(eval(&expr).unwrap(), "2");
    }

    #[test]
    fn test_eval_binary_add() {
        let expr = Expr::binary_expr(
            Expr::literal_number(10),
            BinaryOp::Add,
            Expr::literal_number(32),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_sub() {
        let expr = Expr::binary_expr(
            Expr::literal_number(50),
            BinaryOp::Sub,
            Expr::literal_number(8),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_mul() {
        let expr = Expr::binary_expr(
            Expr::literal_number(6),
            BinaryOp::Mul,
            Expr::literal_number(7),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_div() {
        let expr = Expr::binary_expr(
            Expr::literal_number(84),
            BinaryOp::Div,
            Expr::literal_number(2),
        );
        assert_eq!(eval(&expr).unwrap(), "42");
    }

    #[test]
    fn test_eval_binary_div_by_zero() {
        let expr = Expr::binary_expr(
            Expr::literal_number(1),
            BinaryOp::Div,
            Expr::literal_number(0),
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
                Expr::literal_number(1),
                BinaryOp::Eq,
                Expr::literal_number(1),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(1),
                BinaryOp::Eq,
                Expr::literal_number(2),
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
                Expr::literal_number(1),
                BinaryOp::Ne,
                Expr::literal_number(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(1),
                BinaryOp::Ne,
                Expr::literal_number(1),
            ))
            .unwrap(),
            "false"
        );
    }

    #[test]
    fn test_eval_binary_gt_lt_gte_lte() {
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(3),
                BinaryOp::Gt,
                Expr::literal_number(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(1),
                BinaryOp::Gt,
                Expr::literal_number(2),
            ))
            .unwrap(),
            "false"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(1),
                BinaryOp::Lt,
                Expr::literal_number(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(2),
                BinaryOp::Lte,
                Expr::literal_number(2),
            ))
            .unwrap(),
            "true"
        );
        assert_eq!(
            eval(&Expr::binary_expr(
                Expr::literal_number(3),
                BinaryOp::Gte,
                Expr::literal_number(3),
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
                    Expr::literal_number(1),
                    BinaryOp::Add,
                    Expr::literal_number(2),
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
            Expr::literal_number(42),
        );
        assert_eq!(eval(&expr).unwrap(), "\"foo42\"");
    }

    #[test]
    fn test_eval_binary_pipe_unsupported() {
        let expr = Expr::binary_expr(
            Expr::literal_number(1),
            BinaryOp::Pipe,
            Expr::literal_number(2),
        );
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::UnsupportedFunctionCall)
        ));
    }

    #[test]
    fn test_eval_if_expr_with_else() {
        let expr = Expr::if_expr(
            Expr::literal_boolean(true),
            Expr::literal_number(1),
            Some(Expr::literal_number(2)),
        );
        assert_eq!(eval(&expr).unwrap(), "1");

        let expr = Expr::if_expr(
            Expr::literal_boolean(false),
            Expr::literal_number(1),
            Some(Expr::literal_number(2)),
        );
        assert_eq!(eval(&expr).unwrap(), "2");
    }

    #[test]
    fn test_eval_if_expr_without_else_returns_unit() {
        let expr = Expr::if_expr(Expr::literal_boolean(true), Expr::literal_number(1), None);
        assert_eq!(eval(&expr).unwrap(), "1");

        let expr = Expr::if_expr(Expr::literal_boolean(false), Expr::literal_number(1), None);
        assert_eq!(eval(&expr).unwrap(), "()");
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
        let inner = Expr::map_literal(vec![("bar", Expr::literal_number(42))]);
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
        let expr = Expr::field_access(Expr::literal_number(42), "foo");
        let err = eval(&expr).unwrap_err();
        assert!(matches!(
            err,
            Error::RuntimeError(RuntimeError::InvalidOperandType)
        ));
    }

    #[test]
    fn test_eval_if_expr_condition_must_be_boolean() {
        let expr = Expr::if_expr(
            Expr::literal_number(1),
            Expr::literal_number(10),
            Some(Expr::literal_number(20)),
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
            Expr::literal_number(1),
            Expr::literal_number(2),
            Expr::literal_number(3),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[1, 2, 3]");
    }

    #[test]
    fn test_eval_array_mixed_types() {
        let expr = Expr::array_literal(vec![
            Expr::literal_number(1),
            Expr::literal_string("foo"),
            Expr::literal_boolean(true),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[1, \"foo\", true]");
    }

    #[test]
    fn test_eval_nested_array() {
        let expr = Expr::array_literal(vec![
            Expr::array_literal(vec![Expr::literal_number(1), Expr::literal_number(2)]),
            Expr::literal_number(3),
        ]);
        assert_eq!(eval(&expr).unwrap(), "[[1, 2], 3]");
    }

    #[test]
    fn test_eval_array_propagates_element_error() {
        let expr = Expr::array_literal(vec![Expr::literal_number(1), Expr::ident("x")]);
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
                Expr::literal_number(1),
                Expr::literal_number(2),
            ])],
        );
        assert_eq!(eval(&expr).unwrap(), "Array");
    }
}
