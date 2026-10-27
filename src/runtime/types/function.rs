//! Function types: `fn(P1, P2, *V, k: K) -> R`

use std::fmt;

use crate::runtime::functions::FunctionSignature;
use crate::runtime::types::Type;
use crate::utils::escape_string;
use crate::utils::is_bare_name;

/// The type of a function with a single signature: its parameter types, the type of a
/// variadic last parameter, its keyword arguments' names and types, and its return type.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    pub parameters: Vec<Type>,
    /// The type of each value a variadic last parameter takes, if the function has one. It
    /// isn't in `parameters`.
    pub variadic: Option<Box<Type>>,
    /// The keyword arguments' names and types, in declaration order
    pub keywords: Vec<(String, Type)>,
    /// The type of any other keyword arguments the function accepts, if it accepts them
    pub rest_keywords: Option<Box<Type>>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    /// Create a new function type from its parameter types and return type, without keyword
    /// arguments
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self {
            parameters,
            variadic: None,
            keywords: Vec::new(),
            rest_keywords: None,
            return_type: Box::new(return_type),
        }
    }

    /// Create a new function type whose parameters end with a variadic one, which takes
    /// zero or more values of type `variadic`, e.g. `fn(String, *Any) -> String`
    pub fn with_variadic(parameters: Vec<Type>, variadic: Type, return_type: Type) -> Self {
        Self {
            variadic: Some(Box::new(variadic)),
            ..Self::new(parameters, return_type)
        }
    }
}

