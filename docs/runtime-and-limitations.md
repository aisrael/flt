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
- `if`/`else` expressions; an `if` without `else` evaluates to `None` when its condition is `false`
- `let` bindings (`let x = expr`) and plain assignment (`x = expr`), stored in the global scope; evaluation returns the bound value (what the REPL prints)
- Identifier lookup against the global scope: variables, then [native functions](./functions-and-pipe.md#native-functions) (e.g. `identity` evaluates to `<fn identity>`), then [built-in type names](./types.md#types-as-values) (e.g. `Number` evaluates to the `Number` type)
- [Pipes](./functions-and-pipe.md#pipe-operator): `x |> f(a)` evaluates as `f(x, a)`, and `x |> f` as `f(x)`
- Calls to registered [built-in functions](./functions-and-pipe.md#built-in-functions) (currently `typeof`) and to host-registered [native functions](./functions-and-pipe.md#native-functions), including through any name a function value is bound to

`<>` concatenation coerces values to strings (numbers, booleans, and symbols can be concatenated).

## Current Runtime Errors

- `Invalid Operand Type` - unsupported type/operator combination (e.g. `1 + true`, field access on a non-map)
- `Cannot compare {0} and {1}` - comparing values of mismatched/incomparable types
- `Division By Zero`
- `Unbound identifier: <name>` - referencing an identifier with no binding in scope
- `No such field: <name>` - field access on a map that doesn't have that field
- `Undefined function: <name>` - calling a name that has no binding
- `<name> is not a function` - calling a name bound to a value that isn't a function
- `Cannot pipe into <expr>` - piping into something other than a call or a function name (e.g. `1 |> 2`)
- `Function <name> expected <expected> argument(s), found <found>` - calling any function with the wrong number of arguments

## Parsed but Not Yet Evaluated

These constructs parse successfully but are not fully supported by the current evaluator:

- Bitwise operators (`&`, `|`, `^`)
- Standalone keyword expressions (e.g. `return`, `fn`) - they parse as expressions but have no evaluation behavior yet

## Practical Guidance

- Treat parser support and evaluator support as separate capabilities.
- Use parser-focused tests when documenting syntax.
- Use REPL behavior when documenting what executes today.
