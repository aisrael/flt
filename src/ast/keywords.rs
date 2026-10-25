use std::fmt::Display;

/// Reserved keywords in the language.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keyword {
    If,
    Else,
    Return,
    And,
    Or,
    Not,
    For,
    In,
    Let,
    While,
    Do,
    Fn,
}

impl Keyword {
    /// Every reserved keyword.
    pub const ALL: [Keyword; 12] = [
        Keyword::If,
        Keyword::Else,
        Keyword::Return,
        Keyword::And,
        Keyword::Or,
        Keyword::Not,
        Keyword::For,
        Keyword::In,
        Keyword::Let,
        Keyword::While,
        Keyword::Do,
        Keyword::Fn,
    ];

    /// The keyword's spelling in source, e.g. `if`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Return => "return",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Not => "not",
            Keyword::For => "for",
            Keyword::In => "in",
            Keyword::Let => "let",
            Keyword::While => "while",
            Keyword::Do => "do",
            Keyword::Fn => "fn",
        }
    }

    /// The keyword spelled `name`, if `name` is a reserved keyword.
    pub fn from_name(name: &str) -> Option<Keyword> {
        Self::ALL
            .into_iter()
            .find(|keyword| keyword.as_str() == name)
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_round_trips_with_display() {
        for keyword in Keyword::ALL {
            assert_eq!(Keyword::from_name(&keyword.to_string()), Some(keyword));
        }
    }

    #[test]
    fn test_from_name_rejects_other_words() {
        assert_eq!(Keyword::from_name("iffy"), None);
        assert_eq!(Keyword::from_name("If"), None);
        assert_eq!(Keyword::from_name(""), None);
    }
}
