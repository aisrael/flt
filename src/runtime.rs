//! Runtimes for the `flt` language

pub mod functions;
pub mod native;
pub mod types;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use bigdecimal::Zero;
use indexmap::IndexMap;

use crate::ast::BinaryOp;
use crate::ast::Expr;
use crate::ast::Literal;
use crate::ast::Statement;
use crate::ast::UnaryOp;
use crate::errors::ProtectedName;
use crate::errors::RuntimeError;
use crate::parser::parse_statements;
use crate::runtime::functions::Argument;
use crate::runtime::functions::FunctionDefinition;
use crate::runtime::functions::NativeFunction;
use crate::runtime::native::NativeType;
use crate::runtime::native::NativeTypeInfo;
use crate::runtime::native::NativeValue;
use crate::runtime::types::Type;
use crate::utils::escape_string;
use crate::utils::is_bare_name;
use crate::Error;

/// A map key. Keys written in flt source are symbols; only a host can create string keys. A
/// symbol and a string with the same text are different keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MapKey {
    Symbol(String),
    String(String),
}

impl MapKey {
    /// A symbol key, e.g. `:b` in `{b: 1}`
    pub fn symbol<S: Into<String>>(name: S) -> Self {
        MapKey::Symbol(name.into())
    }

    /// A string key, which flt source can't write yet
    pub fn string<S: Into<String>>(s: S) -> Self {
        MapKey::String(s.into())
    }
}

/// A key as the flt value it stands for: a `Symbol` or a `String`
impl From<MapKey> for Value {
    fn from(key: MapKey) -> Self {
        match key {
            MapKey::Symbol(name) => Value::Symbol(name),
            MapKey::String(s) => Value::String(s),
        }
    }
}

/// A map of keys to values, in insertion order. Equality ignores order.
pub type Map = IndexMap<MapKey, Value>;

/// Writes `name` bare if it's a valid bare name, and quoted otherwise.
fn write_name(f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
    if is_bare_name(name) {
        write!(f, "{name}")
    } else {
        write!(f, "\"{}\"", escape_string(name))
    }
}

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
    /// A map of keys to values, in insertion order
    Map(Map),
    /// An array of values
    Array(Vec<Value>),
    /// A type as a first-class value
    Type(Type),
    /// A function as a first-class value
    Function(Arc<NativeFunction>),
    /// A value of a native type, created by the host
    Native(NativeValue),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Symbol(s) => {
                write!(f, ":")?;
                write_name(f, s)
            }
            // Symbol keys print as written in a literal, e.g. `{b: 1, "c d": 2}`. String keys
            // print as `"b" => 1`, which flt can't parse yet.
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match k {
                        MapKey::Symbol(name) => {
                            write_name(f, name)?;
                            write!(f, ": {v}")?;
                        }
                        MapKey::String(s) => write!(f, "\"{}\" => {v}", escape_string(s))?,
                    }
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
            Value::Native(native) => write!(f, "{native}"),
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
            Value::Native(native) => Ok(Type::Native(native.type_info())),
        }
    }

    /// Wraps a host value as an flt value of its native type.
    pub fn native<T: NativeType>(value: T) -> Self {
        Value::Native(NativeValue::new(value))
    }

    /// Borrows the host value inside, if this is a native value of type `T`.
    pub fn get_native<T: NativeType>(&self) -> Option<&T> {
        match self {
            Value::Native(native) => native.get::<T>(),
            _ => None,
        }
    }

    /// Borrows the host value inside, if this is a native value of type `T`, and raises
    /// `Expected <T::NAME>, found <type>` otherwise.
    pub fn downcast_native<T: NativeType>(&self) -> Result<&T, Error> {
        self.get_native::<T>()
            .ok_or_else(|| unexpected_type(T::NAME, self))
    }

    /// The number inside, or `Expected Number, found <type>`.
    fn expect_number(&self) -> Result<&BigDecimal, Error> {
        match self {
            Value::Number(n) => Ok(n),
            other => Err(unexpected_type(&Type::number().to_string(), other)),
        }
    }
}

/// The error for a native function given `value` where it expected a value of type `expected`.
fn unexpected_type(expected: &str, value: &Value) -> Error {
    match value.type_of() {
        Ok(found) => Error::RuntimeError(RuntimeError::UnexpectedType {
            expected: expected.to_string(),
            found: found.to_string(),
        }),
        Err(error) => error,
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(BigDecimal::from(n))
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(BigDecimal::from(n))
    }
}

/// `BigDecimal` has no `From<usize>`, so this goes through `u64`.
impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::from(n as u64)
    }
}

/// Implements `TryFrom<&Value>` for Rust integer types: the value must be a whole `Number` that
/// fits the type, or it raises `Expected Number, found <type>` or `Cannot convert <n> to <type>`.
macro_rules! try_from_value_for_integers {
    ($($t:ty),*) => {$(
        impl TryFrom<&Value> for $t {
            type Error = Error;

            fn try_from(value: &Value) -> Result<Self, Error> {
                let n = value.expect_number()?;
                Some(n)
                    .filter(|n| n.is_integer())
                    .and_then(|n| n.to_i128())
                    .and_then(|n| <$t>::try_from(n).ok())
                    .ok_or_else(|| cannot_convert(n, stringify!($t)))
            }
        }
    )*};
}

try_from_value_for_integers!(i32, i64, u32, u64, usize);

/// Any `Number` converts to an `f64`, possibly losing precision.
impl TryFrom<&Value> for f64 {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let n = value.expect_number()?;
        n.to_f64().ok_or_else(|| cannot_convert(n, "f64"))
    }
}

/// The error for a number that doesn't fit the Rust number type `target`.
fn cannot_convert(n: &BigDecimal, target: &'static str) -> Error {
    Error::RuntimeError(RuntimeError::CannotConvert {
        number: n.to_string(),
        target,
    })
}

