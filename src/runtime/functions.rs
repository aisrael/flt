//! Functions in the runtime

use std::fmt;

use crate::errors::RuntimeError;
use crate::runtime::types::Type;
use crate::runtime::Map;
use crate::runtime::MapKey;
use crate::runtime::Value;
use crate::Error;

/// The handler backing a native function: a Rust closure, which may capture host state.
pub type NativeFn = Box<dyn Fn(&[Value]) -> Result<Value, Error> + Send + Sync>;

/// The handler backing a native function that takes keyword arguments. It gets the
/// positional arguments, then the keyword arguments with their defaults filled in.
pub type KeywordFn = Box<dyn Fn(&[Value], &Map) -> Result<Value, Error> + Send + Sync>;

/// A native function's handler, with or without keyword arguments.
enum Handler {
    Positional(NativeFn),
    Keywords(KeywordFn),
}

/// A native function pairs a type signature with a Rust implementation.
pub struct NativeFunction {
    pub definition: FunctionDefinition,
    handler: Handler,
}

impl NativeFunction {
    /// Create a new native function from a definition and a handler.
    ///
    /// # Panics
    ///
    /// If the definition declares keyword arguments, which the handler couldn't see. Use
    /// [`NativeFunction::with_keywords`] for those.
    pub fn new(
        definition: FunctionDefinition,
        handler: impl Fn(&[Value]) -> Result<Value, Error> + Send + Sync + 'static,
    ) -> Self {
        assert!(
            !definition.declares_keywords(),
            "Function {} declares keyword arguments, so it needs NativeFunction::with_keywords",
            definition.name
        );
        Self {
            definition,
            handler: Handler::Positional(Box::new(handler)),
        }
    }

    /// Create a native function whose handler also receives the call's keyword arguments,
    /// with defaults filled in.
    pub fn with_keywords(
        definition: FunctionDefinition,
        handler: impl Fn(&[Value], &Map) -> Result<Value, Error> + Send + Sync + 'static,
    ) -> Self {
        Self {
            definition,
            handler: Handler::Keywords(Box::new(handler)),
        }
    }

    /// The name the function is registered under.
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// The function's type, e.g. `fn(Any) -> Any`.
    pub fn r#type(&self) -> Type {
        self.definition.r#type()
    }

    /// Invoke the function with the given evaluated arguments and no keyword arguments,
    /// after checking their number against the function's signatures.
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        self.call_with_keywords(args, Map::new())
    }

    /// Invoke the function with evaluated positional and keyword arguments. The overload is
    /// picked by the number of positional arguments, then the keywords are checked against
    /// that overload's.
    pub fn call_with_keywords(&self, args: &[Value], keywords: Map) -> Result<Value, Error> {
        let signature = self.definition.resolve(args.len())?;
        let keywords = signature.bind_keywords(&self.definition.name, keywords)?;
        match &self.handler {
            Handler::Positional(handler) => handler(args),
            Handler::Keywords(handler) => handler(args, &keywords),
        }
    }
}

