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
    /// The built-in `None` type, the type of the `None` value
    pub fn none() -> Self {
        Type::Builtin(BuiltinType::None)
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

    /// The built-in `Type` type, the type of a type value
    pub fn type_type() -> Self {
        Type::Builtin(BuiltinType::Type)
    }

    /// The `Any` placeholder, used only in signatures (accepts any value)
    pub fn any() -> Self {
        Type::Builtin(BuiltinType::Any)
    }

    /// The built-in type a name evaluates to, e.g. `Number`. `Any` is not a value, so it
    /// has none.
    pub fn builtin_named(name: &str) -> Option<Self> {
        let builtin = match name {
            "None" => BuiltinType::None,
            "String" => BuiltinType::String,
            "Number" => BuiltinType::Number,
            "Boolean" => BuiltinType::Boolean,
            "Symbol" => BuiltinType::Symbol,
            "Array" => BuiltinType::Array,
            "Map" => BuiltinType::Map,
            "Type" => BuiltinType::Type,
            _ => return None,
        };
        Some(Type::Builtin(builtin))
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
    None,
    String,
    Number,
    Boolean,
    Symbol,
    Array,
    Map,
    /// The type of a type value, e.g. `typeof(Number)`
    Type,
    /// A placeholder used only in signatures; no value has type `Any`
    Any,
}

impl fmt::Display for BuiltinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BuiltinType::None => "None",
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Boolean => "Boolean",
            BuiltinType::Symbol => "Symbol",
            BuiltinType::Array => "Array",
            BuiltinType::Map => "Map",
            BuiltinType::Type => "Type",
            BuiltinType::Any => "Any",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_type_display() {
        assert_eq!(Type::none().to_string(), "None");
    }

    #[test]
    fn test_type_and_any_display() {
        assert_eq!(Type::type_type().to_string(), "Type");
        assert_eq!(Type::any().to_string(), "Any");
    }

    #[test]
    fn test_builtin_named() {
        assert_eq!(Type::builtin_named("Number"), Some(Type::number()));
        assert_eq!(Type::builtin_named("None"), Some(Type::none()));
        assert_eq!(Type::builtin_named("Type"), Some(Type::type_type()));
    }

    #[test]
    fn test_builtin_named_excludes_any_and_unknown_names() {
        assert_eq!(Type::builtin_named("Any"), None);
        assert_eq!(Type::builtin_named("Foo"), None);
        assert_eq!(Type::builtin_named("number"), None);
    }
}
