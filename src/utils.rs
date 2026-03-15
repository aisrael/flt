/// Escapes a string for display as a string literal.
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
