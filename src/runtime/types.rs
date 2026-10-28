//! Types for the flt runtime

use std::fmt;

use crate::runtime::native::NativeType;
use crate::runtime::native::NativeTypeInfo;
use crate::runtime::Value;

mod function;
mod union_types;

pub use function::FunctionType;

/// A type is either a builtin type or a custom type
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
    /// The `Option` wrapper around an inner type (the empty case is `None`)
    Option(Box<Type>),
    /// A union of types: a value of any one of the member types. Build it with
    /// [`Type::union`] so that it stays flattened and free of duplicates.
    Union(Vec<Type>),
    /// The type of a function, e.g. `fn(Any) -> Any`
    Function(FunctionType),
    /// A native type, defined by the host
    Native(NativeTypeInfo),
}

impl Type {
    /// The flt type of the native type `T`, e.g. for function signatures
    pub fn native<T: NativeType>() -> Self {
        Type::Native(NativeTypeInfo::of::<T>())
    }

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

    /// Whether `name` is a built-in type name. This includes `Any`, which isn't a value but
    /// names a type in signatures.
    pub fn is_builtin_name(name: &str) -> bool {
        name == "Any" || Self::builtin_named(name).is_some()
    }

    /// Whether `value` is a value of this type, e.g. `Number | String` accepts `1` and `"a"`.
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Type::Builtin(BuiltinType::Any) => true,
            Type::Builtin(_) => value.type_of().is_ok_and(|found| found == *self),
            Type::Union(members) => members.iter().any(|member| member.accepts(value)),
            Type::Option(inner) => matches!(value, Value::None) || inner.accepts(value),
            Type::Native(info) => {
                matches!(value, Value::Native(native) if native.type_info() == *info)
            }
            // A function whose type, or one of whose overloads' types, is exactly this one
            Type::Function(function_type) => match value {
                Value::Function(function) => function
                    .definition
                    .overloads
                    .iter()
                    .any(|overload| FunctionType::from(overload) == *function_type),
                _ => false,
            },
            // No value has a custom type yet
            Type::Custom(_) => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Builtin(b) => write!(f, "{b}"),
            Type::Custom(c) => write!(f, "{}", c.name),
            Type::Option(inner) => write!(f, "Option<{inner}>"),
            Type::Union(members) => union_types::fmt_union(members, f),
            Type::Function(function_type) => write!(f, "{function_type}"),
            Type::Native(info) => write!(f, "{}", info.name()),
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
    use std::sync::Arc;

    use super::*;
    use crate::runtime::functions::Argument;
    use crate::runtime::functions::FunctionDefinition;
    use crate::runtime::functions::NativeFunction;
    use crate::runtime::native::fixtures::Meters;
    use crate::runtime::native::fixtures::Seconds;
    use crate::runtime::Map;

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

    fn identity() -> Value {
        Value::Function(Arc::new(NativeFunction::new(
            FunctionDefinition::new(
                "identity",
                Type::any(),
                vec![Argument::new("v", Type::any())],
            ),
            |args| Ok(args.get::<&Value>("v")?.clone()),
        )))
    }

    /// One value of each kind, by name
    fn values() -> Vec<(&'static str, Value)> {
        vec![
            ("None", Value::None),
            ("1", Value::from(1u64)),
            ("\"a\"", Value::String("a".into())),
            ("true", Value::Boolean(true)),
            (":a", Value::Symbol("a".into())),
            ("[]", Value::Array(vec![])),
            ("{}", Value::Map(Map::new())),
            ("Number", Value::Type(Type::number())),
            ("identity", identity()),
            ("Meters(1)", Value::native(Meters(1))),
            ("Seconds(1)", Value::native(Seconds(1))),
        ]
    }

    /// Asserts that `r#type` accepts exactly the named values of `values()`
    fn assert_accepts(r#type: Type, accepted: &[&str]) {
        for (name, value) in values() {
            assert_eq!(
                r#type.accepts(&value),
                accepted.contains(&name),
                "{type} accepts {name}: expected {}",
                accepted.contains(&name)
            );
        }
    }

    #[test]
    fn test_any_accepts_every_value() {
        let every: Vec<&str> = values().into_iter().map(|(name, _)| name).collect();
        assert_accepts(Type::any(), &every);
    }

    #[test]
    fn test_builtin_types_accept_their_values() {
        assert_accepts(Type::none(), &["None"]);
        assert_accepts(Type::number(), &["1"]);
        assert_accepts(Type::string(), &["\"a\""]);
        assert_accepts(Type::boolean(), &["true"]);
        assert_accepts(Type::symbol(), &[":a"]);
        assert_accepts(Type::array(), &["[]"]);
        assert_accepts(Type::map(), &["{}"]);
        assert_accepts(Type::type_type(), &["Number"]);
    }

    #[test]
    fn test_union_accepts_any_members_values() {
        assert_accepts(Type::union(Type::number(), Type::string()), &["1", "\"a\""]);
        assert_accepts(
            Type::union(Type::union(Type::number(), Type::string()), Type::symbol()),
            &["1", "\"a\"", ":a"],
        );
    }

    #[test]
    fn test_option_accepts_none_and_its_inner_types_values() {
        assert_accepts(Type::option(Type::string()), &["None", "\"a\""]);
    }

    #[test]
    fn test_native_type_accepts_only_its_own_values() {
        assert_accepts(Type::native::<Meters>(), &["Meters(1)"]);
        assert_accepts(Type::native::<Seconds>(), &["Seconds(1)"]);
    }

    /// Another `Meters`, with the same name but a different Rust type
    #[derive(PartialEq)]
    struct OtherMeters;

    impl fmt::Display for OtherMeters {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "other")
        }
    }

    impl NativeType for OtherMeters {
        const NAME: &'static str = "Meters";
    }

    #[test]
    fn test_native_types_with_the_same_name_are_different_types() {
        assert!(!Type::native::<OtherMeters>().accepts(&Value::native(Meters(1))));
        assert!(!Type::native::<Meters>().accepts(&Value::native(OtherMeters)));
    }

    #[test]
    fn test_function_type_accepts_a_function_of_exactly_that_type() {
        assert_accepts(
            Type::Function(FunctionType::new(vec![Type::any()], Type::any())),
            &["identity"],
        );
        assert_accepts(
            Type::Function(FunctionType::new(vec![Type::number()], Type::number())),
            &[],
        );
    }

    #[test]
    fn test_function_type_accepts_a_function_with_an_overload_of_that_type() {
        let overloaded = Value::Function(Arc::new(NativeFunction::new(
            FunctionDefinition::new("f", Type::number(), vec![Argument::number("n")])
                .add_overload(Type::string(), vec![Argument::string("s")]),
            |args| Ok(args.positional()[0].clone()),
        )));
        let number_to_number = FunctionType::new(vec![Type::number()], Type::number());
        let string_to_string = FunctionType::new(vec![Type::string()], Type::string());
        let any_to_any = FunctionType::new(vec![Type::any()], Type::any());
        assert!(Type::Function(number_to_number).accepts(&overloaded));
        assert!(Type::Function(string_to_string).accepts(&overloaded));
        assert!(!Type::Function(any_to_any).accepts(&overloaded));
    }

    #[test]
    fn test_builtin_named_excludes_any_and_unknown_names() {
        assert_eq!(Type::builtin_named("Any"), None);
        assert_eq!(Type::builtin_named("Foo"), None);
        assert_eq!(Type::builtin_named("number"), None);
    }
}
