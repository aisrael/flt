use std::{fmt::Display, ops::Deref};

use crate::Error;

/// An identifier in the language (e.g. variable name, function name).
#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(pub String);

impl Identifier {}

impl PartialEq<&str> for Identifier {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if !s.is_empty()
            && s.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Ok(Identifier(s.to_string()));
        }

        Err(Error::SyntaxError("Invalid identifier".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_eq_str() {
        assert!(Identifier("foo".to_string()) == "foo");
    }
}
