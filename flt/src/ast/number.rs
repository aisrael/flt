use std::fmt::Display;

use bigdecimal::BigDecimal;

/// A numeric literal: optional `+` or `-`, digits, then optionally `.` followed by any number of decimal digits.
#[derive(Clone, Debug, PartialEq)]
pub struct Numeric {
    value: BigDecimal,
}

impl Numeric {
    pub fn new<T: Into<BigDecimal>>(value: T) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl Display for Numeric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<Numeric> for BigDecimal {
    fn from(value: Numeric) -> Self {
        value.value
    }
}

impl AsRef<BigDecimal> for Numeric {
    fn as_ref(&self) -> &BigDecimal {
        &self.value
    }
}
