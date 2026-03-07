# Runtime Behavior and Limitations

This page describes current evaluator behavior in `flt-cli`.

## What Evaluates Today

The evaluator currently supports:

- Number literals and arithmetic: `+`, `-`, `*`, `/`
- Boolean literals and logic: `&&`, `||`, `^^`, unary `!`
- String literals and concatenation with `<>`
- Parenthesized expressions
- Unary numeric `+` and `-`

`<>` concatenation coerces values to strings (numbers, booleans, and symbols can be concatenated).

## Current Runtime Errors

Common errors include:

- `Invalid Operand Type` for unsupported type/operator combinations
- `Division By Zero`
- `Unbound identifier: <name>`
- `Function calls not yet supported`

## Parsed but Not Yet Evaluated

These constructs parse successfully but are not fully supported by the current evaluator:

- Function calls
- Pipe expressions (`|>`)
- Bitwise operators (`&`, `|`, `^`)

## Practical Guidance

- Treat parser support and evaluator support as separate capabilities.
- Use parser-focused tests when documenting syntax.
- Use REPL behavior when documenting what executes today.
