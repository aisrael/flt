# Changelog

## v0.1.1

- **Fix**: Map literals (`{ key: value }`) now evaluate to `Value::Map` instead of erroring.
- **Fix**: Array literals (`[ expr, ... ]`) now evaluate to `Value::Array` instead of erroring.
- **Dot-access syntax for maps**: `u.foo` reads a field off a map value, supported at every expression precedence level (including unary and `if` conditions), with a runtime error for missing fields or non-map operands.

## v0.1.0

- **REPL**: Interactive read-eval-print loop and `flt version` CLI.
- **Literals**: Numbers (arbitrary precision), strings, booleans, and symbols (`:foo`, `:"quoted"`).
- **Expressions**: Identifiers, unary (`!`, `+`, `-`), binary (arithmetic, logical, bitwise, pipe `|>`, string concat `<>`), function calls with optional key-value arguments, parenthesized expressions, map literals `{ key: value }`, and array literals `[ expr, ... ]`.
- **String interpolation**: `"Hello, {expr}!"` with arbitrary expressions inside `{}`.
- **Conditionals**: `if <cond> { <then> } else { <else> }` (block branches) and `if <cond> <then_expr> else <else_expr>` (expression branches). The `else` branch is optional.
- **Assignment / Keywords**: `let x = expr;` and `x = expr;` (semicolon/newline handling is parser-dependent) plus core keywords like `let`, `if`, and `else`.
- **Comments**: Line comments from `#` to end of line.
- **Evaluation**: Literals, unary and binary operators (arithmetic, logical, string concat), parenthesized expressions, and `if` expressions. Function calls and map/array literals are not yet evaluated (runtime error if reached).