/// flt's runtime API: evaluating flt source, registering native functions and native types,
/// and reading and binding global variables. Every runtime implements it, and a host
/// works with a runtime only through it.
///
/// The trait defines operations, not rules. Which names a runtime refuses to bind or register
/// is up to the runtime: [`Runtime::set_variable`] and [`Runtime::check_bindable`] return an
/// error for a name it won't bind, and the `register_*` methods for a name it won't register.
pub trait Runtime {
    /// Parses all of `source` with [`parse_statements`], then evaluates its statements in
    /// order and returns the last one's value, or `None` if there are none.
    ///
    /// Statements are separated by newlines or by `;`. A syntax error, including two
    /// statements on one line with no `;` between them, is an [`Error::ParserError`], and no
    /// statement is evaluated.
    fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let mut value = Value::None;
        for statement in parse_statements(source)? {
            value = self.eval_statement(&statement)?;
        }
        Ok(value)
    }

    /// Evaluates one statement, already parsed.
    fn eval_statement(&mut self, statement: &Statement) -> Result<Value, Error>;

    /// Registers a native function under its name, replacing any function already registered
    /// under that name, unless the runtime refuses the name.
    fn register_function(&mut self, function: NativeFunction) -> Result<(), Error>;

    /// Registers a native type, so that its name evaluates to the type, unless the runtime
    /// refuses the name. Unlike [`Runtime::register_native_type`], it can be called through
    /// `dyn Runtime`.
    fn register_native_type_info(&mut self, info: NativeTypeInfo) -> Result<(), Error>;

    /// Registers the native type `T`, unless the runtime refuses its name.
    fn register_native_type<T: NativeType>(&mut self) -> Result<(), Error>
    where
        Self: Sized,
    {
        self.register_native_type_info(NativeTypeInfo::of::<T>())
    }

    /// A registered native type, by name.
    fn get_native_type(&self, name: &str) -> Option<NativeTypeInfo>;

    /// Binds the global variable `name` to `value`, unless the runtime refuses the name.
    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), Error>;

    /// The value of the global variable `name`.
    fn get_variable(&self, name: &str) -> Option<Value>;

    /// Unbinds the global variable `name`, returning its value if it was bound.
    fn remove_variable(&mut self, name: &str) -> Option<Value>;

    /// Checks that `name` can be bound as a variable, returning the error binding it would
    /// raise if it can't.
    fn check_bindable(&self, name: &str) -> Result<(), Error>;

    /// The global function `name`.
    fn get_function(&self, name: &str) -> Option<Arc<NativeFunction>>;
}

/// The reference runtime. Create one with `SimpleRuntime::default()`, which registers the
/// built-in functions, and work with it through the [`Runtime`] trait.
///
/// A global name is a variable, a function, a built-in type or a native type, never more than
/// one:
///
/// - [`Runtime::set_variable`] refuses a built-in type name, a function's name and a native
///   type's name, as `let` does.
/// - [`Runtime::register_function`] refuses a built-in type name, a native type's name and a
///   variable's name, with `Cannot register function <name>: …`.
/// - [`Runtime::register_native_type_info`] refuses a built-in type name, a function's name, a
///   variable's name and an already registered native type's name, with
///   `Cannot register native type <name>: …`.
///
/// Register native types before the functions that use them, and both before evaluating any
/// flt source.
pub struct SimpleRuntime {
    global_scope: GlobalScope,
    /// Native types registered by the host, by name
    native_types: HashMap<&'static str, NativeTypeInfo>,
}

impl Default for SimpleRuntime {
    /// A runtime with the built-in native functions (`typeof`) registered in its global scope.
    fn default() -> Self {
        let mut global_scope = GlobalScope::default();
        register_builtins(&mut global_scope);
        Self {
            global_scope,
            native_types: HashMap::new(),
        }
    }
}

/// Registers the built-in native functions in `global_scope`.
fn register_builtins(global_scope: &mut GlobalScope) {
    global_scope
        .register_function(NativeFunction::new(
            FunctionDefinition::new(
                "typeof",
                Type::type_type(),
                vec![Argument::new("V", Type::any())],
            ),
            builtin_typeof,
        ))
        .expect("a new global scope has no names in use");
}

/// `typeof(v)`: the type of `v`. The runtime checks the number of arguments before calling it.
fn builtin_typeof(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::Type(args[0].type_of()?))
}

impl Runtime for SimpleRuntime {
    fn eval_statement(&mut self, statement: &Statement) -> Result<Value, Error> {
        match statement {
            Statement::Expr(expr) => self.eval_expr(expr),
            // The name is checked first, so `let Number = f()` doesn't call `f`.
            Statement::Let(ident, expr) => {
                let name = ident.0.as_str();
                self.global_scope.check_bindable(name)?;
                let value = self.eval_expr(expr)?;
                self.global_scope.set_variable(name, value.clone())?;
                Ok(value)
            }
        }
    }

    fn register_function(&mut self, function: NativeFunction) -> Result<(), Error> {
        self.global_scope.register_function(function)
    }

    fn register_native_type_info(&mut self, info: NativeTypeInfo) -> Result<(), Error> {
        let name = info.name();
        let reason = if Type::is_builtin_name(name) {
            ProtectedName::BuiltinType
        } else if self.global_scope.has_function(name) {
            ProtectedName::Function
        } else if self.global_scope.has_variable(name) {
            ProtectedName::Variable
        } else if self.native_types.contains_key(name) {
            ProtectedName::NativeType
        } else {
            self.native_types.insert(name, info);
            self.global_scope.protect_native_type_name(name);
            return Ok(());
        };
        Err(cannot_register("native type", name, reason))
    }

    fn get_native_type(&self, name: &str) -> Option<NativeTypeInfo> {
        self.native_types.get(name).copied()
    }

    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), Error> {
        self.global_scope.set_variable(name, value)
    }

    fn get_variable(&self, name: &str) -> Option<Value> {
        self.global_scope.get_variable(name).cloned()
    }

    fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.global_scope.remove_variable(name)
    }

    fn check_bindable(&self, name: &str) -> Result<(), Error> {
        self.global_scope.check_bindable(name)
    }

    fn get_function(&self, name: &str) -> Option<Arc<NativeFunction>> {
        self.global_scope.get_function(name).cloned()
    }
}

