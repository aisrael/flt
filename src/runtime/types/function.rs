//! Function types: `fn(P1, P2) -> R`

use std::fmt;

use crate::runtime::functions::FunctionSignature;
use crate::runtime::types::Type;

/// The type of a function with a single signature: its parameter types and return type.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    /// Create a new function type from its parameter types and return type
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self {
            parameters,
            return_type: Box::new(return_type),
        }
    }
}

impl From<&FunctionSignature> for FunctionType {
    fn from(signature: &FunctionSignature) -> Self {
        Self::new(
            signature
                .arguments
                .iter()
                .map(|argument| argument.r#type.clone())
                .collect(),
            signature.return_type.clone(),
        )
    }
}

/// Writes `fn(P1, P2) -> R`. `->` groups before `|`, so a union return type is bracketed,
/// e.g. `fn(Number) -> (Number | None)`.
impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        for (i, parameter) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{parameter}")?;
        }
        match self.return_type.as_ref() {
            Type::Union(_) => write!(f, ") -> ({})", self.return_type),
            return_type => write!(f, ") -> {return_type}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::functions::Argument;

    #[test]
    fn test_display_without_parameters() {
        let function_type = FunctionType::new(vec![], Type::number());
        assert_eq!(function_type.to_string(), "fn() -> Number");
    }

    #[test]
    fn test_display_with_parameters() {
        let function_type =
            FunctionType::new(vec![Type::number(), Type::string()], Type::boolean());
        assert_eq!(function_type.to_string(), "fn(Number, String) -> Boolean");
    }

    #[test]
    fn test_display_brackets_union_return_type() {
        let function_type = FunctionType::new(
            vec![Type::number()],
            Type::union(Type::number(), Type::none()),
        );
        assert_eq!(function_type.to_string(), "fn(Number) -> (Number | None)");
    }

    #[test]
    fn test_from_function_signature() {
        let signature = FunctionSignature {
            arguments: vec![Argument::string("s"), Argument::number("n")],
            return_type: Type::string(),
        };
        assert_eq!(
            FunctionType::from(&signature),
            FunctionType::new(vec![Type::string(), Type::number()], Type::string())
        );
    }

    #[test]
    fn test_type_function_display() {
        let function_type = Type::Function(FunctionType::new(vec![Type::any()], Type::any()));
        assert_eq!(function_type.to_string(), "fn(Any) -> Any");
    }
}