/// Two native functions are equal only if they're the same function.
impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.definition.name)
            .field("type", &self.r#type().to_string())
            .finish_non_exhaustive()
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
            overloads: vec![FunctionSignature::new(arguments, return_type)],
        }
    }

    /// Add a new overload to the function definition
    pub fn add_overload(mut self, return_type: Type, arguments: Vec<Argument>) -> Self {
        self.overloads
            .push(FunctionSignature::new(arguments, return_type));
        self
    }

    /// Add an optional keyword argument, with a default, to the most recently added overload.
    ///
    /// # Panics
    ///
    /// If that overload already declares a keyword with this name.
    pub fn keyword<S: Into<String>>(
        mut self,
        name: S,
        r#type: Type,
        default: impl Into<Value>,
    ) -> Self {
        let name = name.into();
        let declared = self.overloads.last().is_some_and(|signature| {
            signature
                .keywords
                .iter()
                .any(|keyword| keyword.name == name)
        });
        assert!(
            !declared,
            "Function {} declares the keyword argument {name} twice",
            self.name
        );
        self.last_overload().keywords.push(KeywordArgument {
            name,
            r#type,
            default: default.into(),
        });
        self
    }

    /// Let the most recently added overload accept any other keyword arguments, of the
    /// given type.
    pub fn rest_keywords(mut self, r#type: Type) -> Self {
        self.last_overload().rest_keywords = Some(r#type);
        self
    }

    fn last_overload(&mut self) -> &mut FunctionSignature {
        self.overloads
            .last_mut()
            .expect("a function definition has at least one overload")
    }

    /// Whether any overload declares keyword arguments or rest keywords.
    fn declares_keywords(&self) -> bool {
        self.overloads
            .iter()
            .any(|overload| !overload.keywords.is_empty() || overload.rest_keywords.is_some())
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

    /// The function's type: the union of its overloads' function types, so a single
    /// overload gives a plain function type. A definition with no overloads gives the
    /// empty union.
    pub fn r#type(&self) -> Type {
        self.overloads
            .iter()
            .map(|overload| Type::Function(overload.into()))
            .reduce(Type::union)
            .unwrap_or_else(|| Type::Union(Vec::new()))
    }

    /// The first overload that takes `found` positional arguments.
    pub fn resolve(&self, found: usize) -> Result<&FunctionSignature, Error> {
        self.overloads
            .iter()
            .find(|overload| overload.arguments.len() == found)
            .ok_or_else(|| {
                Error::RuntimeError(RuntimeError::ArgumentCountMismatch {
                    name: self.name.clone(),
                    expected: self
                        .overloads
                        .first()
                        .map_or(0, |overload| overload.arguments.len()),
                    found,
                })
            })
    }

    /// Check that some overload takes `found` positional arguments.
    pub fn check_arity(&self, found: usize) -> Result<(), Error> {
        self.resolve(found).map(|_| ())
    }
}

/// A function signature is a single function definition with a name, arguments, and return type
pub struct FunctionSignature {
    pub arguments: Vec<Argument>,
    /// The optional keyword arguments, in declaration order
    pub keywords: Vec<KeywordArgument>,
    /// The type of any other keyword arguments the signature accepts, if it accepts them
    pub rest_keywords: Option<Type>,
    pub return_type: Type,
}

impl FunctionSignature {
    /// A signature with positional arguments only
    pub fn new(arguments: Vec<Argument>, return_type: Type) -> Self {
        Self {
            arguments,
            keywords: Vec::new(),
            rest_keywords: None,
            return_type,
        }
    }

    /// Builds the keyword arguments a handler receives from a call's: every declared keyword,
    /// in declaration order, with the call's value or else its default, then any rest
    /// keywords in call order. A keyword the signature doesn't accept raises
    /// `UnexpectedKeywordArgument`.
    pub fn bind_keywords(&self, function_name: &str, mut keywords: Map) -> Result<Map, Error> {
        let mut bound = Map::new();
        for keyword in &self.keywords {
            let key = MapKey::symbol(keyword.name.as_str());
            let value = keywords
                .shift_remove(&key)
                .unwrap_or_else(|| keyword.default.clone());
            bound.insert(key, value);
        }
        if let (None, Some(key)) = (&self.rest_keywords, keywords.keys().next()) {
            let (MapKey::Symbol(key) | MapKey::String(key)) = key;
            return Err(Error::RuntimeError(
                RuntimeError::UnexpectedKeywordArgument {
                    name: function_name.to_string(),
                    key: key.clone(),
                },
            ));
        }
        bound.extend(keywords);
        Ok(bound)
    }
}

/// An optional keyword argument, with a default value
#[derive(Debug, PartialEq)]
pub struct KeywordArgument {
    pub name: String,
    pub r#type: Type,
    pub default: Value,
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
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use super::*;
    use crate::runtime::types::FunctionType;

