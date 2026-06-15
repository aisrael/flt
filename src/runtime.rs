//! Runtimes for the `flt` language

pub mod functions;
pub mod types;

use std::collections::HashMap;
use std::fmt;

use bigdecimal::BigDecimal;
use bigdecimal::Zero;

use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Literal;
use crate::ast::Statement;
use crate::ast::UnaryOp;
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::runtime::functions::FunctionDefinition;
use crate::runtime::types::Type;
use crate::utils::escape_string;
use crate::Error;

/// A value in the runtime
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The unit value (like `()` in Rust/Elixir)
    Unit,
    /// The `None` sentinel value (the empty `Option`)
    None,
    /// A number value
    Number(BigDecimal),
    /// A string value
    String(String),
    /// A boolean value
    Boolean(bool),
    /// A symbol value
    Symbol(String),
    /// A map of string keys to values
    Map(HashMap<String, Value>),
    /// An array of values
    Array(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::None => write!(f, "None"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Symbol(s) => write!(f, ":{}", s),
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", escape_string(k), v)?;
                }
                write!(f, "}}")
            }
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Value {
    /// Map a runtime value to its built-in type.
    ///
    /// Bare `None` cannot be typed without contextual information about the
    /// wrapped `Option<T>`, so it returns `InterpreterError::NotYetImplemented`.
    pub fn type_of(&self) -> Result<Type, Error> {
        match self {
            Value::Unit => Ok(Type::unit()),
            Value::None => Err(Error::InterpreterError(InterpreterError::NotYetImplemented)),
            Value::Number(_) => Ok(Type::number()),
            Value::String(_) => Ok(Type::string()),
            Value::Boolean(_) => Ok(Type::boolean()),
            Value::Symbol(_) => Ok(Type::symbol()),
            Value::Map(_) => Ok(Type::map()),
            Value::Array(_) => Ok(Type::array()),
        }
    }
}

pub trait Runtime {
    fn eval(&mut self, statement: &Statement) -> Result<Value, Error>;
}

#[derive(Default)]
pub struct SimpleRuntime {
    pub built_in_functions: HashMap<String, FunctionDefinition>,
    pub global_scope: GlobalScope,
}

impl Runtime for SimpleRuntime {
    fn eval(&mut self, statement: &Statement) -> Result<Value, Error> {
        match statement {
            Statement::Expr(expr) => self.eval_expr(expr),
            Statement::Let(ident, expr) => {
                let value = self.eval_expr(expr)?;
                self.global_scope
                    .set_variable(ident.0.as_str(), value.clone());
                Ok(value)
            }
        }
    }
}

