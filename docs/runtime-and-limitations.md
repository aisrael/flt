# Runtime Behavior and Limitations

This page describes current evaluator behavior in `flt-cli`.

## What Evaluates Today

The evaluator currently supports:

- Number literals and arithmetic: `+`, `-`, `*`, `/`
- Boolean literals and logic: `&&`, `||`, `^^`, unary `!`
- String literals and concatenation with `<>`
- Parenthesized expressions
- Unary numeric `+` and `-`
- `let` bindings (`let x = expr`) and plain assignment (`x = expr`), stored in the global scope; evaluation returns the bound value (what the REPL prints)

`<>` concatenation coerces values to strings (numbers, booleans, and symbols can be concatenated).

## Current Runtime Errors

Common errors include:

- `Invalid Operand Type` for unsupported type/operator combinations
- `Division By Zero`
- `Unbound identifier: <name>`
- `Function calls not yet supported`

## Parsed but Not Yet Evaluated

These constructs parse successfully but are not fully supported by the current evaluator:

- Function calls (including calls with keyword arguments, e.g. `foo(1, bar: true)`)
- Pipe expressions (`|>`)
- Bitwise operators (`&`, `|`, `^`)
- Standalone keyword expressions (e.g. `if`, `return`, `fn`) — they parse as expressions but have no evaluation behavior yet

## Practical Guidance

- Treat parser support and evaluator support as separate capabilities.
- Use parser-focused tests when documenting syntax.
- Use REPL behavior when documenting what executes today.
