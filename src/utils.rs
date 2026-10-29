use std::fmt;

use bigdecimal::BigDecimal;

/// Escapes a string for display as a string literal.
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Whether `name` can be written bare, as a symbol (`:name`) or a map key (`name: 1`): a letter,
/// then letters, digits or `_`. Other names are written quoted.
pub fn is_bare_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(char::is_alphabetic) && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Writes `name` bare if it's a valid bare name, and quoted otherwise.
pub(crate) fn write_name(f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
    if is_bare_name(name) {
        write!(f, "{name}")
    } else {
        write!(f, "\"{}\"", escape_string(name))
    }
}

/// Writes a `Number` as flt prints it: with a trailing `.0` if it has no digits after the
/// decimal point, so it never looks like an `Integer`, e.g. `2.0`, `2.5` or `2.50`.
pub(crate) fn write_number(f: &mut fmt::Formatter<'_>, n: &BigDecimal) -> fmt::Result {
    let text = n.to_string();
    if text.contains(['.', 'e', 'E']) {
        write!(f, "{text}")
    } else {
        write!(f, "{text}.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bare_name() {
        assert!(is_bare_name("b"));
        assert!(is_bare_name("abc123"));
        assert!(is_bare_name("foo_bar"));
        assert!(!is_bare_name(""));
        assert!(!is_bare_name("_x"));
        assert!(!is_bare_name("1x"));
        assert!(!is_bare_name("c d"));
        assert!(!is_bare_name("foo-bar"));
    }
}
