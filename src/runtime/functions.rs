//! Functions in the runtime

use crate::runtime::types::Type;
use crate::runtime::Value;
use crate::Error;

/// The native handler backing a built-in function.
pub type BuiltinFn = fn(&[Value]) -> Result<Value, Error>;

/// A built-in function pairs a type signature with a native implementation.
pub struct BuiltinFunction {
    pub definition: FunctionDefinition,
    pub handler: BuiltinFn,
}

impl BuiltinFunction {
    /// Create a new built-in function from a definition and a native handler.
    pub fn new(definition: FunctionDefinition, handler: BuiltinFn) -> Self {
        Self {
            definition,
            handler,
        }
    }

    /// Invoke the built-in function with the given evaluated arguments.
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        (self.handler)(args)
    }
}

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

    #[test]
    fn test_builtin_function_call() {
        let identity = BuiltinFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::value(),
                vec![Argument::new("v", Type::value())],
            ),
            |args| Ok(args[0].clone()),
        );
        assert_eq!(
            identity.call(&[Value::Boolean(true)]).unwrap(),
            Value::Boolean(true)
        );
    }
}
