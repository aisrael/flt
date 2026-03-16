//! Functions in the runtime

use crate::runtime::types::Type;

/// A function definition is a collection of function signatures (overloads)
pub struct FunctionDefinition {
    pub name: String,
    pub overloads: Vec<FunctionSignature>,
}

impl FunctionDefinition {
    /// Create a new function definition with a name, return type, and arguments
    pub fn new(name: String, return_type: Type, arguments: Vec<Argument>) -> Self {
        Self {
            name,
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

    /// Find an overload of the function definition that matches the given arguments
    /// The arguments are matched by name and type, and according to the order they 
    /// were defined (inserted) into the function definition.
    pub fn find_overload(self, arguments: Vec<Argument>) -> Option<FunctionSignature> {
        for overload in self.overloads {
            if overload.arguments == arguments {
                return Some(overload);
            }
        }
        None
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

#[cfg(test)]
mod tests {
    use super::*;

}