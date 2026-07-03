# Runtime Behavior and Limitations

This page describes current evaluator behavior in `flt-cli`.

## What Evaluates Today

The evaluator currently supports:

- Number literals and arithmetic: `+`, `-`, `*`, `/`
- Boolean literals and logic: `&&`, `||`, `^^`, unary `!`
- String literals and concatenation with `<>`
- Symbol and `None` literals
- Map literals (`{ key: value }`), evaluating to a `Map` value
- Array literals (`[ expr, ... ]`), evaluating each element in order to an `Array` value
- [Field access](./field-access.md) (`.field`) on maps
- Comparisons: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Parenthesized expressions
- Unary numeric `+` and `-`
- `if`/`else` expressions
- `let` bindings (`let x = expr`) and plain assignment (`x = expr`), stored in the global scope; evaluation returns the bound value (what the REPL prints)
- Identifier lookup against the global scope
- Calls to registered [built-in functions](./functions-and-pipe.md#built-in-functions) (currently `typeof`)

`<>` concatenation coerces values to strings (numbers, booleans, and symbols can be concatenated).

## Current Runtime Errors

- `Invalid Operand Type` - unsupported type/operator combination (e.g. `1 + true`, field access on a non-map)
- `Cannot compare {0} and {1}` - comparing values of mismatched/incomparable types
- `Division By Zero`
- `Unbound identifier: <name>` - referencing an identifier with no binding in scope
- `No such field: <name>` - field access on a map that doesn't have that field
- `Function calls not yet supported` - calling an unregistered function name, or evaluating a pipe (`|>`) expression
- `Function <name> expected <expected> argument(s), found <found>` - calling a built-in with the wrong arity

Separately, `typeof(None)` raises an interpreter-level error ("Not yet
implemented"), since `None` doesn't carry information about its wrapped
type.

## Parsed but Not Yet Evaluated

These constructs parse successfully but are not fully supported by the current evaluator:

- Pipe expressions (`|>`)
- Bitwise operators (`&`, `|`, `^`)
- Function calls to any name that isn't a registered built-in
- Standalone keyword expressions (e.g. `return`, `fn`) - they parse as expressions but have no evaluation behavior yet

## Practical Guidance

- Treat parser support and evaluator support as separate capabilities.
- Use parser-focused tests when documenting syntax.
- Use REPL behavior when documenting what executes today.
