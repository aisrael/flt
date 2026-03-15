use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Literal;
use crate::ast::UnaryOp;
use crate::errors::Error;
use crate::errors::RuntimeError;
use crate::utils::escape_string;
use bigdecimal::BigDecimal;

pub fn eval(expr: &Expr) -> Result<String, Error> {
    let lit = eval_to_literal(expr)?;
    Ok(literal_to_string(&lit))
}

fn eval_to_literal(expr: &Expr) -> Result<Literal, Error> {
    match expr {
        Expr::Literal(lit) => eval_literal(lit),
        Expr::Ident(s) => Err(Error::RuntimeError(RuntimeError::UnboundIdentifier(
            s.clone(),
        ))),
        Expr::UnaryExpr(op, inner) => eval_unary_expr(*op, inner),
        Expr::BinaryExpr(left, op, right) => eval_binary_expr(left, *op, right),
        Expr::FunctionCall(_, _) => Err(Error::RuntimeError(RuntimeError::UnsupportedFunctionCall)),
        Expr::Parenthesized(inner) => eval_to_literal(inner),
        Expr::MapLiteral(_) => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        Expr::ArrayLiteral(_) => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
    }
}

fn literal_to_string(lit: &Literal) -> String {
    match lit {
        Literal::Number(n) => n.as_ref().to_string(),
        Literal::String(s) => format!("\"{}\"", escape_string(s)),
        Literal::Boolean(b) => b.to_string(),
        Literal::Symbol(s) => format!(":{}", s),
    }
}

fn eval_literal(lit: &Literal) -> Result<Literal, Error> {
    match lit {
        Literal::Number(n) => Ok(Literal::number(n.as_ref().clone())),
        Literal::String(s) => Ok(Literal::string(s.clone())),
        Literal::Boolean(b) => Ok(Literal::boolean(*b)),
        Literal::Symbol(s) => Ok(Literal::symbol(s.clone())),
    }
}

fn eval_unary_expr(op: UnaryOp, inner: &Expr) -> Result<Literal, Error> {
    let val = eval_to_literal(inner)?;
    match op {
        UnaryOp::Not => match &val {
            Literal::Boolean(b) => Ok(Literal::boolean(!b)),
            _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        },
        UnaryOp::Plus => match &val {
            Literal::Number(n) => Ok(Literal::number(n.as_ref().clone())),
            _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        },
        UnaryOp::Minus => match &val {
            Literal::Number(n) => Ok(Literal::number(-n.as_ref().clone())),
            _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        },
    }
}

fn eval_binary_expr(left: &Expr, op: BinaryOp, right: &Expr) -> Result<Literal, Error> {
    let l = eval_to_literal(left)?;
    let r = eval_to_literal(right)?;
    match op {
        BinaryOp::Add => binary_number(&l, &r, |a, b| a + b),
        BinaryOp::Sub => binary_number(&l, &r, |a, b| a - b),
        BinaryOp::Mul => binary_number(&l, &r, |a, b| a * b),
        BinaryOp::Div => {
            let (a, b) = (as_bigdecimal(&l)?, as_bigdecimal(&r)?);
            if b == 0 {
                Err(Error::RuntimeError(RuntimeError::DivisionByZero))
            } else {
                Ok(Literal::number(a / b))
            }
        }
        BinaryOp::And => binary_bool(&l, &r, |a, b| a && b),
        BinaryOp::Or => binary_bool(&l, &r, |a, b| a || b),
        BinaryOp::Xor => binary_bool(&l, &r, |a, b| a ^ b),
        BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
            Err(Error::RuntimeError(RuntimeError::InvalidOperandType))
        }
        BinaryOp::Concat => binary_string(&l, &r),
        BinaryOp::Pipe => Err(Error::RuntimeError(RuntimeError::UnsupportedFunctionCall)),
    }
}

fn as_bigdecimal(lit: &Literal) -> Result<BigDecimal, Error> {
    match lit {
        Literal::Number(n) => Ok(n.as_ref().clone()),
        _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
    }
}

fn binary_number<F>(l: &Literal, r: &Literal, f: F) -> Result<Literal, Error>
where
    F: FnOnce(BigDecimal, BigDecimal) -> BigDecimal,
{
    let a = as_bigdecimal(l)?;
    let b = as_bigdecimal(r)?;
    Ok(Literal::number(f(a, b)))
}

fn binary_bool<F>(l: &Literal, r: &Literal, f: F) -> Result<Literal, Error>
where
    F: FnOnce(bool, bool) -> bool,
{
    match (l, r) {
        (Literal::Boolean(a), Literal::Boolean(b)) => Ok(Literal::boolean(f(*a, *b))),
        _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
    }
}

fn literal_to_concat_str(lit: &Literal) -> String {
    match lit {
        Literal::Number(n) => n.as_ref().to_string(),
        Literal::String(s) => s.clone(),
        Literal::Boolean(b) => b.to_string(),
        Literal::Symbol(s) => s.clone(),
    }
}

fn binary_string(l: &Literal, r: &Literal) -> Result<Literal, Error> {
    let a = literal_to_concat_str(l);
    let b = literal_to_concat_str(r);
    Ok(Literal::string(format!("{}{}", a, b)))
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
}