    fn identity() -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::any(),
                vec![Argument::new("v", Type::any())],
            ),
            |args| Ok(args[0].clone()),
        )
    }

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
    fn test_native_function_call() {
        assert_eq!(
            identity().call(&[Value::Boolean(true)]).unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_native_function_call_checks_arity() {
        let result = identity().call(&[Value::None, Value::None]);
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected: 1, found: 2 }))
                if name == "identity"
        ));
    }

    #[test]
    fn test_native_function_closure_captures_state() {
        let calls = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&calls);
        let tick = NativeFunction::new(
            FunctionDefinition::new("tick", Type::number(), vec![]),
            move |_args| {
                Ok(Value::Number(
                    (counter.fetch_add(1, Ordering::SeqCst) + 1).into(),
                ))
            },
        );
        tick.call(&[]).unwrap();
        assert_eq!(tick.call(&[]).unwrap(), Value::Number(2.into()));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_native_function_equality_is_identity() {
        let a = Arc::new(identity());
        let b = Arc::new(identity());
        assert_eq!(a, Arc::clone(&a));
        assert_ne!(a, b);
    }

    #[test]
    fn test_native_function_name_and_type() {
        let function = identity();
        assert_eq!(function.name(), "identity");
        assert_eq!(function.r#type().to_string(), "fn(Any) -> Any");
    }

    #[test]
    fn test_type_of_single_overload_is_function_type() {
        let definition =
            FunctionDefinition::new("len", Type::number(), vec![Argument::string("s")]);
        assert_eq!(
            definition.r#type(),
            Type::Function(FunctionType::new(vec![Type::string()], Type::number()))
        );
    }

    #[test]
    fn test_type_of_overloads_is_union_of_function_types() {
        let definition =
            FunctionDefinition::new("pad", Type::string(), vec![Argument::string("s")])
                .add_overload(
                    Type::string(),
                    vec![Argument::string("s"), Argument::number("n")],
                );
        assert_eq!(
            definition.r#type().to_string(),
            "fn(String) -> String | fn(String, Number) -> String"
        );
    }

    #[test]
    fn test_check_arity_accepts_any_overload() {
        let definition =
            FunctionDefinition::new("pad", Type::string(), vec![Argument::string("s")])
                .add_overload(
                    Type::string(),
                    vec![Argument::string("s"), Argument::number("n")],
                );
        assert!(definition.check_arity(1).is_ok());
        assert!(definition.check_arity(2).is_ok());
        assert!(definition.check_arity(3).is_err());
    }

    fn number(n: u64) -> Value {
        Value::from(n)
    }

    fn keywords(entries: &[(&str, Value)]) -> Map {
        entries
            .iter()
            .map(|(name, value)| (MapKey::symbol(*name), value.clone()))
            .collect()
    }

    /// echo(**) returns the keyword arguments it was given, as a map
    fn echo(definition: FunctionDefinition) -> NativeFunction {
        NativeFunction::with_keywords(definition, |_args, keywords| {
            Ok(Value::Map(keywords.clone()))
        })
    }

    fn unexpected_keyword(result: Result<Value, Error>) -> Option<(String, String)> {
        match result {
            Err(Error::RuntimeError(RuntimeError::UnexpectedKeywordArgument { name, key })) => {
                Some((name, key))
            }
            _ => None,
        }
    }

    #[test]
    fn test_keyword_defaults_fill_in_and_call_values_override_them() {
        let function = echo(
            FunctionDefinition::new("f", Type::map(), vec![])
                .keyword("a", Type::number(), 1u64)
                .keyword("b", Type::number(), 2u64),
        );
        assert_eq!(
            function.call(&[]).unwrap(),
            Value::Map(keywords(&[("a", number(1)), ("b", number(2))]))
        );
        assert_eq!(
            function
                .call_with_keywords(&[], keywords(&[("b", number(20))]))
                .unwrap(),
            Value::Map(keywords(&[("a", number(1)), ("b", number(20))]))
        );
    }

    #[test]
    fn test_declared_keywords_come_first_then_rest_keywords_in_call_order() {
        let function = echo(
            FunctionDefinition::new("f", Type::map(), vec![])
                .keyword("a", Type::number(), 1u64)
                .keyword("b", Type::number(), 2u64)
                .rest_keywords(Type::any()),
        );
        let result = function
            .call_with_keywords(
                &[],
                keywords(&[
                    ("z", number(26)),
                    ("b", number(20)),
                    ("y", number(25)),
                    ("a", number(10)),
                ]),
            )
            .unwrap();
        let Value::Map(map) = result else {
            panic!("expected a map, got {result}");
        };
        assert_eq!(
            map.into_iter().collect::<Vec<_>>(),
            vec![
                (MapKey::symbol("a"), number(10)),
                (MapKey::symbol("b"), number(20)),
                (MapKey::symbol("z"), number(26)),
                (MapKey::symbol("y"), number(25)),
            ]
        );
    }

    #[test]
    fn test_unexpected_keyword_is_an_error() {
        let function =
            echo(FunctionDefinition::new("f", Type::map(), vec![]).keyword("a", Type::any(), 1u64));
        assert_eq!(
            unexpected_keyword(function.call_with_keywords(&[], keywords(&[("b", number(1))]))),
            Some(("f".to_string(), "b".to_string()))
        );
    }

    #[test]
    fn test_function_without_keywords_rejects_them() {
        assert_eq!(
            unexpected_keyword(
                identity().call_with_keywords(&[Value::None], keywords(&[("k", number(1))]))
            ),
            Some(("identity".to_string(), "k".to_string()))
        );
    }

    #[test]
    fn test_overload_is_picked_by_positional_count_before_keywords_are_checked() {
        let function = echo(
            FunctionDefinition::new("f", Type::map(), vec![])
                .add_overload(Type::map(), vec![Argument::new("v", Type::any())])
                .keyword("k", Type::any(), 0u64),
        );
        assert_eq!(
            function
                .call_with_keywords(&[Value::None], keywords(&[("k", number(1))]))
                .unwrap(),
            Value::Map(keywords(&[("k", number(1))]))
        );
        assert_eq!(
            unexpected_keyword(function.call_with_keywords(&[], keywords(&[("k", number(1))]))),
            Some(("f".to_string(), "k".to_string()))
        );
        assert!(matches!(
            function.call_with_keywords(&[Value::None, Value::None], keywords(&[("j", number(1))])),
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch {
                found: 2,
                ..
            }))
        ));
    }

    #[test]
    #[should_panic(expected = "NativeFunction::with_keywords")]
    fn test_new_panics_on_a_definition_with_keywords() {
        NativeFunction::new(
            FunctionDefinition::new("f", Type::any(), vec![]).keyword("k", Type::any(), 1u64),
            |_args| Ok(Value::None),
        );
    }

    #[test]
    #[should_panic(expected = "NativeFunction::with_keywords")]
    fn test_new_panics_on_a_definition_with_rest_keywords() {
        NativeFunction::new(
            FunctionDefinition::new("f", Type::any(), vec![]).rest_keywords(Type::any()),
            |_args| Ok(Value::None),
        );
    }

    #[test]
    #[should_panic(expected = "declares the keyword argument k twice")]
    fn test_declaring_a_keyword_twice_panics() {
        let _ = FunctionDefinition::new("f", Type::any(), vec![])
            .keyword("k", Type::any(), 1u64)
            .keyword("k", Type::any(), 2u64);
    }

    #[test]
    fn test_type_shows_keywords() {
        let definition =
            FunctionDefinition::new("greet", Type::string(), vec![Argument::string("name")])
                .keyword("greeting", Type::string(), Value::String("Hello".into()));
        assert_eq!(
            definition.r#type().to_string(),
            "fn(String, greeting: String) -> String"
        );
    }
}
