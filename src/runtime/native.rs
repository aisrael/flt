//! Native types and values: a host's own Rust values, passed through flt

use std::any::Any;
use std::any::TypeId;
use std::fmt;
use std::sync::Arc;

/// A host's Rust type that flt can hold as an opaque value. Its values print with `Display`
/// and compare with `PartialEq`; only the host's Rust code can create or read them.
pub trait NativeType: Any + fmt::Display + PartialEq + Send + Sync {
    /// The type's name in flt, e.g. `Date`
    const NAME: &'static str;
}

/// The flt type of a native type: its name, and the Rust type that identifies it.
#[derive(Clone, Copy, Debug)]
pub struct NativeTypeInfo {
    name: &'static str,
    type_id: TypeId,
}

impl NativeTypeInfo {
    /// The flt type of the native type `T`
    pub fn of<T: NativeType>() -> Self {
        Self {
            name: T::NAME,
            type_id: TypeId::of::<T>(),
        }
    }

    /// The type's name in flt
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Two native types are the same if their Rust types are; `NAME` is fixed per Rust type.
impl PartialEq for NativeTypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

/// The object-safe view of a native value. `NativeType` isn't dyn-compatible (it has a
/// constant and requires `PartialEq`), so native values hold a `dyn NativeObject`.
pub(crate) trait NativeObject: fmt::Display + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn type_info(&self) -> NativeTypeInfo;
    /// Equality through `T: PartialEq`; false for a different Rust type
    fn eq_dyn(&self, other: &dyn NativeObject) -> bool;
}

impl<T: NativeType> NativeObject for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_info(&self) -> NativeTypeInfo {
        NativeTypeInfo::of::<T>()
    }

    fn eq_dyn(&self, other: &dyn NativeObject) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }
}

/// A value of a native type, created by the host
#[derive(Clone)]
pub struct NativeValue {
    object: Arc<dyn NativeObject>,
}

impl NativeValue {
    /// Wraps a host value
    pub fn new<T: NativeType>(value: T) -> Self {
        Self {
            object: Arc::new(value),
        }
    }

    /// The value's native type
    pub fn type_info(&self) -> NativeTypeInfo {
        self.object.type_info()
    }

    /// Borrows the host value, if it's a `T`
    pub fn get<T: NativeType>(&self) -> Option<&T> {
        self.object.as_any().downcast_ref::<T>()
    }
}

/// Writes the value as the host's `Display` does
impl fmt::Display for NativeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.object.fmt(f)
    }
}

/// Writes `NativeValue(<type name>)`, since a native type needn't implement `Debug`
impl fmt::Debug for NativeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NativeValue({})", self.type_info().name())
    }
}

/// Compares with the host's `PartialEq`; values of different native types are never equal
impl PartialEq for NativeValue {
    fn eq(&self, other: &Self) -> bool {
        self.object.eq_dyn(&*other.object)
    }
}

/// Native types for tests in this crate.
#[cfg(test)]
pub(crate) mod fixtures {
    use std::fmt;

    use super::NativeType;

    /// A length in meters, displayed as e.g. `5m`
    #[derive(Debug, PartialEq)]
    pub(crate) struct Meters(pub i64);

    impl fmt::Display for Meters {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}m", self.0)
        }
    }

    impl NativeType for Meters {
        const NAME: &'static str = "Meters";
    }

    /// A duration in seconds, displayed as e.g. `5s`
    #[derive(Debug, PartialEq)]
    pub(crate) struct Seconds(pub i64);

    impl fmt::Display for Seconds {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}s", self.0)
        }
    }

    impl NativeType for Seconds {
        const NAME: &'static str = "Seconds";
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;

    use super::fixtures::Meters;
    use super::fixtures::Seconds;
    use super::*;
    use crate::errors::RuntimeError;
    use crate::runtime::types::Type;
    use crate::runtime::Value;
    use crate::Error;

    #[test]
    fn test_native_value_round_trips() {
        let value = Value::native(Meters(5));
        assert_eq!(value.get_native::<Meters>(), Some(&Meters(5)));
        assert_eq!(value.downcast_native::<Meters>().unwrap(), &Meters(5));
    }

    #[test]
    fn test_get_native_is_none_for_other_values() {
        assert_eq!(
            Value::Number(BigDecimal::from(5)).get_native::<Meters>(),
            None
        );
        assert_eq!(Value::None.get_native::<Meters>(), None);
        assert_eq!(Value::native(Seconds(5)).get_native::<Meters>(), None);
    }

    #[test]
    fn test_downcast_native_of_another_value_is_unexpected_type() {
        let error = Value::Number(BigDecimal::from(5))
            .downcast_native::<Meters>()
            .unwrap_err();
        assert!(matches!(
            &error,
            Error::RuntimeError(RuntimeError::UnexpectedType { expected, found })
                if expected == "Meters" && found == "Number"
        ));
        assert_eq!(
            error.to_string(),
            "Runtime Error: Expected Meters, found Number"
        );
    }

    #[test]
    fn test_downcast_native_of_another_native_type_names_it() {
        let error = Value::native(Seconds(5))
            .downcast_native::<Meters>()
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Runtime Error: Expected Meters, found Seconds"
        );
    }

    #[test]
    fn test_native_type_info() {
        let info = NativeTypeInfo::of::<Meters>();
        assert_eq!(info.name(), "Meters");
        assert_eq!(info, NativeTypeInfo::of::<Meters>());
        assert_ne!(info, NativeTypeInfo::of::<Seconds>());
    }

    #[test]
    fn test_native_type_is_a_type() {
        let native_type = Type::native::<Meters>();
        assert_eq!(native_type, Type::Native(NativeTypeInfo::of::<Meters>()));
        assert_eq!(native_type.to_string(), "Meters");
        assert_ne!(native_type, Type::native::<Seconds>());
    }

    #[test]
    fn test_type_of_native_value_is_its_native_type() {
        // Registration isn't needed for a value to have its type
        assert_eq!(
            Value::native(Meters(5)).type_of().unwrap(),
            Type::native::<Meters>()
        );
    }

    #[test]
    fn test_native_value_displays_as_the_host_does() {
        assert_eq!(Value::native(Meters(5)).to_string(), "5m");
    }

    #[test]
    fn test_native_value_debug_names_its_type() {
        assert_eq!(
            format!("{:?}", Value::native(Meters(5))),
            "Native(NativeValue(Meters))"
        );
    }

    #[test]
    fn test_native_values_compare_with_the_hosts_equality() {
        let value = Value::native(Meters(5));
        assert_eq!(value, value.clone());
        assert_eq!(value, Value::native(Meters(5)));
        assert_ne!(value, Value::native(Meters(6)));
    }

    #[test]
    fn test_native_values_of_different_types_are_never_equal() {
        assert_ne!(Value::native(Meters(5)), Value::native(Seconds(5)));
        assert_ne!(Value::native(Meters(5)), Value::Number(BigDecimal::from(5)));
    }
}
