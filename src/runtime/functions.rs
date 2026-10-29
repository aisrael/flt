//! Functions in the runtime

use std::cmp::Reverse;
use std::fmt;

use bigdecimal::num_bigint::BigInt;
use bigdecimal::BigDecimal;

use crate::errors::ArgumentCount;
use crate::errors::ArgumentCounts;
use crate::errors::HandlerError;
use crate::errors::RuntimeError;
use crate::runtime::native::NativeType;
use crate::runtime::types::Type;
use crate::runtime::unexpected_type;
use crate::runtime::Map;
use crate::runtime::MapKey;
use crate::runtime::Value;
use crate::Error;

/// The handler backing a native function: a Rust closure, which may capture host state. It
/// gets the call's [`Arguments`], and reads them by name.
pub type NativeFn = Box<dyn Fn(&Arguments) -> Result<Value, Error> + Send + Sync>;

/// A native function pairs a type signature with a Rust implementation.
pub struct NativeFunction {
    pub definition: FunctionDefinition,
    handler: NativeFn,
}

impl NativeFunction {
    /// Create a new native function from a definition and a handler, with or without keyword
    /// arguments.
    pub fn new(
        definition: FunctionDefinition,
        handler: impl Fn(&Arguments) -> Result<Value, Error> + Send + Sync + 'static,
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

    /// Invoke the function with the given evaluated arguments and no keyword arguments,
    /// after checking their number against the function's signatures.
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        self.call_with_keywords(args, Map::new())
    }

    /// Invoke the function with evaluated positional and keyword arguments. The overload is
    /// picked by the arguments' count and types and by the keywords
    /// ([`FunctionDefinition::resolve`]), and the result is checked against its return type.
    ///
    /// Each value is converted for the type the overload declares for it, before the handler
    /// runs, and the result after, so a `Number` argument or result is never an `Integer`.
    pub fn call_with_keywords(&self, args: &[Value], keywords: Map) -> Result<Value, Error> {
        let signature = self.definition.resolve(args, &keywords)?;
        let keywords = signature.bind_keywords(&self.definition.name, keywords)?;
        let args: Vec<Value> = args
            .iter()
            .enumerate()
            .map(|(i, value)| match signature.argument_type(i) {
                Some(r#type) => r#type.coerce(value.clone()),
                None => value.clone(),
            })
            .collect();
        let keywords: Map = keywords
            .into_iter()
            .map(
                |(key, value)| match signature.keyword_type(key_name(&key)) {
                    Some(r#type) => {
                        let value = r#type.coerce(value);
                        (key, value)
                    }
                    None => (key, value),
                },
            )
            .collect();
        let result = (self.handler)(&Arguments {
            function_name: &self.definition.name,
            signature,
            positional: &args,
            keywords: &keywords,
        })?;
        if signature.return_type.accepts(&result) {
            Ok(signature.return_type.coerce(result))
        } else {
            Err(Error::RuntimeError(RuntimeError::ReturnTypeMismatch {
                name: self.definition.name.clone(),
                expected: signature.return_type.to_string(),
                found: type_name(&result),
            }))
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

/// A call's arguments, as a native function's handler gets them: the positional values, the
/// keywords bound to the overload that matched, and that overload, which names them. A handler
/// reads each argument by its name, whether the call passed it positionally or as a keyword.
#[derive(Clone, Copy)]
pub struct Arguments<'a> {
    function_name: &'a str,
    signature: &'a FunctionSignature,
    positional: &'a [Value],
    keywords: &'a Map,
}

impl<'a> Arguments<'a> {
    /// The argument or declared keyword called `name`, converted to `T`. It raises
    /// `Expected <type>, found <type>` or `Cannot convert <number> to <type>` if the value
    /// doesn't convert, and a [`HandlerError`] if the overload has no single value of that name.
    pub fn get<T: FromValue<'a>>(&self, name: &str) -> Result<T, Error> {
        T::from_value(self.lookup(name)?)
    }

    /// Borrows the native value of the argument or declared keyword called `name`, raising
    /// `Expected <T::NAME>, found <type>` if it isn't a `T`, and a [`HandlerError`] if the
    /// overload has no single value of that name.
    pub fn native<T: NativeType>(&self, name: &str) -> Result<&'a T, Error> {
        self.lookup(name)?.downcast_native::<T>()
    }

    /// The value of the argument or declared keyword called `name`, or `None` if the overload
    /// that matched has none: it declares no such name, or it's the variadic argument, whose
    /// values are in [`Arguments::rest`].
    pub fn value(&self, name: &str) -> Option<&'a Value> {
        self.lookup(name).ok()
    }

    /// The values of the variadic last argument, empty if the overload has none.
    pub fn rest(&self) -> &'a [Value] {
        if self.signature.is_variadic() {
            &self.positional[self.signature.minimum_arguments()..]
        } else {
            &[]
        }
    }

    /// The keyword arguments: every declared keyword, in declaration order, with the call's
    /// value or else its default, then any rest keywords in call order.
    pub fn keywords(&self) -> &'a Map {
        self.keywords
    }

    /// Every positional value, including the variadic argument's.
    pub fn positional(&self) -> &'a [Value] {
        self.positional
    }

    /// The name of the function being called.
    pub fn function_name(&self) -> &'a str {
        self.function_name
    }

    /// The overload that matched the call.
    pub fn signature(&self) -> &'a FunctionSignature {
        self.signature
    }

    /// The positional argument called `name`, else the declared keyword
    fn lookup(&self, name: &str) -> Result<&'a Value, Error> {
        let arguments = &self.signature.arguments;
        let value = match arguments.iter().position(|argument| argument.name == name) {
            Some(i) if arguments[i].variadic => {
                return Err(self.handler_error(name, |name, argument| {
                    HandlerError::VariadicArgument { name, argument }
                }));
            }
            Some(i) => self.positional.get(i),
            None if self.signature.declares_keyword(name) => {
                self.keywords.get(&MapKey::symbol(name))
            }
            None => None,
        };
        value.ok_or_else(|| {
            self.handler_error(name, |name, argument| HandlerError::UndeclaredArgument {
                name,
                argument,
            })
        })
    }

    fn handler_error(
        &self,
        argument: &str,
        error: impl FnOnce(String, String) -> HandlerError,
    ) -> Error {
        Error::HandlerError(error(self.function_name.to_string(), argument.to_string()))
    }
}

/// A Rust type that a handler can read an argument as, with [`Arguments::get`]. A conversion
/// that fails raises an error, e.g. `Expected String, found Number`.
pub trait FromValue<'a>: Sized {
    fn from_value(value: &'a Value) -> Result<Self, Error>;
}

/// The value as it is
impl<'a> FromValue<'a> for &'a Value {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        Ok(value)
    }
}

impl<'a> FromValue<'a> for &'a str {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => Ok(s),
            other => Err(unexpected_type(&Type::string().to_string(), other)),
        }
    }
}

impl<'a> FromValue<'a> for String {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        <&str>::from_value(value).map(str::to_string)
    }
}

impl<'a> FromValue<'a> for bool {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        match value {
            Value::Boolean(b) => Ok(*b),
            other => Err(unexpected_type(&Type::boolean().to_string(), other)),
        }
    }
}

impl<'a> FromValue<'a> for &'a Map {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        match value {
            Value::Map(map) => Ok(map),
            other => Err(unexpected_type(&Type::map().to_string(), other)),
        }
    }
}

/// An array's elements
impl<'a> FromValue<'a> for &'a [Value] {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        match value {
            Value::Array(values) => Ok(values),
            other => Err(unexpected_type(&Type::array().to_string(), other)),
        }
    }
}

/// `None` for `None`, and otherwise the value converted to `T`, for an `Option<…>` argument
impl<'a, T: FromValue<'a>> FromValue<'a> for Option<T> {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        match value {
            Value::None => Ok(None),
            other => T::from_value(other).map(Some),
        }
    }
}

/// Rust number types convert as their `TryFrom<&Value>` does
macro_rules! from_value_for_numbers {
    ($($t:ty),*) => {$(
        impl<'a> FromValue<'a> for $t {
            fn from_value(value: &'a Value) -> Result<Self, Error> {
                <$t>::try_from(value)
            }
        }
    )*};
}

