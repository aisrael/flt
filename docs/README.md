# flt Language Documentation

This folder contains initial documentation for the `flt` language as it exists in this repository today.

## Contents

- [`spec.md`](spec.md) - One-page minimal language specification (grammar, types, operators, semantics).
- [`quickstart.md`](quickstart.md) - Running `flt` and trying expressions in the REPL.
- [`syntax.md`](syntax.md) - Core expression forms, literals, comments, identifiers, reserved keywords, and statements.
- [`types.md`](types.md) - Core flt types
  - [`types/none.md`](types/none.md) - `None`: the value, its type, and the result of `if` without `else`.
- [`operators.md`](operators.md) - Unary/binary operators and precedence.
- [`field-access.md`](field-access.md) - Dot-access syntax for reading map fields (`u.foo`).
- [`functions-and-pipe.md`](functions-and-pipe.md) - Function call syntax (including keyword arguments) and pipe chaining.
- [`runtime-and-limitations.md`](runtime-and-limitations.md) - Evaluation behavior, errors, and current limitations.

## Scope

These docs describe the currently implemented parser and CLI evaluator for version `0.2.0`.
