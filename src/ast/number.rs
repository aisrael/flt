use std::fmt::Display;
use std::str::FromStr;

use bigdecimal::BigDecimal;

/// A numeric literal: optional `+` or `-`, digits, then optionally `.` followed by any number of decimal digits.
#[derive(Clone, Debug, PartialEq)]
pub struct Numeric(pub BigDecimal);

impl Display for Numeric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Numeric> for BigDecimal {
    fn from(value: Numeric) -> Self {
        value.0
    }
}

impl AsRef<BigDecimal> for Numeric {
    fn as_ref(&self) -> &BigDecimal {
        &self.0
    }
}

impl FromStr for Numeric {
    type Err = <BigDecimal as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigDecimal::from_str(s).map(Numeric)
    }
}
