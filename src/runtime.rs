//! Runtimes for the `flt` language

pub mod functions;
pub mod types;

use std::collections::HashMap;

use bigdecimal::BigDecimal;

/// A value in the runtime
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The unit value (like `()` in Rust/Elixir)
    Unit,
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

pub trait Runtime {}

pub struct SimpleRuntime;

impl Runtime for SimpleRuntime {}

pub struct GlobalScope {
    pub functions: HashMap<String, Box<dyn BuiltInFunction>>,
    pub constants: HashMap<String, Value>,
}

impl GlobalScope {
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get_function(&self, name: &str) -> Option<&dyn BuiltInFunction> {
        self.functions.get(name).map(|f| f.as_ref())
    }
}

pub struct FunctionSignature {
    pub name: String,
}

pub trait BuiltInFunction {
    fn signature(&self) -> FunctionSignature;
}