from_value_for_numbers!(i32, i64, u32, u64, usize, f64);

/// An `Integer`, or a whole `Number`, as a `BigInt`
impl<'a> FromValue<'a> for BigInt {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        value.to_big_int(&Type::integer().to_string(), "BigInt")
    }
}

/// A `Number`, or an `Integer`, as a `BigDecimal`
impl<'a> FromValue<'a> for BigDecimal {
    fn from_value(value: &'a Value) -> Result<Self, Error> {
        value
            .to_big_decimal()
            .ok_or_else(|| unexpected_type(&Type::number().to_string(), value))
    }
}

/// A function definition is a collection of function signatures (overloads)
pub struct FunctionDefinition {
    pub name: String,
    pub overloads: Vec<FunctionSignature>,
}

impl FunctionDefinition {
    /// Create a new function definition with a name, return type, and arguments
    ///
    /// # Panics
    ///
    /// If an argument other than the last is variadic.
    pub fn new<S: Into<String>>(name: S, return_type: Type, arguments: Vec<Argument>) -> Self {
        let name = name.into();
        assert_variadic_last(&name, &arguments);
        Self {
            name,
            overloads: vec![FunctionSignature::new(arguments, return_type)],
        }
    }

    /// Add a new overload to the function definition
    ///
    /// # Panics
    ///
    /// If an argument other than the last is variadic.
    pub fn add_overload(mut self, return_type: Type, arguments: Vec<Argument>) -> Self {
        assert_variadic_last(&self.name, &arguments);
        self.overloads
            .push(FunctionSignature::new(arguments, return_type));
        self
    }

    /// Add an optional keyword argument, with a default, to the most recently added overload.
    ///
    /// # Panics
    ///
    /// If that overload already declares a keyword with this name, or if the default isn't of
    /// the keyword's type.
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
        let default = default.into();
        assert!(
            r#type.accepts(&default),
            "Function {} declares the keyword argument {name} with a default of the wrong type",
            self.name
        );
        self.last_overload().keywords.push(KeywordArgument {
            name,
            r#type,
            default,
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

    /// The overload that accepts these positional arguments and keywords, by count and by
    /// type. Overloads are tried by arity, longest first. Among overloads of the same arity
    /// that all accept the call, the narrowest wins, and if none is narrowest, the one declared
    /// first.
    ///
    /// If no overload takes that many positional arguments, the error lists the counts they
    /// take. If some do, the error is the first way the call doesn't match the closest of them,
    /// the one that passes the most checks before one fails, and if several are equally close,
    /// `No overload of <name> accepts (…)`.
    pub fn resolve(&self, args: &[Value], keywords: &Map) -> Result<&FunctionSignature, Error> {
        let mut candidates: Vec<&FunctionSignature> = self
            .overloads
            .iter()
            .filter(|overload| overload.accepts_count(args.len()))
            .collect();
        // A stable sort, so overloads of the same arity stay in declaration order
        candidates.sort_by_key(|overload| Reverse(overload.minimum_arguments()));
        let mut mismatches = Vec::new();
        let mut matches: Vec<&FunctionSignature> = Vec::new();
        for overload in candidates {
            // Only overloads of the arity of the first match compete with it
            if matches
                .first()
                .is_some_and(|first| first.minimum_arguments() != overload.minimum_arguments())
            {
                break;
            }
            match overload.mismatch(&self.name, args, keywords) {
                None => matches.push(overload),
                Some(mismatch) => mismatches.push(mismatch),
            }
        }
        if let Some(first) = matches.first() {
            return Ok(narrowest(&matches, args.len(), keywords).unwrap_or(first));
        }
        let Some(closest) = mismatches.iter().map(|mismatch| mismatch.progress).max() else {
            return Err(Error::RuntimeError(self.count_mismatch(args.len())));
        };
        let mut closest = mismatches
            .into_iter()
            .filter(|mismatch| mismatch.progress == closest);
        Err(Error::RuntimeError(
            match (closest.next(), closest.next()) {
                (Some(only), None) => only.error,
                _ => self.no_matching_overload(args, keywords),
            },
        ))
    }

    /// Check that some overload takes `found` positional arguments.
    pub fn check_arity(&self, found: usize) -> Result<(), Error> {
        if self
            .overloads
            .iter()
            .any(|overload| overload.accepts_count(found))
        {
            Ok(())
        } else {
            Err(Error::RuntimeError(self.count_mismatch(found)))
        }
    }

    /// The error for a call with `found` positional arguments, which no overload takes
    fn count_mismatch(&self, found: usize) -> RuntimeError {
        RuntimeError::ArgumentCountMismatch {
            name: self.name.clone(),
            expected: ArgumentCounts::new(
                self.overloads.iter().map(FunctionSignature::argument_count),
            ),
            found,
        }
    }

    /// The error for a call that several overloads take by count, none of them closer than
    /// the others: it lists the types of the call's positional arguments, then its keywords
    fn no_matching_overload(&self, args: &[Value], keywords: &Map) -> RuntimeError {
        RuntimeError::NoMatchingOverload {
            name: self.name.clone(),
            found: args
                .iter()
                .map(type_name)
                .chain(
                    keywords
                        .iter()
                        .map(|(key, value)| format!("{}: {}", key_name(key), type_name(value))),
                )
                .collect(),
        }
    }
}

/// How far a call got through a signature's checks before one failed. A signature that gets
/// further is closer to the call.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Progress {
    /// The checks that passed: positional arguments first, then keywords in call order
    passed: usize,
    /// Whether the check that failed is for a keyword the signature accepts, with a value of
    /// the wrong type
    accepts_keyword: bool,
}

/// The first way a call doesn't match a signature, and how far it got before that
struct Mismatch {
    progress: Progress,
    error: RuntimeError,
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

    /// Whether the signature declares a keyword argument called `name`
    fn declares_keyword(&self, name: &str) -> bool {
        self.keywords.iter().any(|keyword| keyword.name == name)
    }

    /// Whether the last argument is variadic
    fn is_variadic(&self) -> bool {
        self.arguments
            .last()
            .is_some_and(|argument| argument.variadic)
    }

    /// How many positional arguments a call needs at least: all of them, except a variadic
    /// last one
    fn minimum_arguments(&self) -> usize {
        self.arguments.len() - usize::from(self.is_variadic())
    }

    /// How many positional arguments the signature takes
    fn argument_count(&self) -> ArgumentCount {
        if self.is_variadic() {
            ArgumentCount::AtLeast(self.minimum_arguments())
        } else {
            ArgumentCount::Exactly(self.arguments.len())
        }
    }

