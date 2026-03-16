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
    While,
    Do,
    Fn,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Return => "return",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Not => "not",
            Keyword::For => "for",
            Keyword::In => "in",
            Keyword::While => "while",
            Keyword::Do => "do",
            Keyword::Fn => "fn",
        };
        write!(f, "{s}")
    }
}
