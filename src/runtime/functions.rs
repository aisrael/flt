//! Functions in the runtime

use std::fmt;

use crate::errors::RuntimeError;
use crate::runtime::types::Type;
use crate::runtime::Value;
use crate::Error;

/// The handler backing a native function: a Rust closure, which may capture host state.
pub type NativeFn = Box<dyn Fn(&[Value]) -> Result<Value, Error> + Send + Sync>;

/// A native function pairs a type signature with a Rust implementation.
pub struct NativeFunction {
    pub definition: FunctionDefinition,
    handler: NativeFn,
}

impl NativeFunction {
    /// Create a new native function from a definition and a handler.
    pub fn new(
        definition: FunctionDefinition,
        handler: impl Fn(&[Value]) -> Result<Value, Error> + Send + Sync + 'static,
    ) -> Self {
        Self {
            definition,
            handler: Box::new(handler),
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

    /// Invoke the function with the given evaluated arguments, after checking their number
    /// against the function's signatures.
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        self.definition.check_arity(args.len())?;
        (self.handler)(args)
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

    /// Check that some overload takes `found` arguments.
    pub fn check_arity(&self, found: usize) -> Result<(), Error> {
        if self
            .overloads
            .iter()
            .any(|overload| overload.arguments.len() == found)
        {
            return Ok(());
        }
        Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch {
            name: self.name.clone(),
            expected: self
                .overloads
                .first()
                .map_or(0, |overload| overload.arguments.len()),
            found,
        }))
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
}
