//! Types for the flt runtime

/// A type is either a builtin type or a custom type
#[derive(Debug, PartialEq)]
pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
}

/// A builtin type is a type that is predefined in the runtime.
#[derive(Debug, PartialEq)]
pub enum BuiltinType {
    String,
    Number,
    Boolean,
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