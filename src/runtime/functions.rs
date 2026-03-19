//! Functions in the runtime

use crate::runtime::types::Type;

/// A function definition is a collection of function signatures (overloads)
pub struct FunctionDefinition {
    pub name: String,
    pub overloads: Vec<FunctionSignature>,
}

impl FunctionDefinition {
    /// Create a new function definition with a name, return type, and arguments
    pub fn new<S: Into<String>>(name: S, return_type: Type, arguments: Vec<Argument>) -> Self {
        Self {
            name: name.into(),
            overloads: vec![FunctionSignature {
                arguments,
                return_type,
            }],
        }
    }

    /// Add a new overload to the function definition
    pub fn add_overload(mut self, return_type: Type, arguments: Vec<Argument>) -> Self {
        self.overloads.push(FunctionSignature {
            arguments,
            return_type,
        });
        self
    }

    /// Check if the function definition accepts the given arguments.
    /// The arguments are matched by name and type, and according to the order they
    /// were defined (inserted) into the function definition.
    pub fn accepts(&self, arguments: Vec<Argument>) -> bool {
        for overload in &self.overloads {
            if overload.arguments == arguments {
                return true;
            }
        }
        false
    }
}

/// A function signature is a single function definition with a name, arguments, and return type
pub struct FunctionSignature {
    pub arguments: Vec<Argument>,
    pub return_type: Type,
}

/// An argument is a single argument to a function
#[derive(Debug, PartialEq)]
pub struct Argument {
    pub name: String,
    pub r#type: Type,
}

impl Argument {
    pub fn new<S: Into<String>>(name: S, r#type: Type) -> Self {
        Self {
            name: name.into(),
            r#type,
        }
    }

    /// A convenience method for an argument with the built-in number type
    pub fn number<S: Into<String>>(name: S) -> Self {
        Self::new(name, Type::number())
    }

    /// A convenience method for an argument with the built-in string type
    pub fn string<S: Into<String>>(name: S) -> Self {
        Self::new(name, Type::string())
    }

    /// A convenience method for an argument with the built-in boolean type
    pub fn boolean<S: Into<String>>(name: S) -> Self {
        Self::new(name, Type::boolean())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accepts() {
        let function_definition = FunctionDefinition::new(
            "add",
            Type::number(),
            vec![Argument::number("a"), Argument::number("b")],
        );
        assert!(function_definition.accepts(vec![Argument::number("a"), Argument::number("b")]));
        assert!(!function_definition.accepts(vec![Argument::number("b")]));
    }
}