impl SimpleRuntime {
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(lit) => Ok(Self::literal_to_value(lit)),
            Expr::Ident(s) => self
                .global_scope
                .get_variable(s.as_str())
                .cloned()
                .ok_or_else(|| Error::RuntimeError(RuntimeError::UnboundIdentifier(s.clone()))),
            Expr::IfExpr {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                let cond_bool = match cond_val {
                    Value::Boolean(b) => b,
                    _ => return Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
                };

                if cond_bool {
                    self.eval_expr(then_branch)
                } else {
                    match else_branch {
                        Some(expr) => self.eval_expr(expr),
                        None => Ok(Value::Unit),
                    }
                }
            }
            Expr::UnaryExpr(op, inner) => {
                let val = self.eval_expr(inner)?;
                Self::eval_unary(*op, &val)
            }
            Expr::BinaryExpr(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                Self::eval_binary(&l, *op, &r)
            }
            Expr::FunctionCall(_, _) => {
                Err(Error::RuntimeError(RuntimeError::UnsupportedFunctionCall))
            }
            Expr::Parenthesized(inner) => self.eval_expr(inner),
            Expr::MapLiteral(_) => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
            Expr::ArrayLiteral(_) => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
            Expr::Keyword(_) => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        }
    }

    fn literal_to_value(lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(n.as_ref().clone()),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Symbol(s) => Value::Symbol(s.clone()),
            Literal::None => Value::None,
        }
    }

    fn eval_unary(op: UnaryOp, inner: &Value) -> Result<Value, Error> {
        match op {
            UnaryOp::Not => match inner {
                Value::Boolean(b) => Ok(Value::Boolean(!b)),
                _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
            },
            UnaryOp::Plus => match inner {
                Value::Number(n) => Ok(Value::Number(n.clone())),
                _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
            },
            UnaryOp::Minus => match inner {
                Value::Number(n) => Ok(Value::Number(-n.clone())),
                _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
            },
        }
    }

    fn eval_binary(l: &Value, op: BinaryOp, r: &Value) -> Result<Value, Error> {
        match op {
            BinaryOp::Add => Self::binary_number(l, r, |a, b| a + b),
            BinaryOp::Sub => Self::binary_number(l, r, |a, b| a - b),
            BinaryOp::Mul => Self::binary_number(l, r, |a, b| a * b),
            BinaryOp::Div => {
                let (a, b) = (Self::as_bigdecimal(l)?, Self::as_bigdecimal(r)?);
                if b.is_zero() {
                    Err(Error::RuntimeError(RuntimeError::DivisionByZero))
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            BinaryOp::And => Self::binary_bool(l, r, |a, b| a && b),
            BinaryOp::Or => Self::binary_bool(l, r, |a, b| a || b),
            BinaryOp::Xor => Self::binary_bool(l, r, |a, b| a ^ b),
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                Err(Error::RuntimeError(RuntimeError::InvalidOperandType))
            }
            BinaryOp::Concat => Self::binary_string(l, r),
            BinaryOp::Eq => Ok(Value::Boolean(l == r)),
            BinaryOp::Ne => Ok(Value::Boolean(l != r)),
            BinaryOp::Lt => Self::binary_compare(l, r, |a, b| a < b),
            BinaryOp::Gt => Self::binary_compare(l, r, |a, b| a > b),
            BinaryOp::Lte => Self::binary_compare(l, r, |a, b| a <= b),
            BinaryOp::Gte => Self::binary_compare(l, r, |a, b| a >= b),
            BinaryOp::Pipe => Err(Error::RuntimeError(RuntimeError::UnsupportedFunctionCall)),
        }
    }

    fn as_bigdecimal(v: &Value) -> Result<BigDecimal, Error> {
        match v {
            Value::Number(n) => Ok(n.clone()),
            _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        }
    }

    fn binary_number<F>(l: &Value, r: &Value, f: F) -> Result<Value, Error>
    where
        F: FnOnce(BigDecimal, BigDecimal) -> BigDecimal,
    {
        let a = Self::as_bigdecimal(l)?;
        let b = Self::as_bigdecimal(r)?;
        Ok(Value::Number(f(a, b)))
    }

    fn binary_bool<F>(l: &Value, r: &Value, f: F) -> Result<Value, Error>
    where
        F: FnOnce(bool, bool) -> bool,
    {
        match (l, r) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(f(*a, *b))),
            _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
        }
    }

    fn binary_compare<F>(l: &Value, r: &Value, f: F) -> Result<Value, Error>
    where
        F: FnOnce(&BigDecimal, &BigDecimal) -> bool,
    {
        let a = Self::as_bigdecimal(l)?;
        let b = Self::as_bigdecimal(r)?;
        Ok(Value::Boolean(f(&a, &b)))
    }

    fn value_to_concat_str(v: &Value) -> String {
        match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Symbol(s) => s.clone(),
            _ => String::new(),
        }
    }

    fn binary_string(l: &Value, r: &Value) -> Result<Value, Error> {
        let a = Self::value_to_concat_str(l);
        let b = Self::value_to_concat_str(r);
        Ok(Value::String(format!("{}{}", a, b)))
    }
}

/// The global scope is the scope that is available to all other scopes.
#[derive(Default)]
pub struct GlobalScope {
    pub functions: HashMap<String, FunctionDefinition>,
    pub variables: HashMap<String, Value>,
}

impl GlobalScope {
    /// Check if the global scope has a function with the given name.
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get a function from the global scope by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    /// Check if the global scope has a variable with the given name.
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get a variable from the global scope by name.
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable in the global scope by name.
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_of_unit() {
        assert_eq!(Value::Unit.type_of().unwrap(), Type::unit());
    }

    #[test]
    fn test_type_of_number() {
        assert_eq!(
            Value::Number(BigDecimal::from(42)).type_of().unwrap(),
            Type::number()
        );
    }

    #[test]
    fn test_type_of_string() {
        assert_eq!(
            Value::String("hello".to_string()).type_of().unwrap(),
            Type::string()
        );
    }

    #[test]
    fn test_type_of_boolean() {
        assert_eq!(Value::Boolean(true).type_of().unwrap(), Type::boolean());
    }

    #[test]
    fn test_type_of_symbol() {
        assert_eq!(
            Value::Symbol("name".to_string()).type_of().unwrap(),
            Type::symbol()
        );
    }

    #[test]
    fn test_type_of_map() {
        assert_eq!(Value::Map(HashMap::new()).type_of().unwrap(), Type::map());
    }

    #[test]
    fn test_type_of_array() {
        assert_eq!(Value::Array(Vec::new()).type_of().unwrap(), Type::array());
    }

    #[test]
    fn test_type_of_none_is_not_yet_implemented() {
        assert!(matches!(
            Value::None.type_of(),
            Err(Error::InterpreterError(InterpreterError::NotYetImplemented))
        ));
    }
}
