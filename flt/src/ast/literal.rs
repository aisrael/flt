use std::convert::TryFrom;

use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;

use super::number::Numeric;
use crate::errors::Error;

/// A literal value: number, string, boolean, or symbol.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(Numeric),
    String(String),
    Boolean(bool),
    Symbol(String),
}

impl Literal {
    /// Construct a number literal from a BigDecimal
    pub fn number(n: impl Into<BigDecimal>) -> Self {
        Literal::Number(Numeric::new(n.into()))
    }

    /// Construct a string literal from a string (e.g. `"hello"`).
    pub fn string(s: impl Into<String>) -> Self {
        Literal::String(s.into())
    }

    /// Construct a boolean literal from a string (e.g. `"true"`).
    pub fn boolean(b: bool) -> Self {
        Literal::Boolean(b)
    }

    /// Construct a symbol literal (e.g. `:foo` or `:"hello"`).
    pub fn symbol(s: impl Into<String>) -> Self {
        Literal::Symbol(s.into())
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Boolean(value)
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Literal::Number(Numeric::new(value))
    }
}

impl TryFrom<f64> for Literal {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        BigDecimal::from_f64(value)
            .map(|bd| Literal::Number(Numeric::new(bd)))
            .ok_or(Error::F64ConversionError)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::String(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_literal_from_f64() {
        assert!(Literal::try_from(
            #[allow(clippy::approx_constant)]
            3.14
        )
        .is_ok());
        assert!(matches!(
            Literal::try_from(f64::INFINITY),
            Err(Error::F64ConversionError)
        ));
        assert!(matches!(
            Literal::try_from(f64::NAN),
            Err(Error::F64ConversionError)
        ));
        assert!(matches!(
            Literal::try_from(f64::NEG_INFINITY),
            Err(Error::F64ConversionError)
        ));
    }

    #[test]
    fn test_literal_from_i64() {
        assert_eq!(
            Literal::from(42),
            Literal::Number(Numeric::new(BigDecimal::from_str("42").unwrap()))
        );
    }
}
