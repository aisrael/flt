# Changelog

## v0.1

- **REPL**: Interactive read-eval-print loop and `flt version` CLI.
- **Literals**: Numbers (arbitrary precision), strings, booleans, and symbols (`:foo`, `:"quoted"`).
- **Expressions**: Identifiers, unary (`!`, `+`, `-`), binary (arithmetic, logical, bitwise, pipe `|>`, string concat `<>`), function calls with optional key-value arguments, parenthesized expressions, map literals `{ key: value }`, and array literals `[ expr, ... ]`.
- **String interpolation**: `"Hello, {expr}!"` with arbitrary expressions inside `{}`.
- **Comments**: Line comments from `#` to end of line.
- **Evaluation**: Literals, unary and binary operators (arithmetic, logical, string concat), and parenthesized expressions. Function calls and map/array literals are not yet evaluated (runtime error if reached).