impl SimpleRuntime {
    /// Looks up `name` as a variable, then a global function, then a built-in type name, then
    /// a native type name. A name is never more than one of these, since a binding can't
    /// shadow a function or a type, so the order doesn't change the result.
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
            .or_else(|| {
                self.get_native_type(name)
                    .map(|info| Value::Type(Type::Native(info)))
            })
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
                    Value::Map(m) => m.get(&MapKey::symbol(field)).cloned().ok_or_else(|| {
                        Error::RuntimeError(RuntimeError::NoSuchField(field.clone()))
                    }),
                    _ => Err(Error::RuntimeError(RuntimeError::InvalidOperandType)),
                }
            }
            Expr::MapLiteral(entries) => {
                let mut map = Map::new();
                for kv in entries {
                    let value = self.eval_expr(&kv.value)?;
                    map.insert(MapKey::symbol(kv.key.as_str()), value);
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

/// The error for registering a `kind` of thing, a function or a native type, under `name`,
/// which is already `reason`.
fn cannot_register(kind: &'static str, name: &str, reason: ProtectedName) -> Error {
    Error::RuntimeError(RuntimeError::CannotRegister {
        kind,
        name: name.to_string(),
        reason,
    })
}

/// The global scope is the scope that is available to all other scopes.
///
/// A name in it is a variable, a function, a built-in type, or a native type registered with
/// the runtime, never more than one: a variable can't use a function's name or a type's name,
/// and a function can't use a type's name or a variable's name.
#[derive(Default)]
pub(crate) struct GlobalScope {
    functions: HashMap<String, Arc<NativeFunction>>,
    variables: HashMap<String, Value>,
    /// The names of native types registered with the runtime, which hold the types themselves
    native_type_names: HashSet<String>,
}

impl GlobalScope {
    /// Check if the global scope has a function with the given name.
    pub(crate) fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get a function from the global scope by name.
    pub(crate) fn get_function(&self, name: &str) -> Option<&Arc<NativeFunction>> {
        self.functions.get(name)
    }

    /// Register a native function in the global scope under its name, replacing any
    /// function already registered under that name, unless the name is a built-in type name,
    /// a native type's name, or is already bound as a variable.
    pub(crate) fn register_function(&mut self, function: NativeFunction) -> Result<(), Error> {
        let name = function.name();
        let reason = if Type::is_builtin_name(name) {
            ProtectedName::BuiltinType
        } else if self.native_type_names.contains(name) {
            ProtectedName::NativeType
        } else if self.variables.contains_key(name) {
            ProtectedName::Variable
        } else {
            self.functions.insert(name.to_string(), Arc::new(function));
            return Ok(());
        };
        Err(cannot_register("function", name, reason))
    }

    /// Check if the global scope has a variable with the given name.
    pub(crate) fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get a variable from the global scope by name.
    pub(crate) fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Check that `name` can be bound as a variable: it isn't a built-in type name, a
    /// function's name or a native type's name.
    pub(crate) fn check_bindable(&self, name: &str) -> Result<(), Error> {
        let reason = if Type::is_builtin_name(name) {
            ProtectedName::BuiltinType
        } else if self.has_function(name) {
            ProtectedName::Function
        } else if self.native_type_names.contains(name) {
            ProtectedName::NativeType
        } else {
            return Ok(());
        };
        Err(Error::RuntimeError(RuntimeError::CannotBind {
            name: name.to_string(),
            reason,
        }))
    }

    /// Set a variable in the global scope by name, unless the name is a built-in type name, a
    /// function's name or a native type's name.
    pub(crate) fn set_variable(&mut self, name: &str, value: Value) -> Result<(), Error> {
        self.check_bindable(name)?;
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Remove a variable from the global scope by name, returning its previous value if bound.
    pub(crate) fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// Records that `name` is a native type registered with the runtime, so it can't be bound
    /// as a variable or used for a function.
    pub(crate) fn protect_native_type_name(&mut self, name: &str) {
        self.native_type_names.insert(name.to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;

    use std::str::FromStr;

    use super::*;
    use crate::ast::Identifier;
    use crate::parser::parse_statement;
    use crate::runtime::native::fixtures::Meters;
    use crate::runtime::native::NativeType;
    use crate::runtime::native::NativeTypeInfo;

    /// Evaluates a single expression statement in a fresh runtime.
    fn eval(expr: Expr) -> Value {
        SimpleRuntime::default()
            .eval_statement(&Statement::Expr(expr))
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
        assert_eq!(Value::Map(Map::new()).type_of().unwrap(), Type::map());
    }

    fn map_keys(value: &Value) -> Vec<MapKey> {
        match value {
            Value::Map(map) => map.keys().cloned().collect(),
            other => panic!("expected a map, got {other}"),
        }
    }

    fn symbols(names: &[&str]) -> Vec<MapKey> {
        names.iter().map(|name| MapKey::symbol(*name)).collect()
    }

    #[test]
    fn test_map_literal_keeps_source_order() {
        let mut runtime = SimpleRuntime::default();
        let value = runtime.eval("{z: 1, y: 2, x: 3}").unwrap();
        assert_eq!(map_keys(&value), symbols(&["z", "y", "x"]));
    }

    #[test]
    fn test_map_literal_duplicate_key_keeps_first_position_and_last_value() {
        let mut runtime = SimpleRuntime::default();
        let value = runtime.eval("{n: 1, m: 2, n: 3}").unwrap();
        assert_eq!(map_keys(&value), symbols(&["n", "m"]));
        assert_eq!(value.to_string(), "{n: 3, m: 2}");
    }

    #[test]
    fn test_map_literal_keys_are_symbols() {
        let mut runtime = SimpleRuntime::default();
        let value = runtime.eval(r#"{b: 1, "c d": 2}"#).unwrap();
        assert_eq!(map_keys(&value), symbols(&["b", "c d"]));
        assert_eq!(value, runtime.eval(r#"{"b": 1, "c d": 2}"#).unwrap());
    }

    #[test]
    fn test_bare_and_quoted_keys_are_the_same_key() {
        let mut runtime = SimpleRuntime::default();
        let value = runtime.eval(r#"{b: 1, "b": 2}"#).unwrap();
        assert_eq!(map_keys(&value), symbols(&["b"]));
        assert_eq!(value.to_string(), "{b: 2}");
    }

    #[test]
    fn test_symbol_and_string_keys_differ() {
        let map = Map::from([
            (MapKey::symbol("b"), number(1)),
            (MapKey::string("b"), number(2)),
        ]);
        assert_eq!(map.len(), 2);
        assert_eq!(Value::Map(map).to_string(), r#"{b: 1, "b" => 2}"#);
    }

    #[test]
    fn test_map_prints_keys_as_written() {
        let mut runtime = SimpleRuntime::default();
        let source = r#"{b: 1, "c d": 2, "_x": 3, "1x": 4}"#;
        assert_eq!(runtime.eval(source).unwrap().to_string(), source);
    }

    #[test]
    fn test_symbol_prints_quoted_when_not_a_bare_name() {
        let symbol = |name: &str| Value::Symbol(name.to_string()).to_string();
        assert_eq!(symbol("b"), ":b");
        assert_eq!(symbol("hello world"), r#":"hello world""#);
        assert_eq!(symbol("_x"), r#":"_x""#);
        assert_eq!(symbol(r#"say "hi""#), r#":"say \"hi\"""#);
    }

    #[test]
    fn test_map_key_to_value() {
        assert_eq!(
            Value::from(MapKey::symbol("b")),
            Value::Symbol("b".to_string())
        );
        assert_eq!(
            Value::from(MapKey::string("b")),
            Value::String("b".to_string())
        );
    }

    #[test]
    fn test_field_access_reads_symbol_key() {
        let mut runtime = SimpleRuntime::default();
        assert_eq!(runtime.eval(r#"{"b": 1}.b"#).unwrap(), number(1));
        runtime
            .set_variable(
                "m",
                Value::Map(Map::from([(MapKey::string("b"), number(1))])),
            )
            .unwrap();
        assert!(matches!(
            runtime.eval("m.b"),
            Err(Error::RuntimeError(RuntimeError::NoSuchField(field))) if field == "b"
        ));
    }

    #[test]
    fn test_map_displays_in_insertion_order() {
        let map = Map::from([
            (MapKey::symbol("b"), number(1)),
            (MapKey::symbol("a"), number(2)),
        ]);
        assert_eq!(Value::Map(map).to_string(), "{b: 1, a: 2}");
    }

    #[test]
    fn test_map_equality_ignores_order() {
        let ab = Map::from([
            (MapKey::symbol("a"), number(1)),
            (MapKey::symbol("b"), number(2)),
        ]);
        let ba = Map::from([
            (MapKey::symbol("b"), number(2)),
            (MapKey::symbol("a"), number(1)),
        ]);
        assert_eq!(Value::Map(ab), Value::Map(ba));
    }

    #[test]
    fn test_native_function_sees_keyword_arguments_in_source_order() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new(
                    "keys",
                    Type::array(),
                    vec![Argument::new("map", Type::map())],
                ),
                |args| {
                    Ok(Value::Array(
                        map_keys(&args[0]).into_iter().map(Value::from).collect(),
                    ))
                },
            ))
            .unwrap();
        let value = runtime.eval("keys(z: 1, y: 2, x: 3)").unwrap();
        assert_eq!(value.to_string(), "[:z, :y, :x]");
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

    /// Asserts that `result` is the error for binding the protected name `name`.
    fn assert_cannot_bind<T: std::fmt::Debug>(
        result: Result<T, Error>,
        name: &str,
        reason: ProtectedName,
    ) {
        match result {
            Err(Error::RuntimeError(RuntimeError::CannotBind {
                name: found_name,
                reason: found_reason,
            })) => {
                assert_eq!(found_name, name);
                assert_eq!(found_reason, reason);
            }
            other => panic!("expected CannotBind for {name}, got {other:?}"),
        }
    }

    #[test]
    fn test_let_cannot_bind_builtin_type_name() {
        let mut runtime = SimpleRuntime::default();
        for name in [
            "None", "Boolean", "Number", "String", "Symbol", "Array", "Map", "Type", "Any",
        ] {
            let result = runtime.eval_statement(&Statement::Let(
                Identifier(name.to_string()),
                Expr::literal_number(5),
            ));
            assert_cannot_bind(result, name, ProtectedName::BuiltinType);
        }
        // The refused binding leaves the type in place
        let value = runtime
            .eval_statement(&Statement::Expr(Expr::ident("Number")))
            .unwrap();
        assert_eq!(value, Value::Type(Type::number()));
    }

    #[test]
    fn test_cannot_bind_error_message() {
        let mut runtime = runtime_with_identity();
        let error = runtime
            .eval_statement(&let_statement("Number", Expr::literal_number(5)))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Cannot bind Number: it's a built-in type"
        );
        let error = runtime
            .eval_statement(&let_statement("identity", Expr::literal_number(5)))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Cannot bind identity: it's a function"
        );
    }

    #[test]
    fn test_refused_binding_does_not_evaluate_its_value() {
        let calls = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&calls);
        let mut runtime = SimpleRuntime::default();
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new("tick", Type::number(), vec![]),
                move |_args| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(Value::Number(BigDecimal::from(1)))
                },
            ))
            .unwrap();
        let result = runtime.eval_statement(&let_statement(
            "Number",
            Expr::function_call("tick", vec![]),
        ));
        assert_cannot_bind(result, "Number", ProtectedName::BuiltinType);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_set_variable_refuses_protected_names() {
        let mut runtime = runtime_with_identity();
        runtime.register_native_type::<Meters>().unwrap();
        let runtime: &mut dyn Runtime = &mut runtime;
        for (name, reason) in [
            ("Number", ProtectedName::BuiltinType),
            ("typeof", ProtectedName::Function),
            ("identity", ProtectedName::Function),
            ("Meters", ProtectedName::NativeType),
        ] {
            assert_cannot_bind(runtime.set_variable(name, Value::None), name, reason);
            assert_cannot_bind(runtime.check_bindable(name), name, reason);
            assert_eq!(runtime.get_variable(name), None);
        }
        assert!(runtime.check_bindable("x").is_ok());
        assert!(runtime.set_variable("x", Value::None).is_ok());
        assert_eq!(runtime.get_variable("x"), Some(Value::None));
    }

    #[test]
    fn test_scope_without_builtins_protects_only_type_names() {
        let mut scope = GlobalScope::default();
        assert_cannot_bind(
            scope.set_variable("Number", Value::None),
            "Number",
            ProtectedName::BuiltinType,
        );
        assert!(scope.set_variable("typeof", Value::None).is_ok());
    }

    #[test]
    fn test_any_is_unbound() {
        let result = SimpleRuntime::default().eval_statement(&Statement::Expr(Expr::ident("Any")));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UnboundIdentifier(name))) if name == "Any"
        ));
    }

    #[test]
    fn test_typeof_signature() {
        let runtime = SimpleRuntime::default();
        let function = runtime
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
        let result = SimpleRuntime::default().eval_statement(&call("typeof", vec![]));
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
        assert!(runtime.get_function("typeof").is_some());
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
            .eval_statement(&let_statement("t", Expr::ident("typeof")))
            .unwrap();
        let value = runtime
            .eval_statement(&call("t", vec![Expr::literal_number(42)]))
            .unwrap();
        assert_eq!(value, Value::Type(Type::number()));
    }

    #[test]
    fn test_host_function_replaces_typeof() {
        let mut runtime = SimpleRuntime::default();
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new(
                    "typeof",
                    Type::string(),
                    vec![Argument::new("v", Type::any())],
                ),
                |_args| Ok(Value::String("mine".to_string())),
            ))
            .unwrap();
        let value = runtime
            .eval_statement(&call("typeof", vec![Expr::literal_number(1)]))
            .unwrap();
        assert_eq!(value, Value::String("mine".to_string()));
        let function = runtime.get_function("typeof").unwrap();
        assert_eq!(function.r#type().to_string(), "fn(Any) -> String");
        // The replacement is a function too, so its name is still protected
        let result = runtime.eval_statement(&let_statement("typeof", Expr::literal_number(1)));
        assert_cannot_bind(result, "typeof", ProtectedName::Function);
    }

    /// A fresh runtime with a native `identity` function of type `fn(Any) -> Any`.
    fn runtime_with_identity() -> SimpleRuntime {
        let mut runtime = SimpleRuntime::default();
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new(
                    "identity",
                    Type::any(),
                    vec![Argument::new("v", Type::any())],
                ),
                |args| Ok(args[0].clone()),
            ))
            .unwrap();
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
            .eval_statement(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert!(matches!(&value, Value::Function(function) if function.name() == "identity"));
        assert_eq!(value.to_string(), "<fn identity>");
        assert_eq!(value.type_of().unwrap().to_string(), "fn(Any) -> Any");
    }

    #[test]
    fn test_call_registered_function() {
        let mut runtime = runtime_with_identity();
        let value = runtime
            .eval_statement(&call("identity", vec![Expr::literal_number(42)]))
            .unwrap();
        assert_eq!(value, Value::Number(BigDecimal::from(42)));
    }

    #[test]
    fn test_call_function_through_another_name() {
        let mut runtime = runtime_with_identity();
        runtime
            .eval_statement(&let_statement("f", Expr::ident("identity")))
            .unwrap();
        let value = runtime
            .eval_statement(&call("f", vec![Expr::literal_boolean(true)]))
            .unwrap();
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn test_function_value_equals_itself() {
        let mut runtime = runtime_with_identity();
        let a = runtime
            .eval_statement(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        let b = runtime
            .eval_statement(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_let_cannot_bind_host_function() {
        let mut runtime = runtime_with_identity();
        let result = runtime.eval_statement(&let_statement("identity", Expr::literal_number(1)));
        assert_cannot_bind(result, "identity", ProtectedName::Function);
        let value = runtime
            .eval_statement(&Statement::Expr(Expr::ident("identity")))
            .unwrap();
        assert!(matches!(value, Value::Function(_)));
    }

    /// Asserts that `result` is the error for registering a `kind` under `name`, which is
    /// already in use.
    fn assert_cannot_register(
        result: Result<(), Error>,
        kind: &str,
        name: &str,
        reason: ProtectedName,
    ) {
        match result {
            Err(Error::RuntimeError(RuntimeError::CannotRegister {
                kind: found_kind,
                name: found_name,
                reason: found_reason,
            })) => {
                assert_eq!(found_kind, kind);
                assert_eq!(found_name, name);
                assert_eq!(found_reason, reason);
            }
            other => panic!("expected CannotRegister for {kind} {name}, got {other:?}"),
        }
    }

    /// A native function named `name`, of type `fn() -> Number`
    fn zero_function(name: &str) -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new(name, Type::number(), vec![]),
            |_args| Ok(Value::Number(BigDecimal::from(0))),
        )
    }

    #[test]
    fn test_register_function_refuses_builtin_type_name() {
        let mut runtime = SimpleRuntime::default();
        assert_cannot_register(
            runtime.register_function(zero_function("Number")),
            "function",
            "Number",
            ProtectedName::BuiltinType,
        );
        assert!(runtime.get_function("Number").is_none());
    }

    #[test]
    fn test_register_function_refuses_variable_name() {
        let mut runtime = SimpleRuntime::default();
        runtime.eval("let x = 1").unwrap();
        assert_cannot_register(
            runtime.register_function(zero_function("x")),
            "function",
            "x",
            ProtectedName::Variable,
        );
        assert_eq!(runtime.get_variable("x"), Some(number(1)));
    }

    #[test]
    fn test_register_function_replaces_a_function() {
        let mut runtime = SimpleRuntime::default();
        runtime.register_function(zero_function("zero")).unwrap();
        assert!(runtime.register_function(zero_function("zero")).is_ok());
    }

    #[test]
    fn test_cannot_register_error_message() {
        let mut runtime = runtime_with_meters();
        let error = runtime
            .register_function(zero_function("Meters"))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Cannot register function Meters: it's a native type"
        );
        let error = runtime.register_native_type::<Meters>().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Cannot register native type Meters: it's a native type"
        );
    }

    #[test]
    fn test_call_checks_arity() {
        let mut runtime = runtime_with_identity();
        let result = runtime.eval_statement(&call(
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
            .eval_statement(&let_statement("x", Expr::literal_number(1)))
            .unwrap();
        let result = runtime.eval_statement(&call("x", vec![Expr::literal_number(2)]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "x"
        ));
    }

    #[test]
    fn test_call_builtin_type_name_is_not_callable() {
        let result = SimpleRuntime::default().eval_statement(&call("Number", vec![]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "Number"
        ));
    }

    #[test]
    fn test_call_unknown_function_is_undefined() {
        let result = SimpleRuntime::default().eval_statement(&call("nope", vec![]));
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
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new(
                    "sub",
                    Type::number(),
                    vec![Argument::number("a"), Argument::number("b")],
                ),
                |args| SimpleRuntime::binary_number(&args[0], &args[1], |a, b| a - b),
            ))
            .unwrap();
        runtime
            .register_function(NativeFunction::new(
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
            ))
            .unwrap();
        let calls = Arc::new(AtomicU64::new(0));
        runtime
            .register_function(NativeFunction::new(
                FunctionDefinition::new("next", Type::number(), vec![]),
                move |_args| {
                    Ok(Value::Number(BigDecimal::from(
                        calls.fetch_add(1, Ordering::SeqCst) + 1,
                    )))
                },
            ))
            .unwrap();
        runtime
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
                    runtime.eval(&form).unwrap(),
                    expected,
                    "evaluating {form:?}"
                );
            }
        }
        // with more arguments, and through another name
        let mut runtime = runtime_for_pipes();
        for form in ["sub 10, 3", "sub(10, 3)", "10 |> sub(3)"] {
            assert_eq!(runtime.eval(form).unwrap(), number(7));
        }
        runtime.eval("let g = identity").unwrap();
        for form in ["g 1", "g(1)", "1 |> g", "1 |> g()"] {
            assert_eq!(runtime.eval(form).unwrap(), number(1));
        }
    }

    #[test]
    fn test_pipe_passes_left_side_as_first_argument() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(runtime.eval("10 |> sub(3)").unwrap(), number(7));
    }

    #[test]
    fn test_pipe_into_bare_name() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(
            runtime.eval("1 |> typeof").unwrap(),
            Value::Type(Type::number())
        );
        assert_eq!(
            runtime.eval("let g = identity; 42 |> g").unwrap(),
            number(42)
        );
    }

    #[test]
    fn test_pipe_keeps_keyword_arguments_last() {
        let mut runtime = runtime_for_pipes();
        let value = runtime.eval("1 |> args(2, k: 3)").unwrap();
        assert_eq!(
            value,
            Value::Array(vec![
                number(1),
                number(2),
                Value::Map(Map::from([(MapKey::symbol("k"), number(3))])),
            ])
        );
    }

    #[test]
    fn test_pipe_chain_passes_previous_result_first() {
        let mut runtime = runtime_for_pipes();
        // sub(sub(10, 3), 2)
        assert_eq!(runtime.eval("10 |> sub(3) |> sub(2)").unwrap(), number(5));
        // sub(identity(10), 1)
        assert_eq!(runtime.eval("10 |> identity |> sub(1)").unwrap(), number(9));
    }

    #[test]
    fn test_pipe_evaluates_left_side_first() {
        let mut runtime = runtime_for_pipes();
        // sub(1, 2), not sub(2, 1)
        assert_eq!(runtime.eval("next() |> sub(next())").unwrap(), number(-1));
    }

    #[test]
    fn test_pipe_right_side_is_not_evaluated_on_its_own() {
        let mut runtime = runtime_for_pipes();
        assert!(runtime.eval("typeof()").is_err());
        assert_eq!(
            runtime.eval("1 |> typeof()").unwrap(),
            Value::Type(Type::number())
        );
    }

    #[test]
    fn test_pipe_argument_count_includes_piped_value() {
        let mut runtime = runtime_for_pipes();
        let result = runtime.eval("1 |> typeof(2)");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected: 1, found: 2 }))
                if name == "typeof"
        ));
    }

    #[test]
    fn test_pipe_into_type_name_is_not_callable() {
        let mut runtime = runtime_for_pipes();
        let result = runtime.eval("1 |> Number");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "Number"
        ));
    }

    #[test]
    fn test_pipe_into_unknown_function_is_undefined() {
        let mut runtime = runtime_for_pipes();
        let result = runtime.eval("1 |> nope()");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UndefinedFunction(name))) if name == "nope"
        ));
    }

    #[test]
    fn test_pipe_into_non_call_is_invalid() {
        let mut runtime = runtime_for_pipes();
        let result = runtime.eval("1 |> 2");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidPipeTarget(target))) if target == "2"
        ));
        let result = runtime.eval("10 |> sub(3) * 2");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidPipeTarget(target))) if target == "sub(3) * 2"
        ));
    }

    #[test]
    fn test_pipe_groups_with_other_operators() {
        let mut runtime = runtime_for_pipes();
        assert_eq!(
            runtime.eval("1 |> typeof == Number").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(runtime.eval("1 + 1 |> sub(2)").unwrap(), number(0));
        assert_eq!(runtime.eval("1 + 2 == 3").unwrap(), Value::Boolean(true));
        assert_eq!(
            runtime.eval(r#""a" <> "b" == "ab""#).unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_let_cannot_bind_builtin_function() {
        let mut runtime = SimpleRuntime::default();
        let result = runtime.eval_statement(&let_statement("typeof", Expr::literal_number(1)));
        assert_cannot_bind(result, "typeof", ProtectedName::Function);
        let value = runtime
            .eval_statement(&call("typeof", vec![Expr::literal_number(2)]))
            .unwrap();
        assert_eq!(value, Value::Type(Type::number()));
    }

    /// A fresh runtime with the native type `Meters` registered, and a native
    /// `meters(n: Number) -> Meters`.
    fn runtime_with_meters() -> SimpleRuntime {
        let mut runtime = SimpleRuntime::default();
        runtime.register_native_type::<Meters>().unwrap();
        runtime.register_function(meters_function()).unwrap();
        runtime
    }

    /// A native `meters(n: Number) -> Meters`
    fn meters_function() -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new(
                "meters",
                Type::native::<Meters>(),
                vec![Argument::number("n")],
            ),
            |args| Ok(Value::native(Meters(i64::try_from(&args[0])?))),
        )
    }

    #[test]
    fn test_register_function_on_runtime() {
        let runtime = runtime_with_meters();
        let function = runtime.get_function("meters").unwrap();
        assert_eq!(function.r#type().to_string(), "fn(Number) -> Meters");
    }

    #[test]
    fn test_register_function_refuses_native_type_name() {
        let mut runtime = runtime_with_meters();
        assert_cannot_register(
            runtime.register_function(zero_function("Meters")),
            "function",
            "Meters",
            ProtectedName::NativeType,
        );
        assert!(runtime.get_function("Meters").is_none());
    }

    #[test]
    fn test_native_type_name_evaluates_to_type() {
        let mut runtime = runtime_with_meters();
        let value = runtime
            .eval_statement(&Statement::Expr(Expr::ident("Meters")))
            .unwrap();
        assert_eq!(value, Value::Type(Type::native::<Meters>()));
        assert_eq!(value.type_of().unwrap(), Type::type_type());
    }

    #[test]
    fn test_get_native_type() {
        let runtime = runtime_with_meters();
        assert_eq!(
            runtime.get_native_type("Meters"),
            Some(NativeTypeInfo::of::<Meters>())
        );
        assert_eq!(runtime.get_native_type("Seconds"), None);
    }

    #[test]
    fn test_unregistered_native_type_name_is_unbound() {
        let result =
            SimpleRuntime::default().eval_statement(&Statement::Expr(Expr::ident("Meters")));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UnboundIdentifier(name))) if name == "Meters"
        ));
    }

    #[test]
    fn test_native_values_through_flt() {
        let mut runtime = runtime_with_meters();
        let cases = [
            ("meters(5)", "5m"),
            ("typeof(meters(5))", "Meters"),
            ("typeof(meters(5)) == Meters", "true"),
            ("typeof(meters)", "fn(Number) -> Meters"),
            ("let m = meters(5); m == m", "true"),
            ("meters(5) == meters(5)", "true"),
            ("meters(5) == meters(6)", "false"),
            ("meters(5) != meters(6)", "true"),
            ("meters(5) == 5", "false"),
            ("\"m: \" <> meters(5)", "\"m: \""),
        ];
        for (input, expected) in cases {
            let value = runtime.eval(input).unwrap();
            assert_eq!(value.to_string(), expected, "for {input}");
        }
    }

    #[test]
    fn test_native_values_cannot_be_ordered() {
        let mut runtime = runtime_with_meters();
        let result = runtime.eval("meters(5) < meters(6)");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidOperandType))
        ));
    }

    #[test]
    fn test_native_type_name_is_not_callable() {
        let result =
            runtime_with_meters().eval_statement(&call("Meters", vec![Expr::literal_number(5)]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::NotCallable(name))) if name == "Meters"
        ));
    }

    #[test]
    fn test_field_access_on_native_value_is_invalid() {
        let mut runtime = runtime_with_meters();
        let result = runtime.eval("meters(5).n");
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::InvalidOperandType))
        ));
    }

    #[test]
    fn test_native_type_name_cannot_be_bound() {
        let mut runtime = runtime_with_meters();
        let result = runtime.eval_statement(&let_statement("Meters", Expr::literal_number(1)));
        assert_cannot_bind(result, "Meters", ProtectedName::NativeType);
        assert_cannot_bind(
            runtime.set_variable("Meters", Value::None),
            "Meters",
            ProtectedName::NativeType,
        );
        let error = runtime
            .eval_statement(&let_statement("Meters", Expr::literal_number(1)))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Cannot bind Meters: it's a native type"
        );
    }

    /// A native type named like a built-in type, for the registration errors.
    #[derive(PartialEq)]
    struct FakeNumber;

    impl fmt::Display for FakeNumber {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "fake")
        }
    }

    impl NativeType for FakeNumber {
        const NAME: &'static str = "Number";
    }

    /// A native type named like the built-in `typeof` function.
    #[derive(PartialEq)]
    struct FakeTypeof;

    impl fmt::Display for FakeTypeof {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "fake")
        }
    }

    impl NativeType for FakeTypeof {
        const NAME: &'static str = "typeof";
    }

    /// A second Rust type that uses `Meters`' name.
    #[derive(PartialEq)]
    struct OtherMeters;

    impl fmt::Display for OtherMeters {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "other")
        }
    }

    impl NativeType for OtherMeters {
        const NAME: &'static str = "Meters";
    }

    #[test]
    fn test_register_native_type_refuses_builtin_type_name() {
        let mut runtime = SimpleRuntime::default();
        assert_cannot_register(
            runtime.register_native_type::<FakeNumber>(),
            "native type",
            "Number",
            ProtectedName::BuiltinType,
        );
    }

    #[test]
    fn test_register_native_type_refuses_function_name() {
        let mut runtime = SimpleRuntime::default();
        assert_cannot_register(
            runtime.register_native_type::<FakeTypeof>(),
            "native type",
            "typeof",
            ProtectedName::Function,
        );
        assert_eq!(runtime.get_native_type("typeof"), None);
    }

    #[test]
    fn test_register_native_type_refuses_variable_name() {
        let mut runtime = SimpleRuntime::default();
        runtime.eval("let Meters = 1").unwrap();
        assert_cannot_register(
            runtime.register_native_type::<Meters>(),
            "native type",
            "Meters",
            ProtectedName::Variable,
        );
        assert_eq!(runtime.get_native_type("Meters"), None);
    }

    #[test]
    fn test_register_native_type_refuses_one_registered_twice() {
        let mut runtime = runtime_with_meters();
        assert_cannot_register(
            runtime.register_native_type::<Meters>(),
            "native type",
            "Meters",
            ProtectedName::NativeType,
        );
    }

    #[test]
    fn test_register_native_type_refuses_another_types_name() {
        let mut runtime = runtime_with_meters();
        assert_cannot_register(
            runtime.register_native_type::<OtherMeters>(),
            "native type",
            "Meters",
            ProtectedName::NativeType,
        );
        // The first type registered under the name keeps it
        assert_eq!(
            runtime.get_native_type("Meters"),
            Some(NativeTypeInfo::of::<Meters>())
        );
    }

    #[test]
    fn test_value_from_rust_integers() {
        assert_eq!(Value::from(3usize), number(3));
        assert_eq!(Value::from(3u64), number(3));
        assert_eq!(Value::from(-3i64), number(-3));
        assert_eq!(Value::from(u64::MAX).to_string(), u64::MAX.to_string());
    }

    fn decimal(s: &str) -> Value {
        Value::Number(BigDecimal::from_str(s).unwrap())
    }

    #[test]
    fn test_try_from_value_for_rust_numbers() {
        assert_eq!(i32::try_from(&number(-5)).unwrap(), -5);
        assert_eq!(i64::try_from(&number(-5)).unwrap(), -5);
        assert_eq!(u32::try_from(&number(5)).unwrap(), 5);
        assert_eq!(
            u64::try_from(&decimal("18446744073709551615")).unwrap(),
            u64::MAX
        );
        assert_eq!(usize::try_from(&number(5)).unwrap(), 5);
        assert_eq!(f64::try_from(&decimal("2.5")).unwrap(), 2.5);
        assert_eq!(f64::try_from(&number(5)).unwrap(), 5.0);
    }

    /// Asserts that `result` is the error for converting `number` to the Rust type `target`.
    fn assert_cannot_convert<T: std::fmt::Debug>(
        result: Result<T, Error>,
        number: &str,
        target: &str,
    ) {
        match result {
            Err(Error::RuntimeError(RuntimeError::CannotConvert {
                number: found_number,
                target: found_target,
            })) => {
                assert_eq!(found_number, number);
                assert_eq!(found_target, target);
            }
            other => panic!("expected CannotConvert for {number} to {target}, got {other:?}"),
        }
    }

    #[test]
    fn test_try_from_value_rejects_numbers_that_dont_fit() {
        assert_cannot_convert(i32::try_from(&decimal("2.5")), "2.5", "i32");
        assert_cannot_convert(i64::try_from(&decimal("2.5")), "2.5", "i64");
        assert_cannot_convert(u32::try_from(&number(-1)), "-1", "u32");
        assert_cannot_convert(u64::try_from(&number(-1)), "-1", "u64");
        assert_cannot_convert(usize::try_from(&number(-1)), "-1", "usize");
        assert_cannot_convert(i32::try_from(&number(3_000_000_000)), "3000000000", "i32");
        assert_cannot_convert(
            i64::try_from(&decimal("1e30")),
            &decimal("1e30").to_string(),
            "i64",
        );
        assert_eq!(
            u32::try_from(&number(-1)).unwrap_err().to_string(),
            "Runtime Error: Cannot convert -1 to u32"
        );
    }

    #[test]
    fn test_try_from_value_rejects_non_numbers() {
        let error = i32::try_from(&Value::String("5".to_string())).unwrap_err();
        assert!(matches!(
            &error,
            Error::RuntimeError(RuntimeError::UnexpectedType { expected, found })
                if expected == "Number" && found == "String"
        ));
        assert_eq!(
            error.to_string(),
            "Runtime Error: Expected Number, found String"
        );
        assert!(f64::try_from(&Value::Boolean(true)).is_err());
        assert!(usize::try_from(&Value::native(Meters(5))).is_err());
    }

    // The `Runtime` trait

    /// Registers the native type `Meters` and `meters` through the trait alone, as a host
    /// holding a `dyn Runtime` would.
    fn install_meters(runtime: &mut dyn Runtime) {
        runtime
            .register_native_type_info(NativeTypeInfo::of::<Meters>())
            .unwrap();
        runtime.register_function(meters_function()).unwrap();
    }

    #[test]
    fn test_eval_returns_the_last_statements_value() {
        let mut runtime = SimpleRuntime::default();
        assert_eq!(
            runtime.eval("let a = 1; let b = a + 1; b").unwrap(),
            number(2)
        );
        assert_eq!(
            runtime.eval("let c = 3\nlet d = c * 2\nd").unwrap(),
            number(6)
        );
        // Bindings persist from one call to the next
        assert_eq!(runtime.eval("a + d").unwrap(), number(7));
    }

    #[test]
    fn test_eval_of_empty_source_is_none() {
        let mut runtime = SimpleRuntime::default();
        for source in ["", "  \n ", "# only a comment", "# one\n# two\n"] {
            assert_eq!(runtime.eval(source).unwrap(), Value::None, "{source:?}");
        }
    }

    #[test]
    fn test_eval_of_a_syntax_error_runs_nothing() {
        let mut runtime = SimpleRuntime::default();
        let result = runtime.eval("let c = 1; let = 2");
        assert!(matches!(result, Err(Error::ParserError(_))), "{result:?}");
        assert_eq!(runtime.get_variable("c"), None);
    }

    #[test]
    fn test_eval_needs_a_separator_between_statements_on_one_line() {
        let mut runtime = SimpleRuntime::default();
        let result = runtime.eval("let x = 1 let y = 2");
        assert!(matches!(result, Err(Error::ParserError(_))), "{result:?}");
        assert_eq!(runtime.get_variable("x"), None);
        assert_eq!(runtime.eval("let x = 1\nlet y = 2").unwrap(), number(2));
    }

    #[test]
    fn test_eval_statement_evaluates_a_parsed_statement() {
        let mut runtime = SimpleRuntime::default();
        let (rest, statement) = parse_statement("1 + 2").unwrap();
        assert_eq!(rest, "");
        assert_eq!(runtime.eval_statement(&statement).unwrap(), number(3));
    }

    #[test]
    fn test_set_variable_then_get_variable() {
        let mut runtime = SimpleRuntime::default();
        let runtime: &mut dyn Runtime = &mut runtime;
        assert_eq!(runtime.get_variable("x"), None);
        runtime.set_variable("x", number(42)).unwrap();
        assert_eq!(runtime.get_variable("x"), Some(number(42)));
        assert_eq!(runtime.eval("x + 1").unwrap(), number(43));
        // flt source sees the host's binding, and the host sees flt's
        runtime.eval("let y = x * 2").unwrap();
        assert_eq!(runtime.get_variable("y"), Some(number(84)));
    }

    #[test]
    fn test_remove_variable() {
        let mut runtime = SimpleRuntime::default();
        let runtime: &mut dyn Runtime = &mut runtime;
        runtime.eval("let x = 1").unwrap();
        assert_eq!(runtime.remove_variable("x"), Some(number(1)));
        assert_eq!(runtime.remove_variable("x"), None);
        assert_eq!(runtime.get_variable("x"), None);
        assert!(matches!(
            runtime.eval("x"),
            Err(Error::RuntimeError(RuntimeError::UnboundIdentifier(name))) if name == "x"
        ));
        // A function isn't a variable, so it can't be removed as one
        assert_eq!(runtime.remove_variable("typeof"), None);
        assert!(runtime.get_function("typeof").is_some());
    }

    #[test]
    fn test_get_function() {
        let mut runtime = runtime_with_meters();
        let runtime: &mut dyn Runtime = &mut runtime;
        runtime.eval("let x = 1").unwrap();
        let function = runtime.get_function("typeof").unwrap();
        assert_eq!(function.r#type().to_string(), "fn(Any) -> Type");
        let function = runtime.get_function("meters").unwrap();
        assert_eq!(function.r#type().to_string(), "fn(Number) -> Meters");
        for name in ["nope", "x", "Number", "Meters"] {
            assert!(runtime.get_function(name).is_none(), "{name}");
        }
    }

    #[test]
    fn test_register_native_type_info_through_dyn_runtime() {
        let mut runtime = SimpleRuntime::default();
        install_meters(&mut runtime);
        assert_eq!(
            runtime.get_native_type("Meters"),
            Some(NativeTypeInfo::of::<Meters>())
        );
        assert_eq!(
            runtime.eval("typeof(meters(5)) == Meters").unwrap(),
            Value::Boolean(true)
        );
        assert_cannot_bind(
            runtime.eval("let Meters = 1"),
            "Meters",
            ProtectedName::NativeType,
        );
    }

    #[test]
    fn test_boxed_runtime() {
        let mut runtime: Box<dyn Runtime> = Box::new(SimpleRuntime::default());
        install_meters(runtime.as_mut());
        assert_eq!(runtime.eval("meters(3)").unwrap(), Value::native(Meters(3)));
    }
}
