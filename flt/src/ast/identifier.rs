use crate::Error;

/// An identifier in the language (e.g. variable name, function name).
#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn as_str(&self) -> &str {
        &self.0
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
