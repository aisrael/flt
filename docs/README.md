# flt Language Documentation

This folder contains initial documentation for the `flt` language as it exists in this repository today.

## Contents

- [`spec.md`](spec.md) - One-page minimal language specification (grammar, types, operators, semantics).
- [`runtime.md`](runtime.md) - What the reference runtime does where the spec leaves a choice: overloads, scopes, constants, `return`, warnings, native types, built-ins, name resolution, what evaluates today, and limitations.
- [`quickstart.md`](quickstart.md) - Running `flt` and trying expressions in the REPL or with `-e`.
- [`syntax.md`](syntax.md) - Core expression forms, literals, comments, identifiers, reserved keywords, and statements.
- [`literals.md`](literals.md) - Integers and numbers, strings and interpolation, booleans, `None`, symbols, arrays and maps.
- [`types.md`](types.md) - Core flt types
  - [`types/none.md`](types/none.md) - `None`: the value, its type, and the result of `if` without `else`.
  - [`types/union_types.md`](types/union_types.md) - Union types (`A | B`) and the type of `if`.
- [`operators.md`](operators.md) - Unary/binary operators and precedence.
- [`field-access.md`](field-access.md) - Dot-access syntax for reading map fields (`u.foo`).
- [`functions-and-pipe.md`](functions-and-pipe.md) - Function call syntax (including keyword arguments), declaring functions with `def`, pattern matching in parameters, and pipe chaining.
- [`native-functions-and-values.md`](native-functions-and-values.md) - Native functions, types and values: extending flt from a Rust host.
- [`host-api.md`](host-api.md) - The `Runtime` trait: how a Rust host evaluates flt source and registers functions and types.
- [`errors.md`](errors.md) - The errors and warnings that the runtime raises, and when it raises them.

## Architecture Decision Records

The decisions behind the language are in [`adrs/`](adrs/), one file for each ADR.
