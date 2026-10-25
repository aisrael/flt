use std::str::FromStr;

use strum::Display;
use strum::EnumIter;
use strum::EnumString;

/// Reserved keywords in the language. Each is spelled as its variant name in lowercase, e.g.
/// `Keyword::If` is `if`.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumIter, EnumString)]
#[strum(serialize_all = "lowercase")]
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
    /// The keyword spelled `name`, if `name` is a reserved keyword. The match is exact, so
    /// `If` and `iffy` aren't keywords.
    pub fn from_name(name: &str) -> Option<Keyword> {
        Keyword::from_str(name).ok()
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_keyword_spellings() {
        let spellings: Vec<String> = Keyword::iter().map(|k| k.to_string()).collect();
        assert_eq!(
            spellings,
            ["if", "else", "return", "and", "or", "not", "for", "in", "let", "while", "do", "fn"]
        );
    }

    #[test]
    fn test_from_name_round_trips_with_display() {
        for keyword in Keyword::iter() {
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
