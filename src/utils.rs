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