impl From<&FunctionSignature> for FunctionType {
    fn from(signature: &FunctionSignature) -> Self {
        let (variadic, parameters): (Vec<_>, Vec<_>) = signature
            .arguments
            .iter()
            .partition(|argument| argument.variadic);
        Self {
            parameters: parameters
                .into_iter()
                .map(|argument| argument.r#type.clone())
                .collect(),
            variadic: variadic
                .first()
                .map(|argument| Box::new(argument.r#type.clone())),
            keywords: signature
                .keywords
                .iter()
                .map(|keyword| (keyword.name.clone(), keyword.r#type.clone()))
                .collect(),
            rest_keywords: signature.rest_keywords.clone().map(Box::new),
            return_type: Box::new(signature.return_type.clone()),
        }
    }
}

/// Writes `fn(P1, P2, *V, k: K, **T) -> R`. `->` groups before `|`, so a union return type is
/// bracketed, e.g. `fn(Number) -> (Number | None)`.
impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| parameter.to_string());
        let keywords = self.keywords.iter().map(|(name, r#type)| {
            if is_bare_name(name) {
                format!("{name}: {type}")
            } else {
                format!("\"{}\": {type}", escape_string(name))
            }
        });
        let variadic = self.variadic.iter().map(|r#type| format!("*{type}"));
        let rest_keywords = self.rest_keywords.iter().map(|r#type| format!("**{type}"));
        for (i, parameter) in parameters
            .chain(variadic)
            .chain(keywords)
            .chain(rest_keywords)
            .enumerate()
        {
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
    use crate::runtime::functions::KeywordArgument;
    use crate::runtime::native::fixtures::Meters;
    use crate::runtime::Value;

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
        let signature = FunctionSignature::new(
            vec![Argument::string("s"), Argument::number("n")],
            Type::string(),
        );
        assert_eq!(
            FunctionType::from(&signature),
            FunctionType::new(vec![Type::string(), Type::number()], Type::string())
        );
    }

    #[test]
    fn test_display_with_variadic() {
        let function_type =
            FunctionType::with_variadic(vec![Type::string()], Type::any(), Type::string());
        assert_eq!(function_type.to_string(), "fn(String, *Any) -> String");
        let function_type = FunctionType::with_variadic(vec![], Type::any(), Type::number());
        assert_eq!(function_type.to_string(), "fn(*Any) -> Number");
        let function_type = with_keywords(
            FunctionType::with_variadic(vec![Type::any()], Type::any(), Type::any()),
            &[("k", Type::number())],
            Some(Type::any()),
        );
        assert_eq!(
            function_type.to_string(),
            "fn(Any, *Any, k: Number, **Any) -> Any"
        );
    }

    #[test]
    fn test_from_function_signature_with_variadic() {
        let signature = FunctionSignature::new(
            vec![
                Argument::string("format"),
                Argument::variadic("args", Type::any()),
            ],
            Type::string(),
        );
        assert_eq!(
            FunctionType::from(&signature),
            FunctionType::with_variadic(vec![Type::string()], Type::any(), Type::string())
        );
    }

    #[test]
    fn test_equality_includes_variadic() {
        assert_ne!(
            FunctionType::new(vec![Type::any()], Type::any()),
            FunctionType::with_variadic(vec![], Type::any(), Type::any())
        );
    }

    fn with_keywords(
        mut function_type: FunctionType,
        keywords: &[(&str, Type)],
        rest_keywords: Option<Type>,
    ) -> FunctionType {
        function_type.keywords = keywords
            .iter()
            .map(|(name, r#type)| (name.to_string(), r#type.clone()))
            .collect();
        function_type.rest_keywords = rest_keywords.map(Box::new);
        function_type
    }

    #[test]
    fn test_display_with_keywords() {
        let function_type = with_keywords(
            FunctionType::new(vec![Type::string()], Type::string()),
            &[("greeting", Type::string()), ("c d", Type::number())],
            None,
        );
        assert_eq!(
            function_type.to_string(),
            r#"fn(String, greeting: String, "c d": Number) -> String"#
        );
    }

    #[test]
    fn test_display_with_rest_keywords() {
        let function_type = with_keywords(
            FunctionType::new(vec![Type::any()], Type::any()),
            &[("n", Type::number())],
            Some(Type::any()),
        );
        assert_eq!(
            function_type.to_string(),
            "fn(Any, n: Number, **Any) -> Any"
        );
        let function_type = with_keywords(
            FunctionType::new(vec![], Type::map()),
            &[],
            Some(Type::any()),
        );
        assert_eq!(function_type.to_string(), "fn(**Any) -> Map");
    }

    #[test]
    fn test_equality_includes_keywords() {
        let plain = FunctionType::new(vec![Type::any()], Type::any());
        let keyword = with_keywords(plain.clone(), &[("k", Type::any())], None);
        let rest = with_keywords(plain.clone(), &[], Some(Type::any()));
        assert_ne!(plain, keyword);
        assert_ne!(plain, rest);
        assert_ne!(keyword, rest);
        assert_eq!(keyword, with_keywords(plain, &[("k", Type::any())], None));
    }

    #[test]
    fn test_from_function_signature_with_keywords() {
        let mut signature = FunctionSignature::new(vec![Argument::string("s")], Type::string());
        signature.keywords.push(KeywordArgument {
            name: "k".to_string(),
            r#type: Type::number(),
            default: Value::None,
        });
        signature.rest_keywords = Some(Type::any());
        assert_eq!(
            FunctionType::from(&signature),
            with_keywords(
                FunctionType::new(vec![Type::string()], Type::string()),
                &[("k", Type::number())],
                Some(Type::any()),
            )
        );
    }

    #[test]
    fn test_type_function_display() {
        let function_type = Type::Function(FunctionType::new(vec![Type::any()], Type::any()));
        assert_eq!(function_type.to_string(), "fn(Any) -> Any");
    }

    #[test]
    fn test_display_with_native_types() {
        let function_type = FunctionType::new(
            vec![Type::native::<Meters>(), Type::string()],
            Type::native::<Meters>(),
        );
        assert_eq!(function_type.to_string(), "fn(Meters, String) -> Meters");
    }

    #[test]
    fn test_from_function_signature_with_native_types() {
        let signature = FunctionSignature::new(
            vec![Argument::new("m", Type::native::<Meters>())],
            Type::number(),
        );
        assert_eq!(
            FunctionType::from(&signature).to_string(),
            "fn(Meters) -> Number"
        );
    }
}
