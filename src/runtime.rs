//! Runtimes for the `flt` language

pub mod functions;
pub mod types;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use bigdecimal::Zero;

use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Literal;
use crate::ast::Statement;
use crate::ast::UnaryOp;
use crate::errors::RuntimeError;
use crate::runtime::functions::Argument;
use crate::runtime::functions::FunctionDefinition;
use crate::runtime::functions::NativeFunction;
use crate::runtime::types::Type;
use crate::utils::escape_string;
use crate::Error;

/// A value in the runtime
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The `None` value: the empty `Option`, and the result of an `if` with no `else`
    /// whose condition is `false`. Its type is the built-in `None` type.
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
    /// A type as a first-class value
    Type(Type),
    /// A function as a first-class value
    Function(Arc<NativeFunction>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Symbol(s) => write!(f, ":{}", s),
            Value::Map(m) => {
                let mut entries: Vec<_> = m.iter().collect();
                entries.sort_by_key(|(k, _)| *k);
                write!(f, "{{")?;
                for (i, (k, v)) in entries.into_iter().enumerate() {
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
            Value::Type(t) => write!(f, "{t}"),
            Value::Function(function) => write!(f, "<fn {}>", function.name()),
        }
    }
}

impl Value {
    /// Map a runtime value to its built-in type.
    pub fn type_of(&self) -> Result<Type, Error> {
        match self {
            Value::None => Ok(Type::none()),
            Value::Number(_) => Ok(Type::number()),
            Value::String(_) => Ok(Type::string()),
            Value::Boolean(_) => Ok(Type::boolean()),
            Value::Symbol(_) => Ok(Type::symbol()),
            Value::Map(_) => Ok(Type::map()),
            Value::Array(_) => Ok(Type::array()),
            Value::Type(_) => Ok(Type::type_type()),
            Value::Function(function) => Ok(function.r#type()),
        }
    }
}

pub trait Runtime {
    fn eval(&mut self, statement: &Statement) -> Result<Value, Error>;
}

pub struct SimpleRuntime {
    pub global_scope: GlobalScope,
}

impl Default for SimpleRuntime {
    /// A runtime with the built-in native functions (`typeof`) registered in its global scope.
    fn default() -> Self {
        let mut global_scope = GlobalScope::default();
        register_builtins(&mut global_scope);
        Self { global_scope }
    }
}

/// Registers the built-in native functions in `global_scope`.
fn register_builtins(global_scope: &mut GlobalScope) {
    global_scope.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "typeof",
            Type::type_type(),
            vec![Argument::new("V", Type::any())],
        ),
        builtin_typeof,
    ));
}

