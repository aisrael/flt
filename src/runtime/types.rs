//! Types for the flt runtime

use std::fmt;

/// A type is either a builtin type or a custom type
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
    /// The `Option` wrapper around an inner type (the empty case is `None`)
    Option(Box<Type>),
}

impl Type {
    /// The built-in unit type
    pub fn unit() -> Self {
        Type::Builtin(BuiltinType::Unit)
    }

    /// The built-in number type
    pub fn number() -> Self {
        Type::Builtin(BuiltinType::Number)
    }

    /// The built-in string type
    pub fn string() -> Self {
        Type::Builtin(BuiltinType::String)
    }

    /// The built-in boolean type
    pub fn boolean() -> Self {
        Type::Builtin(BuiltinType::Boolean)
    }

    /// The built-in symbol type
    pub fn symbol() -> Self {
        Type::Builtin(BuiltinType::Symbol)
    }

    /// The built-in array type
    pub fn array() -> Self {
        Type::Builtin(BuiltinType::Array)
    }

    /// The built-in map type
    pub fn map() -> Self {
        Type::Builtin(BuiltinType::Map)
    }

    /// The `Option` wrapper type around an inner type
    pub fn option(inner: Type) -> Self {
        Type::Option(Box::new(inner))
    }

    /// The universal value type (accepts any value)
    pub fn value() -> Self {
        Type::Builtin(BuiltinType::Value)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Builtin(b) => write!(f, "{b}"),
            Type::Custom(c) => write!(f, "{}", c.name),
            Type::Option(inner) => write!(f, "Option<{inner}>"),
        }
    }
}

/// A builtin type is a type that is predefined in the runtime.
#[derive(Clone, Debug, PartialEq)]
pub enum BuiltinType {
    Unit,
    String,
    Number,
    Boolean,
    Symbol,
    Array,
    Map,
    Value,
}

impl fmt::Display for BuiltinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BuiltinType::Unit => "Unit",
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Boolean => "Boolean",
            BuiltinType::Symbol => "Symbol",
            BuiltinType::Array => "Array",
            BuiltinType::Map => "Map",
            BuiltinType::Value => "Value",
        };
        write!(f, "{name}")
    }
}

/// A custom type is a type that is defined by the user.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomType {
    pub name: String,
}

impl CustomType {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
