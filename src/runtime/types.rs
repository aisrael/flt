//! Types for the flt runtime

/// A type is either a builtin type or a custom type
#[derive(Debug, PartialEq)]
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
}

/// A builtin type is a type that is predefined in the runtime.
#[derive(Debug, PartialEq)]
pub enum BuiltinType {
    Unit,
    String,
    Number,
    Boolean,
    Symbol,
    Array,
    Map,
}

/// A custom type is a type that is defined by the user.
#[derive(Debug, PartialEq)]
pub struct CustomType {
    pub name: String,
}

impl CustomType {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