/// `typeof(v)`: the type of `v`. The runtime checks the number of arguments before calling it.
fn builtin_typeof(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Type(args[0].type_of()?))
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
    /// Looks up `name` as a variable, then a global function, then a built-in type name.
    /// So a variable shadows a function (e.g. after `let identity = 1`), and a function
    /// shadows a built-in type name.
    fn lookup(&self, name: &str) -> Option<Value> {
        self.global_scope
            .get_variable(name)
            .cloned()
            .or_else(|| {
                self.global_scope
                    .get_function(name)
                    .cloned()
                    .map(Value::Function)
            })
            .or_else(|| Type::builtin_named(name).map(Value::Type))
    }

    /// Calls the function bound to `name` with already-evaluated arguments. `name` resolves
    /// like an identifier.
    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, Error> {
        match self.lookup(name) {
            Some(Value::Function(function)) => function.call(args),
            Some(_) => Err(Error::RuntimeError(RuntimeError::NotCallable(
                name.to_string(),
            ))),
            None => Err(Error::RuntimeError(RuntimeError::UndefinedFunction(
                name.to_string(),
            ))),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(lit) => Ok(Self::literal_to_value(lit)),
            Expr::Ident(s) => self
                .lookup(s.as_str())
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
                        None => Ok(Value::None),
                    }
                }
            }
            Expr::UnaryExpr(op, inner) => {
                let val = self.eval_expr(inner)?;
                Self::eval_unary(*op, &val)
            }
            // `x |> f(a, ...)` calls `f(x, a, ...)`, and `x |> f` calls `f(x)`. The right
            // side is never evaluated on its own.
            Expr::BinaryExpr(left, BinaryOp::Pipe, right) => {
                let value = self.eval_expr(left)?;
                match right.as_ref() {
                    Expr::FunctionCall(name, args) => {
                        let mut values = vec![value];
                        for arg in args {
                            values.push(self.eval_expr(arg)?);
                        }
                        self.call_function(name.0.as_str(), &values)
                    }
                    Expr::Ident(name) => self.call_function(name.as_str(), &[value]),
                    other => Err(Error::RuntimeError(RuntimeError::InvalidPipeTarget(
                        other.to_string(),
                    ))),
                }
            }
            Expr::BinaryExpr(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                Self::eval_binary(&l, *op, &r)
            }
            Expr::FunctionCall(name, args) => {
                let arg_values = args
                    .iter()
                    .map(|arg| self.eval_expr(arg))
                    .collect::<Result<Vec<Value>, Error>>()?;
                self.call_function(name.0.as_str(), &arg_values)
            }
            Expr::Parenthesized(inner) => self.eval_expr(inner),
            Expr::FieldAccess(inner, field) => {
                let value = self.eval_expr(inner)?;
                match value {
                    Value::Map(m) => m.get(field).cloned().ok_or_else(|| {
                        Error::RuntimeError(RuntimeError::NoSuchField(field.clone()))
                    }),
                    _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
                }
            }
            Expr::MapLiteral(entries) => {
                let mut map = HashMap::new();
                for kv in entries {
                    let value = self.eval_expr(&kv.value)?;
                    map.insert(kv.key.clone(), value);
                }
                Ok(Value::Map(map))
            }
            Expr::ArrayLiteral(elems) => {
                let values = elems
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<Vec<Value>, Error>>()?;
                Ok(Value::Array(values))
            }
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
            BinaryOp::Eq => Ok(Value::Boolean(Self::values_equal(l, r))),
            BinaryOp::Ne => Ok(Value::Boolean(!Self::values_equal(l, r))),
            BinaryOp::Lt => Self::binary_compare(l, r, |a, b| a < b),
            BinaryOp::Gt => Self::binary_compare(l, r, |a, b| a > b),
            BinaryOp::Lte => Self::binary_compare(l, r, |a, b| a <= b),
            BinaryOp::Gte => Self::binary_compare(l, r, |a, b| a >= b),
            BinaryOp::Pipe => unreachable!("pipes are evaluated by eval_expr"),
        }
    }

    /// Equality for `==` and `!=`. `None` is both a value and a type, so the `None`
    /// value equals the `None` type (e.g. `typeof(None) == None`).
    fn values_equal(l: &Value, r: &Value) -> bool {
        match (l, r) {
            (Value::None, Value::Type(t)) | (Value::Type(t), Value::None) => *t == Type::none(),
            _ => l == r,
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
    pub functions: HashMap<String, Arc<NativeFunction>>,
    pub variables: HashMap<String, Value>,
}

impl GlobalScope {
    /// Check if the global scope has a function with the given name.
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get a function from the global scope by name.
    pub fn get_function(&self, name: &str) -> Option<&Arc<NativeFunction>> {
        self.functions.get(name)
    }

    /// Register a native function in the global scope under its name, replacing any
    /// function already registered under that name.
    pub fn register_function(&mut self, function: NativeFunction) {
        self.functions
            .insert(function.name().to_string(), Arc::new(function));
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

    /// Remove a variable from the global scope by name, returning its previous value if bound.
    pub fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;

    use super::*;
    use crate::ast::Identifier;
    use crate::parser::parse_statement;

    /// Evaluates a single expression statement in a fresh runtime.
    fn eval(expr: Expr) -> Value {
        SimpleRuntime::default()
            .eval(&Statement::Expr(expr))
            .unwrap()
    }

    #[test]
    fn test_type_of_none() {
        assert_eq!(Value::None.type_of().unwrap(), Type::none());
    }

    #[test]
    fn test_none_value_display() {
        assert_eq!(Value::None.to_string(), "None");
    }

    #[test]
    fn test_if_without_else_evaluates_to_then_branch_when_true() {
        let expr = Expr::if_expr(Expr::literal_boolean(true), Expr::literal_number(1), None);
        assert_eq!(eval(expr), Value::Number(BigDecimal::from(1)));
    }

    #[test]
    fn test_if_without_else_evaluates_to_none_when_false() {
        let expr = Expr::if_expr(Expr::literal_boolean(false), Expr::literal_number(1), None);
        assert_eq!(eval(expr), Value::None);
    }

    #[test]
    fn test_typeof_if_without_else_is_none_when_false() {
        let expr = Expr::function_call(
            "typeof",
            vec![Expr::if_expr(
                Expr::literal_boolean(false),
                Expr::literal_number(1),
                None,
            )],
        );
        assert_eq!(eval(expr), Value::Type(Type::none()));
    }

    #[test]
    fn test_none_value_equals_none_type() {
        let typeof_none = Expr::function_call("typeof", vec![Expr::literal_none()]);
        let expr = Expr::binary_expr(typeof_none.clone(), BinaryOp::Eq, Expr::literal_none());
        assert_eq!(eval(expr), Value::Boolean(true));

        let expr = Expr::binary_expr(Expr::literal_none(), BinaryOp::Eq, typeof_none);
        assert_eq!(eval(expr), Value::Boolean(true));
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
    fn test_type_of_type_is_type() {
        assert_eq!(
            Value::Type(Type::number()).type_of().unwrap(),
            Type::type_type()
        );
    }

    #[test]
    fn test_builtin_type_name_evaluates_to_type() {
        assert_eq!(eval(Expr::ident("Number")), Value::Type(Type::number()));
        assert_eq!(eval(Expr::ident("Type")), Value::Type(Type::type_type()));
    }

    #[test]
    fn test_typeof_equals_builtin_type_name() {
        let typeof_one = Expr::function_call("typeof", vec![Expr::literal_number(1)]);
        let expr = Expr::binary_expr(typeof_one, BinaryOp::Eq, Expr::ident("Number"));
        assert_eq!(eval(expr), Value::Boolean(true));
    }

    #[test]
    fn test_typeof_builtin_type_name_is_type() {
        let expr = Expr::function_call("typeof", vec![Expr::ident("Number")]);
        assert_eq!(eval(expr), Value::Type(Type::type_type()));
    }

    #[test]
    fn test_let_shadows_builtin_type_name() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .eval(&Statement::Let(
                Identifier("Number".to_string()),
                Expr::literal_number(5),
            ))
            .unwrap();
        let value = runtime
            .eval(&Statement::Expr(Expr::ident("Number")))
            .unwrap();
        assert_eq!(value, Value::Number(BigDecimal::from(5)));
    }

    #[test]
    fn test_any_is_unbound() {
        let result = SimpleRuntime::default().eval(&Statement::Expr(Expr::ident("Any")));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UnboundIdentifier(name))) if name == "Any"
        ));
    }

    #[test]
    fn test_typeof_signature() {
        let runtime = SimpleRuntime::default();
        let function = runtime
            .global_scope
            .get_function("typeof")
            .expect("typeof should be registered in the global scope");
        let definition = &function.definition;
        assert_eq!(definition.overloads[0].return_type, Type::type_type());
        assert_eq!(definition.overloads[0].arguments[0].r#type, Type::any());
    }

    #[test]
    fn test_value_type_display() {
        assert_eq!(Value::Type(Type::number()).to_string(), "Number");
        assert_eq!(
            Value::Type(Type::option(Type::string())).to_string(),
            "Option<String>"
        );
        assert_eq!(
            Value::Type(Type::union(Type::number(), Type::none())).to_string(),
            "Number | None"
        );
    }

    #[test]
    fn test_builtin_typeof_returns_type() {
        let result = builtin_typeof(&[Value::Number(BigDecimal::from(42))]).unwrap();
        assert_eq!(result, Value::Type(Type::number()));
    }

    #[test]
    fn test_builtin_typeof_none() {
        let result = builtin_typeof(&[Value::None]).unwrap();
        assert_eq!(result, Value::Type(Type::none()));
    }

    #[test]
    fn test_typeof_arity_checked_by_runtime() {
        let result = SimpleRuntime::default().eval(&call("typeof", vec![]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch {
                expected: 1,
                found: 0,
                ..
            }))
        ));
    }

    #[test]
    fn test_typeof_registered_in_global_scope() {
        let runtime = SimpleRuntime::default();
        assert!(runtime.global_scope.get_function("typeof").is_some());
    }

    #[test]
    fn test_typeof_evaluates_to_function_value() {
        let value = eval(Expr::ident("typeof"));
        assert!(matches!(&value, Value::Function(function) if function.name() == "typeof"));
        assert_eq!(value.to_string(), "<fn typeof>");
    }

    #[test]
    fn test_typeof_of_typeof() {
        let value = eval(Expr::function_call("typeof", vec![Expr::ident("typeof")]));
        assert!(matches!(&value, Value::Type(Type::Function(_))));
        assert_eq!(value.to_string(), "fn(Any) -> Type");
    }

    #[test]
    fn test_call_typeof_through_another_name() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .eval(&let_statement("t", Expr::ident("typeof")))
            .unwrap();
        let value = runtime
            .eval(&call("t", vec![Expr::literal_number(42)]))
            .unwrap();
        assert_eq!(value, Value::Type(Type::number()));
    }

    #[test]
    fn test_removing_shadowing_variable_restores_typeof() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .eval(&let_statement("typeof", Expr::literal_number(1)))
            .unwrap();
        runtime.global_scope.remove_variable("typeof");
        let value = runtime
            .eval(&call("typeof", vec![Expr::literal_number(1)]))
            .unwrap();
        assert_eq!(value, Value::Type(Type::number()));
    }

    #[test]
    fn test_host_function_replaces_typeof() {
        let mut runtime = SimpleRuntime::default();
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new(
                "typeof",
                Type::string(),
                vec![Argument::new("v", Type::any())],
            ),
            |_args| Ok(Value::String("mine".to_string())),
        ));
        let value = runtime
            .eval(&call("typeof", vec![Expr::literal_number(1)]))
            .unwrap();
        assert_eq!(value, Value::String("mine".to_string()));
        let function = runtime.global_scope.get_function("typeof").unwrap();
        assert_eq!(function.r#type().to_string(), "fn(Any) -> String");
    }

    /// A fresh runtime with a native `identity` function of type `fn(Any) -> Any`.
    fn runtime_with_identity() -> SimpleRuntime {
        let mut runtime = SimpleRuntime::default();
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::any(),
                vec![Argument::new("v", Type::any())],
            ),
            |args| Ok(args[0].clone()),
        ));
        runtime
    }

    fn let_statement(name: &str, expr: Expr) -> Statement {
        Statement::Let(Identifier(name.to_string()), expr)
    }

    fn call(name: &str, args: Vec<Expr>) -> Statement {
        Statement::Expr(Expr::function_call(name, args))
    }

    #[test]
    fn test_registered_function_evaluates_to_function_value() {
        let mut runtime = runtime_with_identity();
        let value = runtime
            .eval(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert!(matches!(&value, Value::Function(function) if function.name() == "identity"));
        assert_eq!(value.to_string(), "<fn identity>");
        assert_eq!(value.type_of().unwrap().to_string(), "fn(Any) -> Any");
    }

    #[test]
    fn test_call_registered_function() {
        let mut runtime = runtime_with_identity();
        let value = runtime
            .eval(&call("identity", vec![Expr::literal_number(42)]))
            .unwrap();
        assert_eq!(value, Value::Number(BigDecimal::from(42)));
    }

    #[test]
    fn test_call_function_through_another_name() {
        let mut runtime = runtime_with_identity();
        runtime
            .eval(&let_statement("f", Expr::ident("identity")))
            .unwrap();
        let value = runtime
            .eval(&call("f", vec![Expr::literal_boolean(true)]))
            .unwrap();
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn test_function_value_equals_itself() {
        let mut runtime = runtime_with_identity();
        let a = runtime
            .eval(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        let b = runtime
            .eval(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_variable_shadows_function() {
        let mut runtime = runtime_with_identity();
        runtime
            .eval(&let_statement("identity", Expr::literal_number(1)))
            .unwrap();
        let value = runtime
            .eval(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert_eq!(value, Value::Number(BigDecimal::from(1)));
    }

    #[test]
    fn test_function_shadows_builtin_type_name() {
        let mut runtime = SimpleRuntime::default();
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new("Number", Type::number(), vec![]),
            |_args| Ok(Value::Number(BigDecimal::from(0))),
        ));
        let value = runtime
            .eval(&Statement::Expr(Expr::ident("Number")))
            .unwrap();
        assert!(matches!(value, Value::Function(_)));
    }

    #[test]
    fn test_call_checks_arity() {
        let mut runtime = runtime_with_identity();
        let result = runtime.eval(&call(
            "identity",
            vec![Expr::literal_number(1), Expr::literal_number(2)],
        ));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected: 1, found: 2 }))
                if name == "identity"
        ));
    }

    #[test]
    fn test_call_non_function_is_not_callable() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .eval(&let_statement("x", Expr::literal_number(1)))
            .unwrap();
        let result = runtime.eval(&call("x", vec![Expr::literal_number(2)]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "x"
        ));
    }

    #[test]
    fn test_call_builtin_type_name_is_not_callable() {
        let result = SimpleRuntime::default().eval(&call("Number", vec![]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "Number"
        ));
    }

    #[test]
    fn test_call_unknown_function_is_undefined() {
        let result = SimpleRuntime::default().eval(&call("nope", vec![]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UndefinedFunction(name))) if name == "nope"
        ));
    }

    /// A fresh runtime with `identity`, a native `sub(a, b)` returning `a - b`, a native
    /// `args(a, b, c)` returning its arguments as an array, and a native `next()` returning
    /// 1, 2, 3, ... on successive calls.
    fn runtime_for_pipes() -> SimpleRuntime {
        let mut runtime = runtime_with_identity();
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new(
                "sub",
                Type::number(),
                vec![Argument::number("a"), Argument::number("b")],
            ),
            |args| SimpleRuntime::binary_number(&args[0], &args[1], |a, b| a - b),
        ));
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new(
                "args",
                Type::array(),
                vec![
                    Argument::new("a", Type::any()),
                    Argument::new("b", Type::any()),
                    Argument::new("c", Type::any()),
                ],
            ),
            |args| Ok(Value::Array(args.to_vec())),
        ));
        let calls = Arc::new(AtomicU64::new(0));
        runtime.global_scope.register_function(NativeFunction::new(
            FunctionDefinition::new("next", Type::number(), vec![]),
            move |_args| {
                Ok(Value::Number(BigDecimal::from(
                    calls.fetch_add(1, Ordering::SeqCst) + 1,
                )))
            },
        ));
        runtime
    }

    /// Parses and evaluates each `;`-separated statement in `input`, returning the last result.
    fn eval_source(runtime: &mut SimpleRuntime, input: &str) -> Result<Value, Error> {
        let mut last = Ok(Value::None);
        for source in input.split(';') {
            let (rest, statement) = parse_statement(source).expect("input should parse");
            assert_eq!(rest, "", "unparsed input in {source:?}");
            last = Ok(runtime.eval(&statement)?);
        }
        last
    }

    fn number(n: i64) -> Value {
        Value::Number(BigDecimal::from(n))
    }

    /// `f 1`, `f(1)`, `1 |> f` and `1 |> f()` all evaluate as `f(1)`, for native and
    /// built-in functions alike.
    #[test]
    fn test_call_and_pipe_forms_are_equivalent() {
        for (name, expected) in [
            ("identity", number(1)),
            ("typeof", Value::Type(Type::number())),
        ] {
            for form in [
                format!("{name} 1"),
                format!("{name}(1)"),
                format!("1 |> {name}"),
                format!("1 |> {name}()"),
            ] {
                let mut runtime = runtime_for_pipes();
                assert_eq!(
                    eval_source(&mut runtime, &form).unwrap(),
                    expected,
                    "evaluating {form:?}"
                );
            }
        }
        // with more arguments, and through another name
        let mut runtime = runtime_for_pipes();
        for form in ["sub 10, 3", "sub(10, 3)", "10 |> sub(3)"] {
            assert_eq!(eval_source(&mut runtime, form).unwrap(), number(7));
        }
        eval_source(&mut runtime, "let g = identity").unwrap();
        for form in ["g 1", "g(1)", "1 |> g", "1 |> g()"] {
            assert_eq!(eval_source(&mut runtime, form).unwrap(), number(1));
        }
    }

    #[test]
    fn test_pipe_passes_left_side_as_first_argument() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(
            eval_source(&mut runtime, "10 |> sub(3)").unwrap(),
            number(7)
        );
    }

    #[test]
    fn test_pipe_into_bare_name() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(
            eval_source(&mut runtime, "1 |> typeof").unwrap(),
            Value::Type(Type::number())
        );
        assert_eq!(
            eval_source(&mut runtime, "let g = identity; 42 |> g").unwrap(),
            number(42)
        );
    }

    #[test]
    fn test_pipe_keeps_keyword_arguments_last() {
        let mut runtime = runtime_for_pipes();
        let value = eval_source(&mut runtime, "1 |> args(2, k: 3)").unwrap();
        assert_eq!(
            value,
            Value::Array(vec![
                number(1),
                number(2),
                Value::Map(HashMap::from([("k".to_string(), number(3))])),
            ])
        );
    }

    #[test]
    fn test_pipe_chain_passes_previous_result_first() {
        let mut runtime = runtime_for_pipes();
        // sub(sub(10, 3), 2)
        assert_eq!(
            eval_source(&mut runtime, "10 |> sub(3) |> sub(2)").unwrap(),
            number(5)
        );
        // sub(identity(10), 1)
        assert_eq!(
            eval_source(&mut runtime, "10 |> identity |> sub(1)").unwrap(),
            number(9)
        );
    }

    #[test]
    fn test_pipe_evaluates_left_side_first() {
        let mut runtime = runtime_for_pipes();
        // sub(1, 2), not sub(2, 1)
        assert_eq!(
            eval_source(&mut runtime, "next() |> sub(next())").unwrap(),
            number(-1)
        );
    }

    #[test]
    fn test_pipe_right_side_is_not_evaluated_on_its_own() {
        let mut runtime = runtime_for_pipes();
        assert!(eval_source(&mut runtime, "typeof()").is_err());
        assert_eq!(
            eval_source(&mut runtime, "1 |> typeof()").unwrap(),
            Value::Type(Type::number())
        );
    }

    #[test]
    fn test_pipe_argument_count_includes_piped_value() {
        let mut runtime = runtime_for_pipes();
        let result = eval_source(&mut runtime, "1 |> typeof(2)");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected: 1, found: 2 }))
                if name == "typeof"
        ));
    }

    #[test]
    fn test_pipe_into_type_name_is_not_callable() {
        let mut runtime = runtime_for_pipes();
        let result = eval_source(&mut runtime, "1 |> Number");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "Number"
        ));
    }

    #[test]
    fn test_pipe_into_unknown_function_is_undefined() {
        let mut runtime = runtime_for_pipes();
        let result = eval_source(&mut runtime, "1 |> nope()");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UndefinedFunction(name))) if name == "nope"
        ));
    }

    #[test]
    fn test_pipe_into_non_call_is_invalid() {
        let mut runtime = runtime_for_pipes();
        let result = eval_source(&mut runtime, "1 |> 2");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidPipeTarget(target))) if target == "2"
        ));
        let result = eval_source(&mut runtime, "10 |> sub(3) * 2");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidPipeTarget(target))) if target == "sub(3) * 2"
        ));
    }

    #[test]
    fn test_pipe_groups_with_other_operators() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(
            eval_source(&mut runtime, "1 |> typeof == Number").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            eval_source(&mut runtime, "1 + 1 |> sub(2)").unwrap(),
            number(0)
        );
        assert_eq!(
            eval_source(&mut runtime, "1 + 2 == 3").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            eval_source(&mut runtime, r#""a" <> "b" == "ab""#).unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_variable_shadows_builtin_function_in_calls() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .eval(&let_statement("typeof", Expr::literal_number(1)))
            .unwrap();
        let result = runtime.eval(&call("typeof", vec![Expr::literal_number(2)]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "typeof"
        ));
    }
}