    /// The first way a call that this signature takes by count doesn't match it, checking the
    /// positional arguments in order, then the keywords in call order, or `None` if it matches
    fn mismatch(&self, function_name: &str, args: &[Value], keywords: &Map) -> Option<Mismatch> {
        for (i, value) in args.iter().enumerate() {
            // Values past the last argument belong to the variadic one
            let argument = self.arguments.get(i).or(self.arguments.last())?;
            if !argument.r#type.accepts(value) {
                return Some(Mismatch {
                    progress: Progress {
                        passed: i,
                        accepts_keyword: false,
                    },
                    error: RuntimeError::ArgumentTypeMismatch {
                        name: function_name.to_string(),
                        argument: argument.name.clone(),
                        expected: argument.r#type.to_string(),
                        found: type_name(value),
                    },
                });
            }
        }
        for (i, (key, value)) in keywords.iter().enumerate() {
            let key = key_name(key);
            let passed = args.len() + i;
            let declared = self.keywords.iter().find(|keyword| keyword.name == key);
            let r#type = match (declared, &self.rest_keywords) {
                (Some(keyword), _) => &keyword.r#type,
                (None, Some(rest)) => rest,
                (None, None) => {
                    return Some(Mismatch {
                        progress: Progress {
                            passed,
                            accepts_keyword: false,
                        },
                        error: RuntimeError::UnexpectedKeywordArgument {
                            name: function_name.to_string(),
                            key: key.to_string(),
                        },
                    })
                }
            };
            if !r#type.accepts(value) {
                return Some(Mismatch {
                    progress: Progress {
                        passed,
                        accepts_keyword: true,
                    },
                    error: RuntimeError::KeywordTypeMismatch {
                        name: function_name.to_string(),
                        key: key.to_string(),
                        expected: r#type.to_string(),
                        found: type_name(value),
                    },
                });
            }
        }
        None
    }

    /// The type of the `i`th positional value of a call, or `None` past the last argument of
    /// a signature without a variadic one
    fn argument_type(&self, i: usize) -> Option<&Type> {
        match self.arguments.get(i) {
            Some(argument) => Some(&argument.r#type),
            None => self
                .arguments
                .last()
                .filter(|argument| argument.variadic)
                .map(|argument| &argument.r#type),
        }
    }

    /// The type of the keyword `name`: the declared keyword's, or else the rest keywords', if
    /// the signature takes rest keywords
    fn keyword_type(&self, name: &str) -> Option<&Type> {
        self.keywords
            .iter()
            .find(|keyword| keyword.name == name)
            .map(|keyword| &keyword.r#type)
            .or(self.rest_keywords.as_ref())
    }

    /// Whether, for a call with `count` positional values and these keywords, each of this
    /// signature's types is at least as narrow as `other`'s
    fn is_at_least_as_narrow_as(&self, other: &Self, count: usize, keywords: &Map) -> bool {
        let positional =
            (0..count).all(|i| match (self.argument_type(i), other.argument_type(i)) {
                (Some(this), Some(that)) => this.is_at_least_as_narrow_as(that),
                _ => false,
            });
        positional
            && keywords.keys().map(key_name).all(|name| {
                match (self.keyword_type(name), other.keyword_type(name)) {
                    (Some(this), Some(that)) => this.is_at_least_as_narrow_as(that),
                    _ => false,
                }
            })
    }

    /// Whether a call with `found` positional arguments fits this signature
    fn accepts_count(&self, found: usize) -> bool {
        if self.is_variadic() {
            found >= self.minimum_arguments()
        } else {
            found == self.arguments.len()
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

/// The overload among `matches`, which all match a call with `count` positional values and
/// these keywords, that's narrower than every other: at least as narrow for each value the call
/// passes, and strictly narrower for one. `None` if there isn't one.
fn narrowest<'a>(
    matches: &[&'a FunctionSignature],
    count: usize,
    keywords: &Map,
) -> Option<&'a FunctionSignature> {
    matches.iter().copied().find(|candidate| {
        matches.iter().all(|other| {
            std::ptr::eq(*candidate, *other)
                || (candidate.is_at_least_as_narrow_as(other, count, keywords)
                    && !other.is_at_least_as_narrow_as(candidate, count, keywords))
        })
    })
}

/// The name of a value's type, for error messages
fn type_name(value: &Value) -> String {
    value
        .type_of()
        .map_or_else(|error| error.to_string(), |r#type| r#type.to_string())
}

/// A keyword's name, without the `:` of its symbol key
fn key_name(key: &MapKey) -> &str {
    let (MapKey::Symbol(name) | MapKey::String(name)) = key;
    name
}

/// Panics if an argument other than the last is variadic, which is a mistake in host code.
fn assert_variadic_last(function_name: &str, arguments: &[Argument]) {
    let misplaced = arguments
        .iter()
        .rev()
        .skip(1)
        .find(|argument| argument.variadic);
    if let Some(argument) = misplaced {
        panic!(
            "Function {function_name} has a variadic argument {} that isn't its last argument",
            argument.name
        );
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
    /// Whether the argument takes zero or more trailing values. Only the last argument of a
    /// signature can be variadic.
    pub variadic: bool,
}

impl Argument {
    pub fn new<S: Into<String>>(name: S, r#type: Type) -> Self {
        Self {
            name: name.into(),
            r#type,
            variadic: false,
        }
    }

    /// An argument that takes zero or more trailing values, each of the given type. Only the
    /// last argument of a signature can be variadic.
    pub fn variadic<S: Into<String>>(name: S, r#type: Type) -> Self {
        Self {
            variadic: true,
            ..Self::new(name, r#type)
        }
    }

    /// A convenience method for an argument with the built-in integer type
    pub fn integer<S: Into<String>>(name: S) -> Self {
        Self::new(name, Type::integer())
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

    use std::str::FromStr;

    use bigdecimal::num_bigint::BigInt;
    use bigdecimal::BigDecimal;

    use super::*;
    use crate::runtime::types::FunctionType;

    fn identity() -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::any(),
                vec![Argument::new("v", Type::any())],
            ),
            |args| Ok(args.get::<&Value>("v")?.clone()),
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
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected, found: 2 }))
                if name == "identity" && expected.counts() == [ArgumentCount::Exactly(1)]
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
        NativeFunction::new(definition, |args| Ok(Value::Map(args.keywords().clone())))
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

    /// A definition with `fixed` arguments before a variadic one
    fn variadic_definition(fixed: usize) -> FunctionDefinition {
        let mut arguments: Vec<Argument> = (0..fixed)
            .map(|i| Argument::new(format!("a{i}"), Type::any()))
            .collect();
        arguments.push(Argument::variadic("rest", Type::any()));
        FunctionDefinition::new("f", Type::any(), arguments)
    }

    #[test]
    fn test_variadic_accepts_at_least_its_fixed_arguments() {
        for fixed in 0..=2 {
            let definition = variadic_definition(fixed);
            for found in fixed..fixed + 3 {
                assert!(
                    definition.check_arity(found).is_ok(),
                    "{fixed} fixed argument(s), {found} found"
                );
            }
        }
    }

    #[test]
    fn test_variadic_with_too_few_arguments() {
        for fixed in 1..=2 {
            for found in 0..fixed {
                let result = variadic_definition(fixed).check_arity(found);
                assert!(
                    matches!(
                        &result,
                        Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch { name, expected, found: f }))
                            if name == "f"
                                && expected.counts() == [ArgumentCount::AtLeast(fixed)]
                                && *f == found
                    ),
                    "{fixed} fixed argument(s), {found} found: {result:?}"
                );
            }
        }
        assert_eq!(
            variadic_definition(1)
                .check_arity(0)
                .unwrap_err()
                .to_string(),
            "Runtime Error: Function f expected at least 1 argument(s), found 0"
        );
    }

    #[test]
    fn test_variadic_handler_gets_every_value() {
        let function = NativeFunction::new(variadic_definition(1), |args| {
            Ok(Value::Array(args.positional().to_vec()))
        });
        let args = [number(1), number(2), number(3)];
        assert_eq!(function.call(&args).unwrap(), Value::Array(args.to_vec()));
        assert_eq!(
            function.call(&args[..1]).unwrap(),
            Value::Array(vec![number(1)])
        );
    }

    #[test]
    fn test_variadic_rejects_undeclared_keywords() {
        let function = NativeFunction::new(variadic_definition(0), |_args| Ok(Value::None));
        let result = function.call_with_keywords(&[number(1)], keywords(&[("k", number(2))]));
        assert!(matches!(
            result,
            Err(Error::RuntimeError(RuntimeError::UnexpectedKeywordArgument { name, key }))
                if name == "f" && key == "k"
        ));
    }

    #[test]
    fn test_variadic_takes_declared_keywords() {
        let function = NativeFunction::new(
            variadic_definition(0).keyword("k", Type::any(), 0u64),
            |args| {
                Ok(Value::Array(
                    args.rest()
                        .iter()
                        .chain(args.keywords().values())
                        .cloned()
                        .collect(),
                ))
            },
        );
        assert_eq!(
            function
                .call_with_keywords(&[], keywords(&[("k", number(2))]))
                .unwrap(),
            Value::Array(vec![number(2)])
        );
    }

    /// A variadic overload takes the counts that a fixed one doesn't
    #[test]
    fn test_fixed_and_variadic_overloads() {
        let definition = FunctionDefinition::new("f", Type::number(), vec![Argument::number("n")])
            .add_overload(
                Type::string(),
                vec![
                    Argument::number("n"),
                    Argument::number("m"),
                    Argument::variadic("rest", Type::any()),
                ],
            );
        let numbers = |found: u64| (0..found).map(number).collect::<Vec<Value>>();
        let resolve = |found: u64| definition.resolve(&numbers(found), &Map::new());
        assert_eq!(resolve(1).unwrap().return_type, Type::number());
        assert_eq!(resolve(2).unwrap().return_type, Type::string());
        assert_eq!(resolve(5).unwrap().return_type, Type::string());
        assert!(matches!(
            resolve(0),
            Err(Error::RuntimeError(RuntimeError::ArgumentCountMismatch {
                expected,
                found: 0,
                ..
            })) if expected.counts() == [ArgumentCount::Exactly(1), ArgumentCount::AtLeast(2)]
        ));
    }

    #[test]
    #[should_panic(
        expected = "Function f has a variadic argument rest that isn't its last argument"
    )]
    fn test_new_panics_on_a_variadic_argument_that_isnt_last() {
        FunctionDefinition::new(
            "f",
            Type::any(),
            vec![
                Argument::variadic("rest", Type::any()),
                Argument::number("n"),
            ],
        );
    }

    #[test]
    #[should_panic(
        expected = "Function f has a variadic argument rest that isn't its last argument"
    )]
    fn test_add_overload_panics_on_a_variadic_argument_that_isnt_last() {
        FunctionDefinition::new("f", Type::any(), vec![]).add_overload(
            Type::any(),
            vec![
                Argument::variadic("rest", Type::any()),
                Argument::number("n"),
            ],
        );
    }

    #[test]
    fn test_type_shows_variadic() {
        assert_eq!(
            variadic_definition(1).r#type().to_string(),
            "fn(Any, *Any) -> Any"
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

    fn string(s: &str) -> Value {
        Value::String(s.into())
    }

    fn symbol(s: &str) -> Value {
        Value::Symbol(s.into())
    }

    /// The `which` keyword's value: each overload of a function under test declares `which`
    /// with a default that names the overload, so the call shows which one matched.
    fn which(function: &NativeFunction, args: &[Value], call_keywords: &[(&str, Value)]) -> String {
        match function.call_with_keywords(args, keywords(call_keywords)) {
            Ok(Value::Map(map)) => match &map[&MapKey::symbol("which")] {
                Value::String(name) => name.clone(),
                other => panic!("which is {other}"),
            },
            other => panic!("{} returned {other:?}", function.name()),
        }
    }

    /// The error a call raises, as it's displayed
    fn error(function: &NativeFunction, args: &[Value], call_keywords: &[(&str, Value)]) -> String {
        function
            .call_with_keywords(args, keywords(call_keywords))
            .expect_err("the call should fail")
            .to_string()
    }

    /// pick(*args: Any, which = "arity 0") and pick(a: Number, *args: Any, which = "arity 1"),
    /// declared in that order, or reversed
    fn pick(reversed: bool) -> NativeFunction {
        let arity_0 = (vec![Argument::variadic("args", Type::any())], "arity 0");
        let arity_1 = (
            vec![
                Argument::number("a"),
                Argument::variadic("args", Type::any()),
            ],
            "arity 1",
        );
        let overloads = if reversed {
            [arity_1, arity_0]
        } else {
            [arity_0, arity_1]
        };
        let [(first, first_which), (second, second_which)] = overloads;
        echo(
            FunctionDefinition::new("pick", Type::map(), first)
                .keyword("which", Type::string(), string(first_which))
                .add_overload(Type::map(), second)
                .keyword("which", Type::string(), string(second_which)),
        )
    }

    #[test]
    fn test_overloads_are_tried_by_arity_longest_first() {
        for reversed in [false, true] {
            let function = pick(reversed);
            assert_eq!(which(&function, &[number(1), symbol("b")], &[]), "arity 1");
            assert_eq!(which(&function, &[number(1)], &[]), "arity 1");
            assert_eq!(which(&function, &[symbol("b")], &[]), "arity 0");
            assert_eq!(which(&function, &[], &[]), "arity 0");
        }
    }

    #[test]
    fn test_fixed_arity_beats_a_variadic_overload_with_fewer_fixed_arguments() {
        for reversed in [false, true] {
            let fixed = (vec![Argument::number("a"), Argument::number("b")], "fixed");
            let variadic = (
                vec![
                    Argument::number("a"),
                    Argument::variadic("rest", Type::any()),
                ],
                "variadic",
            );
            let overloads = if reversed {
                [variadic, fixed]
            } else {
                [fixed, variadic]
            };
            let [(first, first_which), (second, second_which)] = overloads;
            let function = echo(
                FunctionDefinition::new("f", Type::map(), first)
                    .keyword("which", Type::string(), string(first_which))
                    .add_overload(Type::map(), second)
                    .keyword("which", Type::string(), string(second_which)),
            );
            assert_eq!(which(&function, &[number(1), number(2)], &[]), "fixed");
            assert_eq!(which(&function, &[number(1)], &[]), "variadic");
            assert_eq!(which(&function, &[number(1), string("b")], &[]), "variadic");
            assert_eq!(
                which(&function, &[number(1), number(2), number(3)], &[]),
                "variadic"
            );
        }
    }

    /// describe(n: Number, which = "number") and describe(s: String | Symbol, which = "text")
    fn describe() -> NativeFunction {
        echo(
            FunctionDefinition::new("describe", Type::map(), vec![Argument::number("n")])
                .keyword("which", Type::string(), string("number"))
                .add_overload(
                    Type::map(),
                    vec![Argument::new(
                        "s",
                        Type::union(Type::string(), Type::symbol()),
                    )],
                )
                .keyword("which", Type::string(), string("text")),
        )
    }

    #[test]
    fn test_types_pick_among_overloads_of_the_same_arity() {
        let function = describe();
        assert_eq!(which(&function, &[number(1)], &[]), "number");
        assert_eq!(which(&function, &[string("a")], &[]), "text");
        assert_eq!(which(&function, &[symbol("a")], &[]), "text");
    }

    #[test]
    fn test_no_overload_accepts_the_arguments() {
        assert_eq!(
            error(&describe(), &[Value::Boolean(true)], &[]),
            "Runtime Error: No overload of describe accepts (Boolean)"
        );
    }

    /// foo(s: String, n: Number = 0, b: Boolean = false) and
    /// foo(s: String, t: String = "", b: Boolean = false), declared in that order, or reversed.
    /// Each also declares `which`, naming the keyword only it declares.
    fn foo(reversed: bool) -> NativeFunction {
        let with_n = |definition: FunctionDefinition| {
            definition
                .keyword("n", Type::number(), number(0))
                .keyword("b", Type::boolean(), Value::Boolean(false))
                .keyword("which", Type::string(), string("n"))
        };
        let with_t = |definition: FunctionDefinition| {
            definition
                .keyword("t", Type::string(), string(""))
                .keyword("b", Type::boolean(), Value::Boolean(false))
                .keyword("which", Type::string(), string("t"))
        };
        let (first, second): (
            &dyn Fn(FunctionDefinition) -> FunctionDefinition,
            &dyn Fn(FunctionDefinition) -> FunctionDefinition,
        ) = if reversed {
            (&with_t, &with_n)
        } else {
            (&with_n, &with_t)
        };
        let definition = first(FunctionDefinition::new(
            "foo",
            Type::map(),
            vec![Argument::string("s")],
        ));
        echo(second(
            definition.add_overload(Type::map(), vec![Argument::string("s")]),
        ))
    }

    #[test]
    fn test_keywords_pick_among_overloads_of_the_same_arity() {
        for reversed in [false, true] {
            let function = foo(reversed);
            let abc = [string("abc")];
            assert_eq!(which(&function, &abc, &[("n", number(1))]), "n");
            assert_eq!(
                which(
                    &function,
                    &abc,
                    &[("b", Value::Boolean(true)), ("n", number(1))]
                ),
                "n"
            );
            assert_eq!(which(&function, &abc, &[("t", string("x"))]), "t");
        }
    }

    #[test]
    fn test_a_tie_goes_to_the_overload_declared_first() {
        let tied = [("b", Value::Boolean(true))];
        assert_eq!(which(&foo(false), &[string("abc")], &tied), "n");
        assert_eq!(which(&foo(true), &[string("abc")], &tied), "t");
    }

    #[test]
    fn test_no_overload_accepts_the_keywords() {
        for reversed in [false, true] {
            let function = foo(reversed);
            assert_eq!(
                error(&function, &[string("abc")], &[("z", number(1))]),
                "Runtime Error: No overload of foo accepts (String, z: Integer)"
            );
            assert_eq!(
                error(&function, &[string("abc")], &[("b", number(1))]),
                "Runtime Error: No overload of foo accepts (String, b: Integer)"
            );
        }
    }

    /// head(items: Array, n: Number) and head(s: String, n: Number)
    fn head() -> NativeFunction {
        echo(
            FunctionDefinition::new(
                "head",
                Type::map(),
                vec![Argument::new("items", Type::array()), Argument::number("n")],
            )
            .add_overload(
                Type::map(),
                vec![Argument::string("s"), Argument::number("n")],
            ),
        )
    }

    #[test]
    fn test_the_error_names_the_overload_that_passes_the_most_checks() {
        assert_eq!(
            error(&head(), &[Value::Array(vec![]), string("x")], &[]),
            "Runtime Error: Argument n of head expects Number, found String"
        );
        assert_eq!(
            error(&head(), &[string("abc"), symbol("x")], &[]),
            "Runtime Error: Argument n of head expects Number, found Symbol"
        );
        for reversed in [false, true] {
            assert_eq!(
                error(
                    &foo(reversed),
                    &[string("abc")],
                    &[("n", number(1)), ("z", number(1))]
                ),
                "Runtime Error: Function foo got an unexpected keyword argument: z"
            );
        }
    }

    #[test]
    fn test_overloads_that_fail_at_the_same_argument_name_none() {
        assert_eq!(
            error(&head(), &[number(1), number(2)], &[]),
            "Runtime Error: No overload of head accepts (Integer, Integer)"
        );
    }

    #[test]
    fn test_a_keyword_of_the_wrong_type_is_closer_than_one_not_accepted() {
        for reversed in [false, true] {
            let function = foo(reversed);
            let abc = [string("abc")];
            assert_eq!(
                error(&function, &abc, &[("n", string("x"))]),
                "Runtime Error: Keyword argument n of foo expects Number, found String"
            );
            assert_eq!(
                error(&function, &abc, &[("t", number(1))]),
                "Runtime Error: Keyword argument t of foo expects String, found Integer"
            );
            assert_eq!(
                error(
                    &function,
                    &abc,
                    &[("b", Value::Boolean(true)), ("t", number(1))]
                ),
                "Runtime Error: Keyword argument t of foo expects String, found Integer"
            );
        }
    }

    #[test]
    fn test_rest_keywords_accept_a_keyword_of_the_wrong_type() {
        // opts(s: String, **kwargs: Number) and opts(s: String, flag: Boolean = false)
        let function = echo(
            FunctionDefinition::new("opts", Type::map(), vec![Argument::string("s")])
                .rest_keywords(Type::number())
                .add_overload(Type::map(), vec![Argument::string("s")])
                .keyword("flag", Type::boolean(), Value::Boolean(false)),
        );
        assert_eq!(
            error(&function, &[string("abc")], &[("x", string("y"))]),
            "Runtime Error: Keyword argument x of opts expects Number, found String"
        );
    }

    #[test]
    fn test_an_argument_of_the_wrong_type_is_named() {
        let pad = NativeFunction::new(
            FunctionDefinition::new(
                "pad",
                Type::string(),
                vec![Argument::string("s"), Argument::number("width")],
            ),
            |args| Ok(args.get::<&Value>("s")?.clone()),
        );
        assert_eq!(
            error(&pad, &[number(1), number(2)], &[]),
            "Runtime Error: Argument s of pad expects String, found Integer"
        );
        assert_eq!(
            error(&pad, &[string("x"), string("2")], &[]),
            "Runtime Error: Argument width of pad expects Number, found String"
        );
    }

    #[test]
    fn test_each_variadic_value_is_checked() {
        let total = NativeFunction::new(
            FunctionDefinition::new(
                "total",
                Type::number(),
                vec![Argument::variadic("values", Type::number())],
            ),
            |_args| Ok(number(0)),
        );
        assert!(total.call(&[number(1), number(2)]).is_ok());
        assert_eq!(
            error(&total, &[number(1), string("a")], &[]),
            "Runtime Error: Argument values of total expects Number, found String"
        );
    }

    #[test]
    fn test_native_arguments_are_checked() {
        use crate::runtime::native::fixtures::Meters;
        use crate::runtime::native::fixtures::Seconds;

        let length = NativeFunction::new(
            FunctionDefinition::new(
                "length",
                Type::number(),
                vec![Argument::new("m", Type::native::<Meters>())],
            ),
            |_args| Ok(number(0)),
        );
        assert!(length.call(&[Value::native(Meters(1))]).is_ok());
        assert_eq!(
            error(&length, &[Value::native(Seconds(1))], &[]),
            "Runtime Error: Argument m of length expects Meters, found Seconds"
        );
    }

    #[test]
    fn test_keyword_values_are_checked() {
        let greet =
            echo(
                FunctionDefinition::new("greet", Type::map(), vec![Argument::string("name")])
                    .keyword("greeting", Type::string(), string("Hello")),
            );
        assert_eq!(
            error(&greet, &[string("Ann")], &[("greeting", number(1))]),
            "Runtime Error: Keyword argument greeting of greet expects String, found Integer"
        );
        assert_eq!(
            unexpected_keyword(
                greet.call_with_keywords(&[string("Ann")], keywords(&[("tone", string("warm"))]))
            ),
            Some(("greet".to_string(), "tone".to_string()))
        );
    }

    #[test]
    fn test_rest_keywords_are_checked_against_their_type() {
        let scores = echo(
            FunctionDefinition::new("scores", Type::map(), vec![]).rest_keywords(Type::number()),
        );
        assert!(scores
            .call_with_keywords(&[], keywords(&[("a", number(1))]))
            .is_ok());
        assert_eq!(
            error(&scores, &[], &[("a", string("x"))]),
            "Runtime Error: Keyword argument a of scores expects Number, found String"
        );
    }

    #[test]
    fn test_the_return_value_is_checked() {
        let liar = NativeFunction::new(
            FunctionDefinition::new("liar", Type::number(), vec![]),
            |_args| Ok(string("not a number")),
        );
        assert_eq!(
            error(&liar, &[], &[]),
            "Runtime Error: Function liar returned String, expected Number"
        );
    }

    #[test]
    fn test_the_return_value_is_checked_against_the_overload_that_matched() {
        let function = NativeFunction::new(
            FunctionDefinition::new("f", Type::number(), vec![Argument::number("n")])
                .add_overload(Type::string(), vec![Argument::string("s")]),
            |args| Ok(args.positional()[0].clone()),
        );
        assert_eq!(function.call(&[number(1)]).unwrap(), number(1));
        assert_eq!(function.call(&[string("a")]).unwrap(), string("a"));
    }

    /// A function `f` with an overload for each list of argument names, each argument `Any`;
    /// a name starting with `*` is variadic
    fn counts(overloads: &[&[&str]]) -> NativeFunction {
        let arguments = |names: &[&str]| -> Vec<Argument> {
            names
                .iter()
                .map(|name| match name.strip_prefix('*') {
                    Some(name) => Argument::variadic(name, Type::any()),
                    None => Argument::new(*name, Type::any()),
                })
                .collect()
        };
        let mut definition = FunctionDefinition::new("f", Type::any(), arguments(overloads[0]));
        for overload in &overloads[1..] {
            definition = definition.add_overload(Type::any(), arguments(overload));
        }
        NativeFunction::new(definition, |_args| Ok(Value::None))
    }

    #[test]
    fn test_count_errors_list_every_accepted_count() {
        let cases: &[(&[&[&str]], usize, &str)] = &[
            (&[&["a"]], 0, "1"),
            (&[&["a", "*rest"]], 0, "at least 1"),
            (&[&["a"], &["a", "b"]], 3, "1 or 2"),
            (&[&["a", "b"], &["a"]], 3, "1 or 2"),
            (&[&["a"], &["a", "b", "*rest"]], 0, "1 or at least 2"),
            (&[&["a", "b", "*rest"], &["a"]], 0, "1 or at least 2"),
            (
                &[&["a"], &["a", "b"], &["a", "b", "c", "*rest"]],
                0,
                "1, 2 or at least 3",
            ),
            (&[&["a", "*rest"], &["a", "b"]], 0, "at least 1"),
        ];
        for (overloads, found, expected) in cases {
            let args = vec![Value::None; *found];
            assert_eq!(
                error(&counts(overloads), &args, &[]),
                format!("Runtime Error: Function f expected {expected} argument(s), found {found}"),
                "overloads {overloads:?}"
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "Function f declares the keyword argument k with a default of the wrong type"
    )]
    fn test_a_keyword_default_of_the_wrong_type_panics() {
        let _ = FunctionDefinition::new("f", Type::any(), vec![]).keyword(
            "k",
            Type::number(),
            string("one"),
        );
    }

    /// Calls a function of `definition` whose handler is `handler`
    fn call(
        definition: FunctionDefinition,
        args: &[Value],
        call_keywords: &[(&str, Value)],
        handler: impl Fn(&Arguments) -> Result<Value, Error> + Send + Sync + 'static,
    ) -> Result<Value, Error> {
        NativeFunction::new(definition, handler).call_with_keywords(args, keywords(call_keywords))
    }

    #[test]
    fn test_get_reads_positional_arguments_by_name() {
        let pad = FunctionDefinition::new(
            "pad",
            Type::string(),
            vec![Argument::string("s"), Argument::number("width")],
        );
        let result = call(pad, &[string("ab"), number(4)], &[], |args| {
            let width: usize = args.get("width")?;
            let s: &str = args.get("s")?;
            Ok(string(&format!("{s:>width$}")))
        });
        assert_eq!(result.unwrap(), string("  ab"));
    }

    /// greet(name, greeting = "Hello") returns "<greeting>, <name>"
    fn greet() -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new("greet", Type::string(), vec![Argument::string("name")])
                .keyword("greeting", Type::string(), string("Hello")),
            |args| {
                let name: &str = args.get("name")?;
                let greeting: &str = args.get("greeting")?;
                Ok(string(&format!("{greeting}, {name}")))
            },
        )
    }

    #[test]
    fn test_get_reads_declared_keywords_with_their_defaults() {
        assert_eq!(
            greet().call(&[string("Ann")]).unwrap(),
            string("Hello, Ann")
        );
        assert_eq!(
            greet()
                .call_with_keywords(&[string("Ann")], keywords(&[("greeting", string("Hi"))]))
                .unwrap(),
            string("Hi, Ann")
        );
    }

    /// repeat(s, n) and repeat(s, n = 2): `n` is positional in one overload, a keyword in the
    /// other
    fn repeat() -> NativeFunction {
        NativeFunction::new(
            FunctionDefinition::new(
                "repeat",
                Type::string(),
                vec![Argument::string("s"), Argument::number("n")],
            )
            .add_overload(Type::string(), vec![Argument::string("s")])
            .keyword("n", Type::number(), number(2)),
            |args| {
                let s: &str = args.get("s")?;
                let n: usize = args.get("n")?;
                Ok(string(&s.repeat(n)))
            },
        )
    }

    #[test]
    fn test_an_argument_is_read_by_name_whether_positional_or_keyword() {
        let ab = [string("ab")];
        assert_eq!(
            repeat().call(&[string("ab"), number(3)]).unwrap(),
            string("ababab")
        );
        assert_eq!(repeat().call(&ab).unwrap(), string("abab"));
        assert_eq!(
            repeat()
                .call_with_keywords(&ab, keywords(&[("n", number(3))]))
                .unwrap(),
            string("ababab")
        );
    }

    #[test]
    fn test_from_value_converts_to_rust_types() {
        let map = keywords(&[("k", number(1))]);
        let array = vec![number(1), string("a")];
        assert_eq!(<&Value>::from_value(&number(1)).unwrap(), &number(1));
        assert_eq!(<&str>::from_value(&string("a")).unwrap(), "a");
        assert_eq!(String::from_value(&string("a")).unwrap(), "a");
        assert!(bool::from_value(&Value::Boolean(true)).unwrap());
        assert_eq!(<&Map>::from_value(&Value::Map(map.clone())).unwrap(), &map);
        assert_eq!(
            <&[Value]>::from_value(&Value::Array(array.clone())).unwrap(),
            array.as_slice()
        );
        assert_eq!(i32::from_value(&Value::from(-3i64)).unwrap(), -3);
        assert_eq!(i64::from_value(&Value::from(-3i64)).unwrap(), -3);
        assert_eq!(u32::from_value(&number(3)).unwrap(), 3);
        assert_eq!(u64::from_value(&number(3)).unwrap(), 3);
        assert_eq!(usize::from_value(&number(3)).unwrap(), 3);
        assert_eq!(f64::from_value(&number(3)).unwrap(), 3.0);
    }

    #[test]
    fn test_from_value_reads_none_as_none_for_an_option() {
        assert_eq!(Option::<i64>::from_value(&Value::None).unwrap(), None);
        assert_eq!(Option::<i64>::from_value(&number(2)).unwrap(), Some(2));
        assert_eq!(Option::<&str>::from_value(&string("a")).unwrap(), Some("a"));
    }

    #[test]
    fn test_from_value_raises_an_error_for_a_value_that_does_not_convert() {
        let error = |result: Result<(), Error>| result.unwrap_err().to_string();
        let cases = [
            (
                error(<&str>::from_value(&number(1)).map(drop)),
                "Expected String, found Integer",
            ),
            (
                error(String::from_value(&symbol("a")).map(drop)),
                "Expected String, found Symbol",
            ),
            (
                error(bool::from_value(&string("true")).map(drop)),
                "Expected Boolean, found String",
            ),
            (
                error(<&Map>::from_value(&Value::Array(vec![])).map(drop)),
                "Expected Map, found Array",
            ),
            (
                error(<&[Value]>::from_value(&Value::Map(Map::new())).map(drop)),
                "Expected Array, found Map",
            ),
            (
                error(usize::from_value(&Value::from(-1i64)).map(drop)),
                "Cannot convert -1 to usize",
            ),
            (
                error(Option::<&str>::from_value(&number(1)).map(drop)),
                "Expected String, found Integer",
            ),
        ];
        for (found, expected) in cases {
            assert_eq!(found, format!("Runtime Error: {expected}"));
        }
    }

    #[test]
    fn test_get_raises_the_conversion_error() {
        let shout = FunctionDefinition::new(
            "shout",
            Type::string(),
            vec![Argument::new("v", Type::any())],
        );
        let result = call(shout, &[number(1)], &[], |args| {
            Ok(string(&args.get::<&str>("v")?.to_uppercase()))
        });
        assert_eq!(
            result.unwrap_err().to_string(),
            "Runtime Error: Expected String, found Integer"
        );
    }

    #[test]
    fn test_native_borrows_a_native_value() {
        use crate::runtime::native::fixtures::Meters;
        use crate::runtime::native::fixtures::Seconds;

        let length = || {
            FunctionDefinition::new(
                "length",
                Type::number(),
                vec![Argument::new("m", Type::any())],
            )
        };
        let handler = |args: &Arguments| Ok(Value::from(args.native::<Meters>("m")?.0));
        assert_eq!(
            call(length(), &[Value::native(Meters(5))], &[], handler).unwrap(),
            Value::from(5i64)
        );
        assert_eq!(
            call(length(), &[Value::native(Seconds(5))], &[], handler)
                .unwrap_err()
                .to_string(),
            "Runtime Error: Expected Meters, found Seconds"
        );
    }

    #[test]
    fn test_value_tells_overloads_apart_by_their_names() {
        let describe = NativeFunction::new(
            FunctionDefinition::new("describe", Type::string(), vec![Argument::number("n")])
                .add_overload(Type::string(), vec![Argument::string("s")]),
            |args| {
                let kind = match (args.value("n"), args.value("s")) {
                    (Some(_), None) => "number",
                    (None, Some(_)) => "text",
                    _ => "both or neither",
                };
                Ok(string(kind))
            },
        );
        assert_eq!(describe.call(&[number(1)]).unwrap(), string("number"));
        assert_eq!(describe.call(&[string("a")]).unwrap(), string("text"));
    }

    #[test]
    fn test_rest_gives_the_variadic_values() {
        let function = NativeFunction::new(variadic_definition(1), |args| {
            Ok(Value::Array(args.rest().to_vec()))
        });
        assert_eq!(
            function.call(&[number(1), number(2), number(3)]).unwrap(),
            Value::Array(vec![number(2), number(3)])
        );
        assert_eq!(function.call(&[number(1)]).unwrap(), Value::Array(vec![]));
    }

    #[test]
    fn test_rest_is_empty_without_a_variadic_argument() {
        let result = call(
            FunctionDefinition::new("f", Type::any(), vec![Argument::number("n")]),
            &[number(1)],
            &[],
            |args| Ok(Value::Array(args.rest().to_vec())),
        );
        assert_eq!(result.unwrap(), Value::Array(vec![]));
    }

    #[test]
    fn test_rest_keywords_are_in_keywords_but_have_no_names() {
        let options =
            || FunctionDefinition::new("options", Type::any(), vec![]).rest_keywords(Type::any());
        let a = [("a", number(1))];
        assert_eq!(
            call(options(), &[], &a, |args| Ok(Value::Map(
                args.keywords().clone()
            )))
            .unwrap(),
            Value::Map(keywords(&a))
        );
        assert!(matches!(
            call(options(), &[], &a, |args| Ok(args.get::<&Value>("a")?.clone())),
            Err(Error::HandlerError(HandlerError::UndeclaredArgument { name, argument }))
                if name == "options" && argument == "a"
        ));
    }

    #[test]
    fn test_positional_function_name_and_signature() {
        let definition = FunctionDefinition::new("f", Type::any(), vec![Argument::number("a")])
            .add_overload(
                Type::any(),
                vec![Argument::number("a"), Argument::number("b")],
            );
        let result = call(definition, &[number(1), number(2)], &[], |args| {
            Ok(Value::Array(vec![
                string(args.function_name()),
                Value::Array(args.positional().to_vec()),
                Value::from(args.signature().arguments.len()),
            ]))
        });
        assert_eq!(
            result.unwrap(),
            Value::Array(vec![
                string("f"),
                Value::Array(vec![number(1), number(2)]),
                number(2),
            ])
        );
    }

    #[test]
    fn test_reading_an_undeclared_name_is_a_handler_error() {
        let typo = || FunctionDefinition::new("typo", Type::any(), vec![Argument::string("name")]);
        let ann = [string("Ann")];
        let via_get = call(typo(), &ann, &[], |args| {
            Ok(args.get::<&Value>("nmae")?.clone())
        });
        assert!(matches!(
            &via_get,
            Err(Error::HandlerError(HandlerError::UndeclaredArgument { name, argument }))
                if name == "typo" && argument == "nmae"
        ));
        assert_eq!(
            via_get.unwrap_err().to_string(),
            "Handler Error: Function typo has no argument nmae"
        );
        let via_native = call(typo(), &ann, &[], |args| {
            use crate::runtime::native::fixtures::Meters;
            Ok(Value::from(args.native::<Meters>("nmae")?.0))
        });
        assert_eq!(
            via_native.unwrap_err().to_string(),
            "Handler Error: Function typo has no argument nmae"
        );
        assert_eq!(
            call(typo(), &ann, &[], |args| Ok(Value::Boolean(
                args.value("nmae").is_some()
            )))
            .unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn test_names_are_those_of_the_overload_that_matched() {
        let definition = FunctionDefinition::new("f", Type::any(), vec![Argument::number("n")])
            .add_overload(Type::any(), vec![Argument::string("s")]);
        let result = call(definition, &[string("a")], &[], |args| {
            Ok(args.get::<&Value>("n")?.clone())
        });
        assert_eq!(
            result.unwrap_err().to_string(),
            "Handler Error: Function f has no argument n"
        );
    }

    #[test]
    fn test_reading_a_variadic_argument_by_name_is_a_handler_error() {
        let first = || {
            FunctionDefinition::new(
                "first",
                Type::any(),
                vec![Argument::variadic("values", Type::any())],
            )
        };
        for args in [vec![number(1), number(2)], vec![]] {
            let result = call(first(), &args, &[], |args| {
                Ok(args.get::<&Value>("values")?.clone())
            });
            assert!(matches!(
                &result,
                Err(Error::HandlerError(HandlerError::VariadicArgument { name, argument }))
                    if name == "first" && argument == "values"
            ));
            assert_eq!(
                result.unwrap_err().to_string(),
                "Handler Error: Argument values of first is variadic; read it with Arguments::rest"
            );
        }
        assert_eq!(
            call(first(), &[number(1)], &[], |args| {
                Ok(Value::Boolean(args.value("values").is_some()))
            })
            .unwrap(),
            Value::Boolean(false)
        );
    }

    fn integer(n: i64) -> Value {
        Value::Integer(BigInt::from(n))
    }

    fn decimal(s: &str) -> Value {
        Value::Number(BigDecimal::from_str(s).unwrap())
    }

    /// Returns the types of the overload that matched, then the values it got, e.g.
    /// `(Integer) 1`
    fn overload_and_values(definition: FunctionDefinition) -> NativeFunction {
        NativeFunction::new(definition, |args| {
            let signature = args.signature();
            let types: Vec<String> = signature
                .arguments
                .iter()
                .map(|argument| argument.r#type.to_string())
                .chain(
                    signature
                        .keywords
                        .iter()
                        .map(|keyword| format!("{}: {}", keyword.name, keyword.r#type)),
                )
                .collect();
            let names = signature
                .arguments
                .iter()
                .map(|argument| argument.name.as_str())
                .chain(
                    signature
                        .keywords
                        .iter()
                        .map(|keyword| keyword.name.as_str()),
                );
            let values = names
                .map(|name| args.get::<&Value>(name).map(ToString::to_string))
                .collect::<Result<Vec<String>, Error>>()?;
            Ok(Value::String(format!(
                "({}) {}",
                types.join(", "),
                values.join(", ")
            )))
        })
    }

    /// The string a call to an `overload_and_values` function returns
    fn called(
        function: &NativeFunction,
        args: &[Value],
        call_keywords: &[(&str, Value)],
    ) -> String {
        match function.call_with_keywords(args, keywords(call_keywords)) {
            Ok(Value::String(s)) => s,
            other => panic!("{} returned {other:?}", function.name()),
        }
    }

    #[test]
    fn test_argument_integer() {
        let argument = Argument::integer("n");
        assert_eq!(argument.name, "n");
        assert_eq!(argument.r#type, Type::integer());
        assert!(!argument.variadic);
    }

    #[test]
    fn test_an_integer_parameter_refuses_a_number() {
        let function = overload_and_values(FunctionDefinition::new(
            "f",
            Type::string(),
            vec![Argument::integer("i")],
        ));
        assert_eq!(called(&function, &[integer(1)], &[]), "(Integer) 1");
        assert_eq!(
            error(&function, &[decimal("1.0")], &[]),
            "Runtime Error: Argument i of f expects Integer, found Number"
        );
    }

    #[test]
    fn test_a_number_parameter_converts_an_integer() {
        let function = overload_and_values(FunctionDefinition::new(
            "f",
            Type::string(),
            vec![Argument::number("x")],
        ));
        assert_eq!(called(&function, &[integer(1)], &[]), "(Number) 1.0");
        assert_eq!(called(&function, &[decimal("1.5")], &[]), "(Number) 1.5");
    }

    #[test]
    fn test_a_union_converts_an_integer_only_if_no_member_accepts_it() {
        let either = overload_and_values(FunctionDefinition::new(
            "either",
            Type::string(),
            vec![Argument::new(
                "x",
                Type::union(Type::integer(), Type::number()),
            )],
        ));
        assert_eq!(called(&either, &[integer(1)], &[]), "(Integer | Number) 1");
        let num_or_str = overload_and_values(FunctionDefinition::new(
            "num_or_str",
            Type::string(),
            vec![Argument::new(
                "x",
                Type::union(Type::number(), Type::string()),
            )],
        ));
        assert_eq!(
            called(&num_or_str, &[integer(1)], &[]),
            "(Number | String) 1.0"
        );
        let anything = overload_and_values(FunctionDefinition::new(
            "anything",
            Type::string(),
            vec![Argument::new("x", Type::any())],
        ));
        assert_eq!(called(&anything, &[integer(1)], &[]), "(Any) 1");
    }

    #[test]
    fn test_variadic_values_are_converted() {
        let function = NativeFunction::new(
            FunctionDefinition::new(
                "values",
                Type::array(),
                vec![Argument::variadic("values", Type::number())],
            ),
            |args| Ok(Value::Array(args.rest().to_vec())),
        );
        let result = function.call(&[integer(1), decimal("2.5")]).unwrap();
        assert_eq!(result.to_string(), "[1.0, 2.5]");
    }

    #[test]
    fn test_keyword_values_and_defaults_are_converted() {
        let function = NativeFunction::new(
            FunctionDefinition::new("step", Type::any(), vec![])
                .keyword("by", Type::number(), Value::from(1u64))
                .rest_keywords(Type::number()),
            |args| Ok(Value::Map(args.keywords().clone())),
        );
        assert_eq!(function.call(&[]).unwrap().to_string(), "{by: 1.0}");
        let result = function
            .call_with_keywords(&[], keywords(&[("by", integer(2)), ("k", integer(3))]))
            .unwrap();
        assert_eq!(result.to_string(), "{by: 2.0, k: 3.0}");
    }

    #[test]
    fn test_an_integer_default_passes_an_integer_keywords_type_check() {
        // Doesn't panic: `Value::from(usize)` is an `Integer`
        let definition = FunctionDefinition::new("head", Type::array(), vec![]).keyword(
            "n",
            Type::integer(),
            Value::from(10usize),
        );
        assert_eq!(definition.r#type().to_string(), "fn(n: Integer) -> Array");
    }

    #[test]
    #[should_panic(expected = "default of the wrong type")]
    fn test_a_number_default_fails_an_integer_keywords_type_check() {
        let _ = FunctionDefinition::new("head", Type::array(), vec![]).keyword(
            "n",
            Type::integer(),
            decimal("10.0"),
        );
    }

    #[test]
    fn test_a_number_result_is_converted() {
        let function = NativeFunction::new(
            FunctionDefinition::new("rows", Type::number(), vec![]),
            |_args| Ok(Value::from(5usize)),
        );
        let result = function.call(&[]).unwrap();
        assert!(matches!(result, Value::Number(_)), "got {result:?}");
        assert_eq!(result.to_string(), "5.0");
    }

    #[test]
    fn test_a_handler_reads_numbers_as_big_ints_and_big_decimals() {
        let function = NativeFunction::new(
            FunctionDefinition::new(
                "read",
                Type::string(),
                vec![Argument::integer("i"), Argument::number("x")],
            ),
            |args| {
                let i: BigInt = args.get("i")?;
                let x: BigDecimal = args.get("x")?;
                let whole: BigInt = args.get("x")?;
                Ok(Value::String(format!("{i} {x} {whole}")))
            },
        );
        assert_eq!(
            function.call(&[integer(7), integer(2)]).unwrap(),
            string("7 2 2")
        );
        let error = function
            .call(&[integer(7), decimal("2.5")])
            .unwrap_err()
            .to_string();
        assert_eq!(error, "Runtime Error: Cannot convert 2.5 to BigInt");
    }

    #[test]
    fn test_from_value_for_big_numbers() {
        let seven = integer(7);
        assert_eq!(BigInt::from_value(&seven).unwrap(), BigInt::from(7));
        assert_eq!(BigDecimal::from_value(&seven).unwrap(), BigDecimal::from(7));
        let half = decimal("0.5");
        assert_eq!(
            BigDecimal::from_value(&half).unwrap(),
            BigDecimal::from_str("0.5").unwrap()
        );
        assert_eq!(
            BigInt::from_value(&decimal("2.0")).unwrap(),
            BigInt::from(2)
        );
        assert!(matches!(
            BigInt::from_value(&half),
            Err(Error::RuntimeError(RuntimeError::CannotConvert { .. }))
        ));
        assert!(matches!(
            BigInt::from_value(&string("7")),
            Err(Error::RuntimeError(RuntimeError::UnexpectedType { .. }))
        ));
        assert_eq!(Option::<BigInt>::from_value(&Value::None).unwrap(), None);
    }

    /// f(n: Number), then f(i: Integer)
    fn f() -> NativeFunction {
        overload_and_values(
            FunctionDefinition::new("f", Type::string(), vec![Argument::number("n")])
                .add_overload(Type::string(), vec![Argument::integer("i")]),
        )
    }

    #[test]
    fn test_the_narrowest_overload_is_called() {
        assert_eq!(called(&f(), &[integer(1)], &[]), "(Integer) 1");
        assert_eq!(called(&f(), &[decimal("1.0")], &[]), "(Number) 1.0");
    }

    #[test]
    fn test_a_narrower_overload_beats_any_declared_first() {
        let show = overload_and_values(
            FunctionDefinition::new(
                "show",
                Type::string(),
                vec![Argument::new("x", Type::any())],
            )
            .add_overload(Type::string(), vec![Argument::number("x")]),
        );
        assert_eq!(called(&show, &[decimal("1.0")], &[]), "(Number) 1.0");
        assert_eq!(called(&show, &[integer(1)], &[]), "(Number) 1.0");
        assert_eq!(called(&show, &[symbol("a")], &[]), "(Any) :a");
    }

    #[test]
    fn test_overloads_that_are_each_narrower_for_one_argument_go_to_the_first_declared() {
        let g = |reversed: bool| {
            let (first, second) = if reversed {
                (
                    vec![Argument::number("a"), Argument::integer("b")],
                    vec![Argument::integer("a"), Argument::number("b")],
                )
            } else {
                (
                    vec![Argument::integer("a"), Argument::number("b")],
                    vec![Argument::number("a"), Argument::integer("b")],
                )
            };
            overload_and_values(
                FunctionDefinition::new("g", Type::string(), first)
                    .add_overload(Type::string(), second),
            )
        };
        assert_eq!(
            called(&g(false), &[integer(1), integer(2)], &[]),
            "(Integer, Number) 1, 2.0"
        );
        assert_eq!(
            called(&g(true), &[integer(1), integer(2)], &[]),
            "(Number, Integer) 1.0, 2"
        );
        assert_eq!(
            called(&g(false), &[decimal("1.0"), integer(2)], &[]),
            "(Number, Integer) 1.0, 2"
        );
    }

    #[test]
    fn test_keywords_the_call_passes_count_towards_narrowness() {
        let kw = overload_and_values(
            FunctionDefinition::new("kw", Type::string(), vec![])
                .keyword("n", Type::number(), Value::from(0u64))
                .add_overload(Type::string(), vec![])
                .keyword("n", Type::integer(), Value::from(0u64)),
        );
        assert_eq!(called(&kw, &[], &[("n", integer(1))]), "(n: Integer) 1");
        assert_eq!(
            called(&kw, &[], &[("n", decimal("1.0"))]),
            "(n: Number) 1.0"
        );
        // A keyword the call leaves out doesn't count, so this is a tie
        assert_eq!(called(&kw, &[], &[]), "(n: Number) 0.0");
    }

    #[test]
    fn test_arity_still_comes_before_narrowness() {
        let definition = FunctionDefinition::new(
            "f",
            Type::string(),
            vec![Argument::variadic("rest", Type::integer())],
        )
        .add_overload(
            Type::string(),
            vec![
                Argument::number("a"),
                Argument::variadic("rest", Type::any()),
            ],
        );
        // The arity-1 overload is tried first, although the arity-0 one is narrower
        let signature = definition.resolve(&[integer(1)], &Map::new()).unwrap();
        assert_eq!(signature.arguments[0].r#type, Type::number());
    }

    #[test]
    fn test_resolve_picks_the_narrowest_overload() {
        let definition = FunctionDefinition::new("f", Type::string(), vec![Argument::number("n")])
            .add_overload(Type::string(), vec![Argument::integer("i")]);
        let signature = definition.resolve(&[integer(1)], &Map::new()).unwrap();
        assert_eq!(signature.arguments[0].r#type, Type::integer());
        let signature = definition.resolve(&[decimal("1.5")], &Map::new()).unwrap();
        assert_eq!(signature.arguments[0].r#type, Type::number());
    }
}
